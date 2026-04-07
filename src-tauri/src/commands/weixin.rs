use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use reqwest::Client;

const ILINK_BASE: &str = "https://ilinkai.weixin.qq.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QrCodeResponse {
    qrcode: Option<String>,
    qrcode_img_content: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QrCodeStatusResponse {
    status: Option<String>,
    bot_token: Option<String>,
    ilink_bot_id: Option<String>,
    ilink_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetUpdatesResponse {
    msgs: Option<Vec<serde_json::Value>>,
    get_updates_buf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeixinInboundEvent {
    account_id: String,
    from_user_id: String,
    text: String,
    context_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeixinStatusEvent {
    account_id: String,
    status: String,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeixinQrCodeEvent {
    account_id: String,
    qrcode_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeixinLoginResultEvent {
    account_id: String,
    success: bool,
    error: Option<String>,
    bot_token: Option<String>,
    ilink_bot_id: Option<String>,
    ilink_user_id: Option<String>,
}

struct WeixinConnectionState {
    cancel_flag: Arc<AtomicBool>,
    status: String,
    bot_token: String,
    ilink_bot_id: String,
    ilink_user_id: String,
    context_token: String,
    get_updates_buf: String,
}

struct WeixinCredentials {
    bot_token: String,
    ilink_bot_id: String,
    ilink_user_id: String,
    context_token: String,
}

pub struct WeixinManager {
    connections: Arc<RwLock<HashMap<String, WeixinConnectionState>>>,
    http: Client,
}

impl WeixinManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            http: Client::builder()
                .timeout(Duration::from_secs(40))
                .build()
                .unwrap_or_default(),
        }
    }
}

fn credentials_path(account_id: &str) -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("AuroraFairy");
    path.push("weixin");
    let _ = std::fs::create_dir_all(&path);
    path.push(format!("{}.json", account_id));
    path
}

fn save_credentials(account_id: &str, creds: &WeixinCredentials) {
    let path = credentials_path(account_id);
    let json = serde_json::json!({
        "bot_token": creds.bot_token,
        "ilink_bot_id": creds.ilink_bot_id,
        "ilink_user_id": creds.ilink_user_id,
        "context_token": creds.context_token,
    });
    let _ = std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default());
}

fn load_credentials(account_id: &str) -> Option<WeixinCredentials> {
    let path = credentials_path(account_id);
    let content = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    Some(WeixinCredentials {
        bot_token: v.get("bot_token")?.as_str()?.to_string(),
        ilink_bot_id: v.get("ilink_bot_id")?.as_str()?.to_string(),
        ilink_user_id: v.get("ilink_user_id")?.as_str()?.to_string(),
        context_token: v.get("context_token").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    })
}

fn build_headers(token: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("AuthorizationType", "ilink_bot_token".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );
    use base64::Engine;
    let uin = base64::engine::general_purpose::STANDARD
        .encode(format!("{}", rand::random::<u32>()));
    headers.insert("X-WECHAT-UIN", uin.parse().unwrap());
    headers
}

