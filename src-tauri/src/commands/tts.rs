use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use futures_util::{SinkExt, StreamExt};
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

const TRUSTED_CLIENT_TOKEN: &str = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";
const WSS_URL: &str =
    "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4&ConnectionId={UUID}&Sec-MS-GEC={GEC}&Sec-MS-GEC-Version=1-143.0.3650.75";
const VOICE_LIST_URL: &str =
    "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";
const WIN_EPOCH: f64 = 11644473600.0;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoiceInfo {
    pub name: String,
    pub locale: String,
    pub gender: String,
    pub description: String,
}

fn generate_sec_ms_gec() -> String {
    let unix_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    let mut ticks = unix_ts + WIN_EPOCH;
    ticks -= ticks % 300.0;
    ticks *= 1e9 / 100.0;

    let str_to_hash = format!("{:.0}{}", ticks, TRUSTED_CLIENT_TOKEN);

    let mut hasher = Sha256::new();
    hasher.update(str_to_hash.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash).to_uppercase()
}

fn generate_muid() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    hex::encode(bytes).to_uppercase()
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[tauri::command]
pub async fn tts_generate(
    text: String,
    voice: Option<String>,
    rate: Option<String>,
    pitch: Option<String>,
    volume: Option<String>,
) -> Result<String, String> {
    let voice = voice.unwrap_or_else(|| "zh-CN-XiaoxiaoNeural".to_string());
    let rate = rate.unwrap_or_else(|| "+0%".to_string());
    let pitch = pitch.unwrap_or_else(|| "+0Hz".to_string());
    let volume = volume.unwrap_or_else(|| "+0%".to_string());

    let escaped_text = xml_escape(&text);
    let request_id = uuid::Uuid::new_v4().to_string();
    let connection_id = uuid::Uuid::new_v4().to_string();
    let sec_ms_gec = generate_sec_ms_gec();
    let muid = generate_muid();

    let ws_url = WSS_URL
        .replace("{UUID}", &connection_id)
        .replace("{GEC}", &sec_ms_gec);

    let mut request = ws_url
        .into_client_request()
        .map_err(|e| format!("构建 WebSocket 请求失败: {}", e))?;

    let headers = request.headers_mut();
    headers.insert("Origin", "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold".parse().unwrap());
    headers.insert("Pragma", "no-cache".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert(
        "User-Agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0"
            .parse()
            .unwrap(),
    );
    headers.insert("Accept-Encoding", "gzip, deflate, br, zstd".parse().unwrap());
    headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
    headers.insert("Cookie", format!("muid={};", muid).parse().unwrap());

    let (ws_stream, _) = tokio_tungstenite::connect_async_tls_with_config(request, None, false, None)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;

    let (mut write, read_half) = ws_stream.split();

    let timestamp = chrono::Utc::now().format("%a %b %d %Y %H:%M:%S GMT+0000 (Coordinated Universal Time)").to_string();

    let config_msg = format!(
        "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"}},\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"}}}}}}}}",
        timestamp
    );
    write
        .send(Message::Text(config_msg.into()))
        .await
        .map_err(|e| format!("发送配置消息失败: {}", e))?;

    let ssml_msg = format!(
        "X-RequestId:{request_id}\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:{timestamp}Z\r\nPath:ssml\r\n\r\n<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'><voice name='{voice}'><prosody pitch='{pitch}' rate='{rate}' volume='{volume}'>{escaped_text}</prosody></voice></speak>"
    );
    write
        .send(Message::Text(ssml_msg.into()))
        .await
        .map_err(|e| format!("发送 SSML 消息失败: {}", e))?;

    drop(write);

    let result = timeout(Duration::from_secs(30), async move {
        let mut audio_buffer: Vec<u8> = Vec::new();
        let mut read = read_half;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if text.contains("Path:turn.end") {
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    if data.len() < 2 {
                        continue;
                    }
                    let header_length = u16::from_be_bytes([data[0], data[1]]) as usize;
                    if header_length > data.len() {
                        continue;
                    }
                    let audio_start = header_length + 2;
                    if audio_start < data.len() {
                        audio_buffer.extend_from_slice(&data[audio_start..]);
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => return Err(format!("WebSocket 消息读取错误: {}", e)),
                _ => {}
            }
        }

        if audio_buffer.is_empty() {
            return Err("未接收到音频数据".to_string());
        }

        let base64_audio = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &audio_buffer,
        );
        Ok(base64_audio)
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => Err("TTS 生成超时 (30秒)".to_string()),
    }
}

#[tauri::command]
pub async fn tts_list_voices() -> Result<Vec<VoiceInfo>, String> {
    let muid = generate_muid();

    let client = reqwest::Client::new();

    let resp = client
        .get(VOICE_LIST_URL)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0")
        .header("Accept-Encoding", "gzip, deflate, br, zstd")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Sec-CH-UA", "\" Not;A Brand\";v=\"99\", \"Microsoft Edge\";v=\"143\", \"Chromium\";v=\"143\"")
        .header("Sec-CH-UA-Mobile", "?0")
        .header("Accept", "*/*")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Dest", "empty")
        .header("Authority", "speech.platform.bing.com")
        .header("Cookie", format!("muid={};", muid))
        .send()
        .await
        .map_err(|e| format!("获取语音列表失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("获取语音列表失败, HTTP status: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析语音列表失败: {}", e))?;

    let voices = body
        .as_array()
        .cloned()
        .unwrap_or_default();

    let filtered: Vec<VoiceInfo> = voices
        .iter()
        .filter_map(|v| {
            let locale = v.get("Locale")?.as_str()?.to_string();
            let name = v.get("ShortName")?.as_str()?.to_string();
            let gender = v.get("Gender").and_then(|g| g.as_str()).unwrap_or("Unknown").to_string();
            let description = v
                .get("FriendlyName")
                .and_then(|d| d.as_str())
                .unwrap_or(&name)
                .to_string();

            if (locale.starts_with("zh-") || locale.starts_with("en-")) && name.contains("Neural")
            {
                Some(VoiceInfo {
                    name,
                    locale,
                    gender,
                    description,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(filtered)
}
