use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use bytes::Bytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenResponse {
    code: i32,
    tenant_access_token: Option<String>,
    expire: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReplyResponse {
    code: i32,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeishuEventPayload {
    schema: Option<String>,
    header: Option<FeishuEventHeader>,
    event: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeishuEventHeader {
    event_id: Option<String>,
    event_type: Option<String>,
    create_time: Option<String>,
    token: Option<String>,
    app_id: Option<String>,
    tenant_key: Option<String>,
}

struct ConnectionState {
    cancel_flag: Arc<AtomicBool>,
    status: String,
    token: Option<String>,
    token_expires_at: Option<std::time::Instant>,
}

pub struct FeishuManager {
    connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
}

impl FeishuManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for FeishuManager {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_domain(domain: &str) -> String {
    match domain {
        "feishu" => "https://open.feishu.cn".to_string(),
        "lark" => "https://open.larksuite.com".to_string(),
        _ if domain.starts_with("http") => domain.to_string(),
        _ => "https://open.feishu.cn".to_string(),
    }
}

async fn get_tenant_token(domain: &str, app_id: &str, app_secret: &str) -> Result<(String, i64), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/open-apis/auth/v3/tenant_access_token/internal", domain);
    let resp = client.post(&url)
        .json(&serde_json::json!({
            "app_id": app_id,
            "app_secret": app_secret
        }))
        .send()
        .await
        .map_err(|e| format!("Token request failed: {}", e))?;

    let body: TokenResponse = resp.json().await
        .map_err(|e| format!("Token parse failed: {}", e))?;

    if body.code != 0 {
        return Err(format!("Token API error: code {}", body.code));
    }

    let token = body.tenant_access_token.ok_or("No token in response")?;
    let expire = body.expire.unwrap_or(7200);
    Ok((token, expire))
}

struct WsEndpointInfo {
    url: String,
    ping_interval: u64,
    service_id: u64,
}

async fn get_ws_endpoint(domain: &str, app_id: &str, app_secret: &str) -> Result<WsEndpointInfo, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/callback/ws/endpoint", domain);

    let resp = client.post(&url)
        .header("locale", "zh")
        .json(&serde_json::json!({
            "AppID": app_id,
            "AppSecret": app_secret
        }))
        .send()
        .await
        .map_err(|e| format!("Endpoint request failed: {}", e))?;

    let status = resp.status();
    let body_text = resp.text().await
        .map_err(|e| format!("Endpoint read body failed: {}", e))?;

    eprintln!("[Feishu] WS endpoint response status: {}", status);
    eprintln!("[Feishu] WS endpoint response body: {}", body_text);

    let body: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|e| format!("Endpoint parse failed: {} (body: {})", e, body_text))?;

    let code = body.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
    if code != 0 {
        let msg = body.get("msg").and_then(|m| m.as_str()).unwrap_or("unknown");
        return Err(format!("Endpoint API error: code {} msg {}", code, msg));
    }

    let data = body.get("data").ok_or_else(|| format!("No data in endpoint response: {}", body_text))?;

    let ws_url = data.get("URL")
        .or_else(|| data.get("url"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| format!("No URL in endpoint response data: {}", body_text))?
        .to_string();

    let ping_interval = data.get("ClientConfig")
        .and_then(|c| c.get("PingInterval"))
        .and_then(|v| v.as_u64())
        .unwrap_or(90);

    let mut service_id: u64 = 0;
    if let Ok(parsed) = url::Url::parse(&ws_url) {
        for (k, v) in parsed.query_pairs() {
            if k == "service_id" {
                service_id = v.parse().unwrap_or(0);
            }
        }
    }

    eprintln!("[Feishu] WS URL: {}..., PingInterval: {}s, ServiceID: {}",
        &ws_url[..ws_url.len().min(60)], ping_interval, service_id);

    if !ws_url.starts_with("wss://") && !ws_url.starts_with("ws://") {
        return Err(format!("Invalid WebSocket URL in endpoint response: {}", ws_url));
    }

    Ok(WsEndpointInfo {
        url: ws_url,
        ping_interval,
        service_id,
    })
}

async fn reply_message(domain: &str, token: &str, message_id: &str, text: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/open-apis/im/v1/messages/{}/reply", domain, message_id);

    let content = serde_json::json!({
        "zh_cn": {
            "content": [[{"tag": "md", "text": text}]]
        }
    });

    let resp = client.post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "content": serde_json::to_string(&content).unwrap_or_default(),
            "msg_type": "post"
        }))
        .send()
        .await
        .map_err(|e| format!("Reply request failed: {}", e))?;

    let body: ReplyResponse = resp.json().await
        .map_err(|e| format!("Reply parse failed: {}", e))?;

    if body.code != 0 {
        return Err(format!("Reply API error: code {}", body.code));
    }

    Ok(())
}