async fn poll_messages(
    http: &Client,
    token: &str,
    account_id: &str,
    app_handle: &tauri::AppHandle,
    _cancel_flag: &AtomicBool,
    connections: &Arc<RwLock<HashMap<String, WeixinConnectionState>>>,
    cursor: &mut String,
) -> Result<(), String> {
    let body = serde_json::json!({
        "get_updates_buf": cursor,
        "base_info": { "channel_version": "1.0.3" }
    });
    let body_bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;

    let mut headers = build_headers(token);
    headers.insert("Content-Length", body_bytes.len().to_string().parse().unwrap());

    let resp = http
        .post(format!("{}/ilink/bot/getupdates", ILINK_BASE))
        .headers(headers)
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = resp.text().await.map_err(|e| e.to_string())?;
    if text.is_empty() || text == "{}" {
        return Ok(());
    }

    let result: GetUpdatesResponse = serde_json::from_str(&text).unwrap_or(GetUpdatesResponse {
        msgs: None,
        get_updates_buf: None,
    });

    if let Some(buf) = result.get_updates_buf {
        *cursor = buf;
    }

    if let Some(msgs) = result.msgs {
        for msg in msgs {
            let from_user_id = msg.get("from_user_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let context_token = msg.get("context_token")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let mut text_parts: Vec<String> = Vec::new();
            if let Some(items) = msg.get("item_list").and_then(|v| v.as_array()) {
                for item in items {
                    let item_type = item.get("type").and_then(|v| v.as_i64()).unwrap_or(0);
                    if item_type == 1 {
                        if let Some(text) = item.get("text_item").and_then(|t| t.get("text")).and_then(|v| v.as_str()) {
                            text_parts.push(text.to_string());
                        }
                    }
                }
            }

            let text = text_parts.join("");
            if text.is_empty() {
                continue;
            }

            if !context_token.is_empty() {
                let mut conns = connections.write().await;
                if let Some(conn) = conns.get_mut(account_id) {
                    conn.context_token = context_token.clone();
                    save_credentials(account_id, &WeixinCredentials {
                        bot_token: conn.bot_token.clone(),
                        ilink_bot_id: conn.ilink_bot_id.clone(),
                        ilink_user_id: conn.ilink_user_id.clone(),
                        context_token: context_token.clone(),
                    });
                }
            }

            let _ = app_handle.emit("weixin-message", WeixinInboundEvent {
                account_id: account_id.to_string(),
                from_user_id: from_user_id.clone(),
                text,
                context_token,
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn weixin_get_qrcode(
    manager: tauri::State<'_, Arc<Mutex<WeixinManager>>>,
    app: tauri::AppHandle,
    account_id: String,
) -> Result<String, String> {
    let mgr = manager.lock().await;
    let resp = mgr.http
        .get(format!("{}/ilink/bot/get_bot_qrcode?bot_type=3", ILINK_BASE))
        .send()
        .await
        .map_err(|e| format!("获取二维码失败: {}", e))?;

    let data: QrCodeResponse = resp.json().await.map_err(|e| format!("解析二维码响应失败: {}", e))?;

    let qrcode_key = data.qrcode.ok_or("二维码 key 为空")?;
    let qrcode_url = data.qrcode_img_content.ok_or("二维码图片 URL 为空")?;

    let code = qrcode::QrCode::new(qrcode_url.as_bytes())
        .map_err(|e| format!("生成二维码失败: {}", e))?;
    let svg = code.render::<qrcode::render::svg::Color>()
        .min_dimensions(256, 256)
        .quiet_zone(false)
        .build();
    let svg_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, svg.as_bytes());
    let data_uri = format!("data:image/svg+xml;base64,{}", svg_b64);

    let _ = app.emit("weixin-qrcode", WeixinQrCodeEvent {
        account_id: account_id.clone(),
        qrcode_url: data_uri.clone(),
    });

    let http = mgr.http.clone();
    let connections_clone = mgr.connections.clone();
    let app_handle = app.clone();
    let account_id_clone = account_id.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(3)).await;

            let resp = match http
                .get(format!(
                    "{}/ilink/bot/get_qrcode_status?qrcode={}",
                    ILINK_BASE, qrcode_key
                ))
                .header("iLink-App-ClientVersion", "1")
                .timeout(Duration::from_secs(40))
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[Weixin] QR status request error: {}", e);
                    continue;
                }
            };

            let status_text = match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[Weixin] QR status read error: {}", e);
                    continue;
                }
            };

            eprintln!("[Weixin] QR status raw: {}", status_text);

            let status_data: QrCodeStatusResponse = match serde_json::from_str(&status_text) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[Weixin] QR status parse error: {}", e);
                    continue;
                }
            };

            let status = status_data.status.clone().unwrap_or_default();
            eprintln!("[Weixin] QR status: '{}', bot_token={}, ilink_bot_id={}, ilink_user_id={}", 
                status, 
                status_data.bot_token.as_deref().unwrap_or("None"),
                status_data.ilink_bot_id.as_deref().unwrap_or("None"),
                status_data.ilink_user_id.as_deref().unwrap_or("None"));

            match status.as_str() {
                "scaned" => {
                    let _ = app_handle.emit("weixin-login-status", WeixinLoginResultEvent {
                        account_id: account_id_clone.clone(),
                        success: false,
                        error: Some("已扫码，请在手机上确认…".to_string()),
                        bot_token: None,
                        ilink_bot_id: None,
                        ilink_user_id: None,
                    });
                }
                "confirmed" => {
                    let bot_token = status_data.bot_token.unwrap_or_default();
                    let ilink_bot_id = status_data.ilink_bot_id.unwrap_or_default();
                    let ilink_user_id = status_data.ilink_user_id.unwrap_or_default();

                    if bot_token.is_empty() {
                        let _ = app_handle.emit("weixin-login-status", WeixinLoginResultEvent {
                            account_id: account_id_clone.clone(),
                            success: false,
                            error: Some("登录失败：token 为空".to_string()),
                            bot_token: None,
                            ilink_bot_id: None,
                            ilink_user_id: None,
                        });
                        return;
                    }

                    save_credentials(&account_id_clone, &WeixinCredentials {
                        bot_token: bot_token.clone(),
                        ilink_bot_id: ilink_bot_id.clone(),
                        ilink_user_id: ilink_user_id.clone(),
                        context_token: String::new(),
                    });

                    {
                        let mut conns = connections_clone.write().await;
                        conns.insert(account_id_clone.clone(), WeixinConnectionState {
                            cancel_flag: Arc::new(AtomicBool::new(false)),
                            status: "connected".to_string(),
                            bot_token: bot_token.clone(),
                            ilink_bot_id: ilink_bot_id.clone(),
                            ilink_user_id: ilink_user_id.clone(),
                            context_token: String::new(),
                            get_updates_buf: String::new(),
                        });
                    }

                    let event = WeixinLoginResultEvent {
                        account_id: account_id_clone.clone(),
                        success: true,
                        error: None,
                        bot_token: Some(bot_token),
                        ilink_bot_id: Some(ilink_bot_id),
                        ilink_user_id: Some(ilink_user_id),
                    };
                    eprintln!("[Weixin] Emitting login-success event: {:?}", event);
                    if let Err(e) = app_handle.emit("weixin-login-status", event) {
                        eprintln!("[Weixin] emit login-status FAILED: {}", e);
                    } else {
                        eprintln!("[Weixin] emit login-status OK");
                    }
                    return;
                }
                "expired" => {
                    let _ = app_handle.emit("weixin-login-status", WeixinLoginResultEvent {
                        account_id: account_id_clone.clone(),
                        success: false,
                        error: Some("二维码已过期，请重新获取".to_string()),
                        bot_token: None,
                        ilink_bot_id: None,
                        ilink_user_id: None,
                    });
                    return;
                }
                _ => {}
            }
        }
    });

    Ok(data_uri)
}

