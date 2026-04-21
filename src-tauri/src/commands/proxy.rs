use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChatRequest {
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyChatResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

#[tauri::command]
pub async fn proxy_chat(request: ProxyChatRequest) -> Result<ProxyChatResponse, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let body_bytes = serde_json::to_vec(&request.body)
        .map_err(|e| format!("序列化body失败: {}", e))?;

    let mut req_builder = client.post(&request.url);

    for (key, value) in &request.headers {
        req_builder = req_builder.header(key.as_str(), value.as_str());
    }

    let resp = req_builder
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| format!("代理请求失败: {}", e))?;

    let status = resp.status().as_u16();

    let mut response_headers = std::collections::HashMap::new();
    for (key, value) in resp.headers().iter() {
        if let Ok(v) = value.to_str() {
            response_headers.insert(key.to_string(), v.to_string());
        }
    }

    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应体失败: {}", e))?;

    Ok(ProxyChatResponse {
        status,
        headers: response_headers,
        body,
    })
}

#[tauri::command]
pub async fn proxy_chat_stream(
    app: tauri::AppHandle,
    request: ProxyChatRequest,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let body_bytes = serde_json::to_vec(&request.body)
        .map_err(|e| format!("序列化body失败: {}", e))?;

    let mut req_builder = client.post(&request.url);

    for (key, value) in &request.headers {
        req_builder = req_builder.header(key.as_str(), value.as_str());
    }

    let resp = req_builder
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| format!("代理流式请求失败: {}", e))?;

    let status = resp.status();

    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("代理请求失败: HTTP {} - {}", status, text));
    }

    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let _ = app.emit("proxy-chat-chunk", bytes.to_vec());
            }
            Err(e) => {
                let _ = app.emit("proxy-chat-error", e.to_string());
                break;
            }
        }
    }

    let _ = app.emit("proxy-chat-done", ());

    Ok(())
}