// === pbbp2 protobuf frame codec (matching lark-oapi SDK exactly) ===
// Frame { SeqID=1:uint64, LogID=2:uint64, service=3:int32, method=4:int32,
//         headers=5:repeated Header, payload_encoding=6:string, payload_type=7:string,
//         payload=8:bytes, LogIDNew=9:string }
// Header { key=1:string, value=2:string }
// method: 0=CONTROL, 1=DATA
// message type in header["type"]: "ping","pong","event","card"

fn decode_varint(data: &[u8], pos: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut p = pos;
    loop {
        if p >= data.len() { return None; }
        let byte = data[p];
        p += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 { break; }
        shift += 7;
        if shift >= 64 { return None; }
    }
    Some((result, p))
}

fn skip_field(data: &[u8], pos: usize, wire_type: u64) -> Option<usize> {
    match wire_type {
        0 => { let (_, np) = decode_varint(data, pos)?; Some(np) }
        1 => Some(pos + 8),
        2 => { let (len, np) = decode_varint(data, pos)?; Some(np + len as usize) }
        5 => Some(pos + 4),
        _ => None,
    }
}

struct PbFrame {
    seq_id: u64,
    log_id: u64,
    service: i32,
    method: i32,
    headers: Vec<(String, String)>,
    payload_encoding: Option<String>,
    payload_type: Option<String>,
    payload: Vec<u8>,
}

fn decode_frame(data: &[u8]) -> Option<PbFrame> {
    let mut pos = 0;
    let mut seq_id: u64 = 0;
    let mut log_id: u64 = 0;
    let mut service: i32 = 0;
    let mut method: i32 = 0;
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut payload_encoding: Option<String> = None;
    let mut payload_type: Option<String> = None;
    let mut payload: Vec<u8> = Vec::new();

    while pos < data.len() {
        let (raw_tag, new_pos) = decode_varint(data, pos)?;
        pos = new_pos;
        let field_number = raw_tag >> 3;
        let wire_type = raw_tag & 0x07;

        match (field_number, wire_type) {
            (1, 0) => { let (v, np) = decode_varint(data, pos)?; pos = np; seq_id = v; }
            (2, 0) => { let (v, np) = decode_varint(data, pos)?; pos = np; log_id = v; }
            (3, 0) => { let (v, np) = decode_varint(data, pos)?; pos = np; service = v as i32; }
            (4, 0) => { let (v, np) = decode_varint(data, pos)?; pos = np; method = v as i32; }
            (5, 2) => {
                let (len, np) = decode_varint(data, pos)?;
                pos = np;
                if pos + len as usize > data.len() { return None; }
                let entry_data = &data[pos..pos + len as usize];
                pos += len as usize;
                if let Some((k, v)) = decode_header_entry(entry_data) {
                    headers.push((k, v));
                }
            }
            (6, 2) => {
                let (len, np) = decode_varint(data, pos)?;
                pos = np;
                if pos + len as usize > data.len() { return None; }
                payload_encoding = Some(String::from_utf8_lossy(&data[pos..pos + len as usize]).to_string());
                pos += len as usize;
            }
            (7, 2) => {
                let (len, np) = decode_varint(data, pos)?;
                pos = np;
                if pos + len as usize > data.len() { return None; }
                payload_type = Some(String::from_utf8_lossy(&data[pos..pos + len as usize]).to_string());
                pos += len as usize;
            }
            (8, 2) => {
                let (len, np) = decode_varint(data, pos)?;
                pos = np;
                if pos + len as usize > data.len() { return None; }
                payload = data[pos..pos + len as usize].to_vec();
                pos += len as usize;
            }
            (9, 2) => {
                let (len, np) = decode_varint(data, pos)?;
                pos = np;
                pos += len as usize;
            }
            _ => { pos = skip_field(data, pos, wire_type)?; }
        }
    }

    Some(PbFrame {
        seq_id, log_id, service, method, headers,
        payload_encoding, payload_type, payload,
    })
}