#[tauri::command]
pub async fn weixin_connect(
    manager: tauri::State<'_, Arc<Mutex<WeixinManager>>>,
    app: tauri::AppHandle,
    account_id: String,
) -> Result<(), String> {
    let creds = load_credentials(&account_id)
        .ok_or("未找到微信登录凭证，请先扫码登录")?;

    let mgr = manager.lock().await;

    {
        let mut conns = mgr.connections.write().await;
        if let Some(old) = conns.remove(&account_id) {
            old.cancel_flag.store(true, Ordering::Relaxed);
        }
    }

    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut conns = mgr.connections.write().await;
        conns.insert(account_id.clone(), WeixinConnectionState {
            cancel_flag: cancel_flag.clone(),
            status: "connecting".to_string(),
            bot_token: creds.bot_token.clone(),
            ilink_bot_id: creds.ilink_bot_id.clone(),
            ilink_user_id: creds.ilink_user_id.clone(),
            context_token: creds.context_token.clone(),
            get_updates_buf: String::new(),
        });
    }

    let http = mgr.http.clone();
    let connections = mgr.connections.clone();
    let account_id_clone = account_id.clone();
    let app_handle = app.clone();

    tokio::spawn(async move {
        let _ = app_handle.emit("weixin-status", WeixinStatusEvent {
            account_id: account_id_clone.clone(),
            status: "connected".to_string(),
            error: None,
        });

        {
            let mut conns = connections.write().await;
            if let Some(conn) = conns.get_mut(&account_id_clone) {
                conn.status = "connected".to_string();
            }
        }

        let mut cursor = String::new();
        {
            let conns = connections.read().await;
            if let Some(conn) = conns.get(&account_id_clone) {
                cursor = conn.get_updates_buf.clone();
            }
        }

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            let token = {
                let conns = connections.read().await;
                match conns.get(&account_id_clone) {
                    Some(conn) => conn.bot_token.clone(),
                    None => break,
                }
            };

            let result = poll_messages(
                &http,
                &token,
                &account_id_clone,
                &app_handle,
                &cancel_flag,
                &connections,
                &mut cursor,
            ).await;

            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            if let Err(e) = result {
                eprintln!("[Weixin] Poll error: {}, reconnecting in 5s...", e);
                let _ = app_handle.emit("weixin-status", WeixinStatusEvent {
                    account_id: account_id_clone.clone(),
                    status: "error".to_string(),
                    error: Some(e),
                });
                sleep(Duration::from_secs(5)).await;
            }
        }

        let _ = app_handle.emit("weixin-status", WeixinStatusEvent {
            account_id: account_id_clone.clone(),
            status: "disconnected".to_string(),
            error: None,
        });
    });

    Ok(())
}