fn decode_header_entry(data: &[u8]) -> Option<(String, String)> {
    let mut pos = 0;
    let mut key = String::new();
    let mut val = String::new();

    while pos < data.len() {
        let (raw_tag, new_pos) = decode_varint(data, pos)?;
        pos = new_pos;
        let field_number = raw_tag >> 3;
        let wire_type = raw_tag & 0x07;

        if wire_type != 2 { return None; }
        let (len, new_pos) = decode_varint(data, pos)?;
        pos = new_pos;
        if pos + len as usize > data.len() { return None; }
        let field_data = &data[pos..pos + len as usize];
        pos += len as usize;

        match field_number {
            1 => key = String::from_utf8_lossy(field_data).to_string(),
            2 => val = String::from_utf8_lossy(field_data).to_string(),
            _ => {}
        }
    }

    Some((key, val))
}

fn encode_varint(buf: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 { buf.push(byte); break; }
        buf.push(byte | 0x80);
    }
}

fn encode_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    encode_varint(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

fn encode_string_field(buf: &mut Vec<u8>, field_num: u64, val: &str) {
    encode_varint(buf, (field_num << 3) | 2);
    encode_bytes(buf, val.as_bytes());
}

fn encode_uint_field(buf: &mut Vec<u8>, field_num: u64, val: u64) {
    encode_varint(buf, (field_num << 3) | 0);
    encode_varint(buf, val);
}

fn encode_int_field(buf: &mut Vec<u8>, field_num: u64, val: i32) {
    encode_varint(buf, (field_num << 3) | 0);
    encode_varint(buf, val as u64);
}

fn build_ping_frame(service_id: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    encode_uint_field(&mut buf, 1, 0); // SeqID
    encode_uint_field(&mut buf, 2, 0); // LogID
    encode_int_field(&mut buf, 3, service_id as i32); // service
    encode_int_field(&mut buf, 4, 0); // method = CONTROL

    let mut header_buf = Vec::new();
    encode_string_field(&mut header_buf, 1, "type");
    encode_string_field(&mut header_buf, 2, "ping");
    encode_varint(&mut buf, (5 << 3) | 2);
    encode_bytes(&mut buf, &header_buf);

    buf
}

fn build_response_frame(original: &PbFrame, response_code: i32) -> Vec<u8> {
    let mut buf = Vec::new();
    encode_uint_field(&mut buf, 1, original.seq_id);
    encode_uint_field(&mut buf, 2, original.log_id);
    encode_int_field(&mut buf, 3, original.service);
    encode_int_field(&mut buf, 4, original.method); // DATA

    for (k, v) in &original.headers {
        let mut header_buf = Vec::new();
        encode_string_field(&mut header_buf, 1, k);
        encode_string_field(&mut header_buf, 2, v);
        encode_varint(&mut buf, (5 << 3) | 2);
        encode_bytes(&mut buf, &header_buf);
    }

    if let Some(ref enc) = original.payload_encoding {
        encode_string_field(&mut buf, 6, enc);
    }
    if let Some(ref pt) = original.payload_type {
        encode_string_field(&mut buf, 7, pt);
    }

    let resp_json = serde_json::json!({"code": response_code});
    let resp_bytes = serde_json::to_vec(&resp_json).unwrap_or_default();
    encode_varint(&mut buf, (8 << 3) | 2);
    encode_bytes(&mut buf, &resp_bytes);

    buf
}

fn get_header_value<'a>(headers: &'a [(String, String)], key: &str) -> Option<&'a str> {
    for (k, v) in headers {
        if k == key { return Some(v.as_str()); }
    }
    None
}

// === Tauri commands ===

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FeishuStatusEvent {
    account_id: String,
    status: String,
    error: Option<String>,
}

#[tauri::command]
pub async fn feishu_connect(
    app: tauri::AppHandle,
    manager: tauri::State<'_, Arc<Mutex<FeishuManager>>>,
    account_id: String,
    app_id: String,
    app_secret: String,
    domain: String,
) -> Result<(), String> {
    let base_domain = resolve_domain(&domain);

    let _ = app.emit("feishu-status", FeishuStatusEvent {
        account_id: account_id.clone(),
        status: "connecting".to_string(),
        error: None,
    });

    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mgr = manager.lock().await;
        {
            let mut conns = mgr.connections.write().await;
            if let Some(old) = conns.remove(&account_id) {
                old.cancel_flag.store(true, Ordering::Relaxed);
            }
        }
        mgr.connections.write().await.insert(account_id.clone(), ConnectionState {
            cancel_flag: cancel_flag.clone(),
            status: "connecting".to_string(),
            token: None,
            token_expires_at: None,
        });
    }

    let account_id_clone = account_id.clone();
    let app_handle = app.clone();
    let base_domain_clone = base_domain.clone();
    let app_id_clone = app_id.clone();
    let app_secret_clone = app_secret.clone();
    let connections_clone = {
        let mgr = manager.lock().await;
        mgr.connections.clone()
    };

    tokio::spawn(async move {
        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            let endpoint = match get_ws_endpoint(&base_domain_clone, &app_id_clone, &app_secret_clone).await {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("[Feishu] Get endpoint failed: {}", e);
                    let _ = app_handle.emit("feishu-status", FeishuStatusEvent {
                        account_id: account_id_clone.clone(),
                        status: "error".to_string(),
                        error: Some(format!("连接失败: {}", e)),
                    });
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let result = run_ws_loop(
                &endpoint,
                &account_id_clone,
                &app_handle,
                &cancel_flag,
                &connections_clone,
            ).await;

            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            match result {
                Ok(()) => break,
                Err(e) => {
                    eprintln!("[Feishu] WS disconnected: {}, reconnecting in 5s...", e);
                    let _ = app_handle.emit("feishu-status", FeishuStatusEvent {
                        account_id: account_id_clone.clone(),
                        status: "reconnecting".to_string(),
                        error: Some(e),
                    });
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    Ok(())
}

struct PendingChunks {
    chunks: Vec<Option<Vec<u8>>>,
}

async fn run_ws_loop(
    endpoint_info: &WsEndpointInfo,
    account_id: &str,
    app_handle: &tauri::AppHandle,
    cancel_flag: &AtomicBool,
    connections: &Arc<RwLock<HashMap<String, ConnectionState>>>,
) -> Result<(), String> {
    let (ws_stream, _) = connect_async(&endpoint_info.url)
        .await
        .map_err(|e| format!("WS connect failed: {}", e))?;

    let _ = app_handle.emit("feishu-status", FeishuStatusEvent {
        account_id: account_id.to_string(),
        status: "connected".to_string(),
        error: None,
    });

    {
        let mut conns = connections.write().await;
        if let Some(conn) = conns.get_mut(account_id) {
            conn.status = "connected".to_string();
        }
    }

    eprintln!("[Feishu] WS connected for account {}", account_id);

    let (mut write, mut read) = ws_stream.split();

    {
        let ping = build_ping_frame(endpoint_info.service_id);
        let _ = write.send(Message::Binary(Bytes::from(ping))).await;
        eprintln!("[Feishu] Initial ping sent (service_id={})", endpoint_info.service_id);
    }

    let mut ping_interval = interval(Duration::from_secs(endpoint_info.ping_interval));
    let service_id = endpoint_info.service_id;
    let write_ping = Arc::new(Mutex::new(write));
    let write_data = write_ping.clone();

    let ping_handle = tokio::spawn(async move {
        loop {
            ping_interval.tick().await;
            let mut w = write_ping.lock().await;
            let ping = build_ping_frame(service_id);
            if w.send(Message::Binary(Bytes::from(ping))).await.is_err() {
                break;
            }
            eprintln!("[Feishu] Ping sent");
        }
    });

    let account_id_msg = account_id.to_string();
    let app_handle_msg = app_handle.clone();

    let mut pending_chunks: HashMap<String, PendingChunks> = HashMap::new();

    while let Some(msg) = read.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            break;
        }

        match msg {
            Ok(Message::Binary(data)) => {
                        if let Some(frame) = decode_frame(&data) {
                            eprintln!("[Feishu] Frame: method={}, service={}, seq={}, headers_count={}, payload_len={}",
                                frame.method, frame.service, frame.seq_id, frame.headers.len(), frame.payload.len());

                            match frame.method {
                                0 => {
                                    let msg_type = get_header_value(&frame.headers, "type").unwrap_or("");
                                    match msg_type {
                                        "pong" => {
                                            if !frame.payload.is_empty() {
                                                if let Ok(config_str) = String::from_utf8(frame.payload.clone()) {
                                                    eprintln!("[Feishu] Pong config: {}", config_str);
                                                }
                                            }
                                            eprintln!("[Feishu] Pong received");
                                        }
                                        "ping" => {
                                            eprintln!("[Feishu] Server ping received (ignored, SDK doesn't respond)");
                                        }
                                        _ => {
                                            eprintln!("[Feishu] Control frame type: {}", msg_type);
                                        }
                                    }
                                }
                                1 => {
                                    let msg_type = get_header_value(&frame.headers, "type").unwrap_or("unknown");
                                    let msg_id = get_header_value(&frame.headers, "message_id").unwrap_or("").to_string();
                                    let sum_str = get_header_value(&frame.headers, "sum").unwrap_or("1");
                                    let seq_str = get_header_value(&frame.headers, "seq").unwrap_or("0");

                                    let sum: usize = sum_str.parse().unwrap_or(1);
                                    let seq: usize = seq_str.parse().unwrap_or(0);

                                    eprintln!("[Feishu] Data frame: type={}, msg_id={}, sum={}, seq={}", msg_type, msg_id, sum, seq);

                                    let pl = if sum > 1 {
                                        if seq == 0 {
                                            pending_chunks.remove(&msg_id);
                                        }
                                        let entry = pending_chunks.entry(msg_id.clone()).or_insert_with(|| PendingChunks {
                                            chunks: (0..sum).map(|_| None).collect(),
                                        });
                                        entry.chunks[seq] = Some(frame.payload.clone());

                                        let all_present = entry.chunks.iter().all(|c| c.is_some());
                                        if !all_present {
                                            eprintln!("[Feishu] Waiting for more chunks ({}/{})", entry.chunks.iter().filter(|c| c.is_some()).count(), sum);
                                            let resp_frame = build_response_frame(&frame, 200);
                                            let mut w = write_data.lock().await;
                                            let _ = w.send(Message::Binary(Bytes::from(resp_frame))).await;
                                            continue;
                                        }

                                        let mut combined = Vec::new();
                                        for chunk in &entry.chunks {
                                            if let Some(ref data) = chunk {
                                                combined.extend_from_slice(data);
                                            }
                                        }
                                        pending_chunks.remove(&msg_id);
                                        eprintln!("[Feishu] All chunks combined, total {} bytes", combined.len());
                                        combined
                                    } else {
                                        frame.payload.clone()
                                    };

                                    match msg_type {
                                        "event" => {
                                            match String::from_utf8(pl.clone()) {
                                                Ok(json_str) => {
                                                    eprintln!("[Feishu] Event payload (first 500): {}", &json_str[..json_str.len().min(500)]);
                                                    match serde_json::from_str::<FeishuEventPayload>(&json_str) {
                                                        Ok(event) => {
                                                            if let Some(header) = &event.header {
                                                                eprintln!("[Feishu] Event type: {:?}", header.event_type);
                                                                if header.event_type.as_deref() == Some("im.message.receive_v1") {
                                                                    if let Some(event_data) = &event.event {
                                                                        eprintln!("[Feishu] Emitting feishu-message event");
                                                                        let _ = app_handle_msg.emit("feishu-message", serde_json::json!({
                                                                            "accountId": account_id_msg,
                                                                            "eventType": "im.message.receive_v1",
                                                                            "event": event_data,
                                                                        }));
                                                                    }
                                                                }
                                                            } else {
                                                                eprintln!("[Feishu] Event has no header");
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("[Feishu] Event JSON parse error: {}", e);
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    eprintln!("[Feishu] Event payload not UTF-8, hex: {:02x?}", &pl[..pl.len().min(50)]);
                                                }
                                            }
                                        }
                                        "card" => {
                                            eprintln!("[Feishu] Card message received (ignored)");
                                        }
                                        _ => {
                                            eprintln!("[Feishu] Unknown data type: {}", msg_type);
                                        }
                                    }

                                    let resp_frame = build_response_frame(&frame, 200);
                                    let mut w = write_data.lock().await;
                                    let _ = w.send(Message::Binary(Bytes::from(resp_frame))).await;
                                    eprintln!("[Feishu] Response ACK sent");
                                }
                                _ => {
                                    eprintln!("[Feishu] Unknown method: {}", frame.method);
                                }
                            }
                        } else {
                            eprintln!("[Feishu] Failed to decode frame, hex: {:02x?}", &data[..data.len().min(50)]);
                        }
                    }
                    Ok(Message::Text(text)) => {
                        eprintln!("[Feishu] Text frame: {}", &text[..text.len().min(200)]);
                    }
                    Ok(Message::Close(frame)) => {
                        eprintln!("[Feishu] Close frame: {:?}", frame);
                        return Err("WS closed by server".to_string());
                    }
                    Ok(Message::Ping(data)) => {
                        eprintln!("[Feishu] WS-level Ping: {} bytes", data.len());
                    }
                    Ok(Message::Pong(data)) => {
                        eprintln!("[Feishu] WS-level Pong: {} bytes", data.len());
                    }
                    Err(e) => {
                        eprintln!("[Feishu] WS error: {}", e);
                        return Err(format!("WS error: {}", e));
                    }
                    _ => {}
        }
    }

    ping_handle.abort();
    Ok(())
}

#[tauri::command]
pub async fn feishu_disconnect(
    manager: tauri::State<'_, Arc<Mutex<FeishuManager>>>,
    account_id: String,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    let mut connections = mgr.connections.write().await;
    if let Some(conn) = connections.remove(&account_id) {
        conn.cancel_flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub async fn feishu_reply_message(
    manager: tauri::State<'_, Arc<Mutex<FeishuManager>>>,
    account_id: String,
    domain: String,
    app_id: String,
    app_secret: String,
    message_id: String,
    text: String,
) -> Result<(), String> {
    let base_domain = resolve_domain(&domain);

    let mgr = manager.lock().await;
    let connections = mgr.connections.read().await;

    let token = if let Some(conn) = connections.get(&account_id) {
        if let Some(ref token) = conn.token {
            if let Some(expires_at) = conn.token_expires_at {
                if std::time::Instant::now() < expires_at {
                    Some(token.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    drop(connections);

    let token = if let Some(t) = token {
        t
    } else {
        let (new_token, expire) = get_tenant_token(&base_domain, &app_id, &app_secret).await?;
        let mut connections = mgr.connections.write().await;
        if let Some(conn) = connections.get_mut(&account_id) {
            conn.token = Some(new_token.clone());
            conn.token_expires_at = Some(std::time::Instant::now() + Duration::from_secs(expire as u64 - 300));
        }
        new_token
    };

    reply_message(&base_domain, &token, &message_id, &text).await
}

#[tauri::command]
pub async fn feishu_get_status(
    manager: tauri::State<'_, Arc<Mutex<FeishuManager>>>,
    account_id: String,
) -> Result<String, String> {
    let mgr = manager.lock().await;
    let connections = mgr.connections.read().await;
    let status = connections.get(&account_id)
        .map(|c| c.status.clone())
        .unwrap_or_else(|| "disconnected".to_string());
    Ok(status)
}