#[tauri::command]
pub async fn weixin_disconnect(
    manager: tauri::State<'_, Arc<Mutex<WeixinManager>>>,
    account_id: String,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    let mut conns = mgr.connections.write().await;
    if let Some(conn) = conns.remove(&account_id) {
        conn.cancel_flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub async fn weixin_reply_message(
    manager: tauri::State<'_, Arc<Mutex<WeixinManager>>>,
    account_id: String,
    to_user_id: String,
    text: String,
    context_token: String,
) -> Result<(), String> {
    let mgr = manager.lock().await;

    let (token, stored_ctx) = {
        let conns = mgr.connections.read().await;
        match conns.get(&account_id) {
            Some(conn) => (conn.bot_token.clone(), conn.context_token.clone()),
            None => return Err("微信未连接".to_string()),
        }
    };

    let ctx = if context_token.is_empty() { stored_ctx } else { context_token };

    let uid = uuid::Uuid::new_v4().to_string().replace("-", "");
    let client_id = format!("aurora-{}", &uid[..16]);

    let body = serde_json::json!({
        "msg": {
            "from_user_id": "",
            "to_user_id": to_user_id,
            "client_id": client_id,
            "message_type": 2,
            "message_state": 2,
            "context_token": ctx,
            "item_list": [{
                "type": 1,
                "text_item": { "text": text }
            }]
        },
        "base_info": { "channel_version": "1.0.3" }
    });

    let body_bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;

    let mut headers = build_headers(&token);
    headers.insert("Content-Length", body_bytes.len().to_string().parse().unwrap());

    mgr.http
        .post(format!("{}/ilink/bot/sendmessage", ILINK_BASE))
        .headers(headers)
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| format!("发送消息失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn weixin_get_status(
    manager: tauri::State<'_, Arc<Mutex<WeixinManager>>>,
    account_id: String,
) -> Result<String, String> {
    let mgr = manager.lock().await;
    let conns = mgr.connections.read().await;
    match conns.get(&account_id) {
        Some(conn) => Ok(conn.status.clone()),
        None => Ok("disconnected".to_string()),
    }
}

#[tauri::command]
pub async fn weixin_has_credentials(
    account_id: String,
) -> Result<bool, String> {
    Ok(load_credentials(&account_id).is_some())
}
