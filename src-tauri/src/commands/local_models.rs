use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::Emitter;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{interval, timeout, Duration};
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";

static DOWNLOAD_PROCESSES: std::sync::OnceLock<Arc<Mutex<HashMap<String, Child>>>> =
    std::sync::OnceLock::new();

fn get_download_processes() -> Arc<Mutex<HashMap<String, Child>>> {
    DOWNLOAD_PROCESSES
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallProgress {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentStatus {
    pub python: bool,
    pub python_version: Option<String>,
    pub modelscope: bool,
    pub ollama: bool,
    pub ollama_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSearchResult {
    pub model_id: String,
    pub name: String,
    pub description: Option<String>,
    pub downloads: Option<u64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub model_id: String,
    pub status: String,
    pub current_file: String,
    pub progress_percent: f64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub model_id: String,
    pub status: String,
    pub port: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeployConfig {
    pub ctx_size: u32,
    pub threads: u32,
    pub gpu_layers: u32,
    pub port: u16,
}

async fn ollama_api_get(path: &str) -> Result<serde_json::Value, String> {
    let url = format!("{}{}", OLLAMA_BASE_URL, path);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Ollama API 请求失败: {} (请确认 Ollama 正在运行)", e))?;
    if !resp.status().is_success() {
        return Err(format!("Ollama API 返回错误: HTTP {}", resp.status()));
    }
    resp.json()
        .await
        .map_err(|e| format!("解析 Ollama API 响应失败: {}", e))
}

async fn ollama_api_post_json(path: &str, body: serde_json::Value) -> Result<(), String> {
    let url = format!("{}{}", OLLAMA_BASE_URL, path);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama API 请求失败: {}", e))?;
    if !resp.status().is_success() && resp.status().as_u16() != 200 {
        return Err(format!("Ollama API 返回错误: HTTP {}", resp.status()));
    }
    let _ = resp.bytes().await;
    Ok(())
}

async fn upload_blob(file_path: &Path) -> Result<String, String> {
    let file_content = fs::read(file_path)
        .map_err(|e| format!("读取文件失败: {} - {}", file_path.display(), e))?;

    let mut hasher = Sha256::new();
    hasher.update(&file_content);
    let digest = format!("sha256:{:x}", hasher.finalize());

    let url = format!("{}/api/blobs/{}", OLLAMA_BASE_URL, digest);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let head_resp = client
        .head(&url)
        .send()
        .await
        .map_err(|e| format!("检查 blob 失败: {}", e))?;

    if head_resp.status().as_u16() == 200 {
        println!("[upload_blob] Blob already exists: {}", digest);
        return Ok(digest);
    }

    println!(
        "[upload_blob] Uploading {} ({} bytes) -> {}",
        file_path.display(),
        file_content.len(),
        digest
    );

    let resp = client
        .post(&url)
        .header("Content-Type", "application/octet-stream")
        .body(file_content)
        .send()
        .await
        .map_err(|e| format!("上传文件失败: {} - {}", file_path.display(), e))?;

    if resp.status().as_u16() != 201 {
        return Err(format!(
            "上传文件失败: {} - HTTP {}",
            file_path.display(),
            resp.status()
        ));
    }

    Ok(digest)
}

#[tauri::command]
pub async fn check_environment() -> Result<EnvironmentStatus, String> {
    let python_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "python", "--version"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    let (python, python_version) = match python_check {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let version_output = if !stdout.is_empty() { &stdout } else { &stderr };
            if output.status.success() {
                let version = version_output.trim().to_string();
                (true, Some(version))
            } else {
                (false, None)
            }
        }
        _ => (false, None),
    };

    let modelscope_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "python", "-c", "import modelscope"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    let modelscope = match modelscope_check {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    };

    let ollama_check = timeout(Duration::from_secs(5), async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build();
        match client {
            Ok(c) => {
                let resp = c.get(format!("{}/api/version", OLLAMA_BASE_URL)).send().await;
                match resp {
                    Ok(r) if r.status().is_success() => {
                        let body: serde_json::Value = r.json().await.unwrap_or_default();
                        let version = body
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        Ok::<(bool, Option<String>), ()>((true, Some(version)))
                    }
                    _ => Ok::<(bool, Option<String>), ()>((false, None)),
                }
            }
            Err(_) => Ok::<(bool, Option<String>), ()>((false, None)),
        }
    })
    .await;

    let (ollama, ollama_version) = match ollama_check {
        Ok(Ok((has, ver))) => (has, ver),
        _ => (false, None),
    };

    Ok(EnvironmentStatus {
        python,
        python_version,
        modelscope,
        ollama,
        ollama_version,
    })
}

#[tauri::command]
pub async fn search_models(keyword: String) -> Result<Vec<ModelSearchResult>, String> {
    let encoded_keyword: String = keyword
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u8)
            }
        })
        .collect();
    let url = format!(
        "https://modelscope.cn/api/v1/models?keyword={}&limit=20",
        encoded_keyword
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("搜索模型失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("搜索模型请求失败, HTTP状态: {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析搜索结果失败: {}", e))?;

    let models = body
        .get("Data")
        .and_then(|d| d.get("Models"))
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();

    let results: Vec<ModelSearchResult> = models
        .iter()
        .filter_map(|m| {
            let model_id = m
                .get("Name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            if model_id.is_empty() {
                return None;
            }
            let name = m
                .get("Name")
                .and_then(|n| n.as_str())
                .unwrap_or(&model_id)
                .to_string();
            let description = m
                .get("Description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());
            let downloads = m.get("Downloads").and_then(|d| d.as_u64());
            let tags = m
                .get("Tags")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|tag| tag.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(ModelSearchResult {
                model_id,
                name,
                description,
                downloads,
                tags,
            })
        })
        .collect();

    Ok(results)
}

#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    model_id: String,
    local_dir: String,
) -> Result<String, String> {
    let processes = get_download_processes();
    {
        let map = processes.lock().await;
        if map.contains_key(&model_id) {
            return Err(format!("模型 {} 正在下载中", model_id));
        }
    }

    let project_root = crate::get_project_root()?;
    let abs_local_dir = project_root.join(&local_dir);
    let script_path = project_root.join("scripts").join("download_model.py");

    println!("[download_model] project_root: {:?}", project_root);
    println!("[download_model] script_path: {:?}", script_path);
    println!("[download_model] abs_local_dir: {:?}", abs_local_dir);

    if !script_path.exists() {
        return Err(format!("下载脚本不存在: {:?}", script_path));
    }

    let models_dir = project_root.join("models");
    if !models_dir.exists() {
        let _ = fs::create_dir_all(&models_dir);
    }
    let progress_file_path = models_dir.join(format!(
        ".download_progress_{}.json",
        model_id.replace('/', "_")
    ));
    let progress_file_str = progress_file_path.to_string_lossy().to_string();

    let script_str = script_path.to_string_lossy().to_string();
    let local_dir_str = abs_local_dir.to_string_lossy().to_string();
    let mut cmd = Command::new("python");
    cmd.args([
        script_str.as_str(),
        model_id.as_str(),
        local_dir_str.as_str(),
        progress_file_str.as_str(),
    ])
    .current_dir(&project_root)
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let child = cmd
        .spawn()
        .map_err(|e| format!("启动下载进程失败: {}", e))?;

    {
        let mut map = processes.lock().await;
        map.insert(model_id.clone(), child);
    }

    let app_clone = app.clone();
    let model_id_clone = model_id.clone();
    let processes_clone = processes.clone();

    tokio::spawn(async move {
        let mut last_status = String::new();
        let mut ticker = interval(Duration::from_millis(500));

        loop {
            ticker.tick().await;

            {
                let map = processes_clone.lock().await;
                if !map.contains_key(&model_id_clone) {
                    break;
                }
            }

            if Path::new(&progress_file_str).exists() {
                if let Ok(content) = fs::read_to_string(&progress_file_str) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        let status = data
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let current_file = data
                            .get("current_file")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let percent = data
                            .get("progress_percent")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        let message = data
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let progress = DownloadProgress {
                            model_id: model_id_clone.clone(),
                            status: status.clone(),
                            current_file: current_file.clone(),
                            progress_percent: percent,
                            message: message.clone(),
                        };
                        let _ = app_clone.emit("model-download-progress", progress);

                        if status == "completed" || status == "error" {
                            let _ = fs::remove_file(&progress_file_str);
                            break;
                        }
                        last_status = status;
                    }
                }
            }
        }

        let mut map = processes_clone.lock().await;
        if let Some(mut child) = map.remove(&model_id_clone) {
            let exit_status = child.wait().await;
            match exit_status {
                Ok(status) if status.success() => {
                    if last_status != "completed" {
                        let progress = DownloadProgress {
                            model_id: model_id_clone.clone(),
                            status: "completed".to_string(),
                            current_file: String::new(),
                            progress_percent: 100.0,
                            message: "下载完成".to_string(),
                        };
                        let _ = app_clone.emit("model-download-progress", progress);
                    }
                }
                Ok(status) => {
                    println!(
                        "[download_model] Download FAILED for {}: exit code: {}",
                        model_id_clone, status
                    );
                    if last_status != "completed" && last_status != "error" {
                        let progress = DownloadProgress {
                            model_id: model_id_clone.clone(),
                            status: "error".to_string(),
                            current_file: String::new(),
                            progress_percent: 0.0,
                            message: format!("下载失败，退出码: {}", status),
                        };
                        let _ = app_clone.emit("model-download-progress", progress);
                    }
                }
                Err(e) => {
                    println!(
                        "[download_model] Download EXCEPTION for {}: {}",
                        model_id_clone, e
                    );
                    if last_status != "completed" && last_status != "error" {
                        let progress = DownloadProgress {
                            model_id: model_id_clone.clone(),
                            status: "error".to_string(),
                            current_file: String::new(),
                            progress_percent: 0.0,
                            message: format!("下载进程异常: {}", e),
                        };
                        let _ = app_clone.emit("model-download-progress", progress);
                    }
                }
            }
        }

        let _ = fs::remove_file(&progress_file_str);
    });

    Ok(local_dir)
}

#[tauri::command]
pub async fn cancel_download(app: tauri::AppHandle, model_id: String) -> Result<(), String> {
    let processes = get_download_processes();
    let mut map = processes.lock().await;

    if let Some(mut child) = map.remove(&model_id) {
        child.kill()
            .await
            .map_err(|e| format!("终止下载进程失败: {}", e))?;
    }

    let progress = DownloadProgress {
        model_id: model_id.clone(),
        status: "cancelled".to_string(),
        current_file: String::new(),
        progress_percent: 0.0,
        message: "下载已取消".to_string(),
    };
    let _ = app.emit("model-download-progress", progress);

    let project_root = crate::get_project_root().unwrap_or_default();
    let progress_file = project_root.join("models").join(format!(
        ".download_progress_{}.json",
        model_id.replace('/', "_")
    ));
    let _ = fs::remove_file(&progress_file);

    Ok(())
}

#[tauri::command]
pub async fn deploy_model(
    app: tauri::AppHandle,
    model_path: String,
    model_name: String,
    _gguf_file: String,
    config: DeployConfig,
) -> Result<u16, String> {
    let project_root = crate::get_project_root()?;
    let model_dir = project_root.join(&model_path);
    if !model_dir.exists() {
        return Err(format!("模型目录不存在: {}", model_path));
    }

    let ollama_model_name = model_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    if ollama_model_name.is_empty() {
        return Err("模型名称无效".to_string());
    }

    let status = ServerStatus {
        model_id: model_path.clone(),
        status: "importing".to_string(),
        port: 11434,
        message: "正在导入模型到 Ollama...".to_string(),
    };
    let _ = app.emit("model-server-status", status);

    let mut file_digests: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut total_files = 0u32;
    let mut uploaded_files = 0u32;

    for entry in WalkDir::new(&model_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with('.') {
            continue;
        }
        total_files += 1;
    }

    for entry in WalkDir::new(&model_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with('.') {
            continue;
        }

        uploaded_files += 1;
        let progress = ServerStatus {
            model_id: model_path.clone(),
            status: "importing".to_string(),
            port: 11434,
            message: format!(
                "正在上传文件 ({}/{}) {}",
                uploaded_files, total_files, file_name
            ),
        };
        let _ = app.emit("model-server-status", progress);

        match upload_blob(entry.path()).await {
            Ok(digest) => {
                file_digests.insert(file_name, digest);
            }
            Err(e) => {
                let status = ServerStatus {
                    model_id: model_path.clone(),
                    status: "error".to_string(),
                    port: 0,
                    message: format!("上传文件失败: {}", e),
                };
                let _ = app.emit("model-server-status", status);
                return Err(e);
            }
        }
    }

    let create_progress = ServerStatus {
        model_id: model_path.clone(),
        status: "importing".to_string(),
        port: 11434,
        message: "正在创建 Ollama 模型 (可能需要几分钟)...".to_string(),
    };
    let _ = app.emit("model-server-status", create_progress);

    let mut create_body = serde_json::json!({
        "model": ollama_model_name,
        "files": file_digests,
        "stream": false,
    });

    let mut params = serde_json::Map::new();
    let ctx_size = if config.ctx_size > 0 { config.ctx_size } else { 2048 };
    let threads = if config.threads > 0 { config.threads } else { 4 };
    params.insert("num_ctx".to_string(), serde_json::json!(ctx_size));
    params.insert("num_thread".to_string(), serde_json::json!(threads));
    if config.gpu_layers > 0 {
        params.insert("num_gpu".to_string(), serde_json::json!(config.gpu_layers));
    }
    create_body["parameters"] = serde_json::json!(params);

    match ollama_api_post_json("/api/create", create_body).await {
        Ok(()) => {}
        Err(e) => {
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: format!("创建模型失败: {}", e),
            };
            let _ = app.emit("model-server-status", status);
            return Err(format!("创建 Ollama 模型失败: {}", e));
        }
    }

    println!(
        "[deploy_model] Ollama model '{}' created, running test...",
        ollama_model_name
    );

    let run_progress = ServerStatus {
        model_id: model_path.clone(),
        status: "starting".to_string(),
        port: 11434,
        message: "正在加载模型...".to_string(),
    };
    let _ = app.emit("model-server-status", run_progress);

    let load_body = serde_json::json!({
        "model": ollama_model_name,
        "prompt": "",
        "keep_alive": "10m",
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    match client
        .post(format!("{}/api/generate", OLLAMA_BASE_URL))
        .json(&load_body)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp
                .json()
                .await
                .unwrap_or(serde_json::json!({"done": false}));
            let done = body.get("done").and_then(|d| d.as_bool()).unwrap_or(false);
            if done {
                let status = ServerStatus {
                    model_id: model_path.clone(),
                    status: "running".to_string(),
                    port: 11434,
                    message: "模型已加载并运行".to_string(),
                };
                let _ = app.emit("model-server-status", status);
                println!(
                    "[deploy_model] Model '{}' loaded successfully",
                    ollama_model_name
                );
            } else {
                let status = ServerStatus {
                    model_id: model_path.clone(),
                    status: "running".to_string(),
                    port: 11434,
                    message: "模型已启动".to_string(),
                };
                let _ = app.emit("model-server-status", status);
            }
        }
        Ok(resp) => {
            let status_text = resp.status().to_string();
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: format!("加载模型失败: HTTP {}", status_text),
            };
            let _ = app.emit("model-server-status", status);
            return Err(format!("加载模型失败: HTTP {}", status_text));
        }
        Err(e) => {
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: format!("加载模型失败: {}", e),
            };
            let _ = app.emit("model-server-status", status);
            return Err(format!("加载模型失败: {}", e));
        }
    }

    Ok(11434)
}

#[tauri::command]
pub async fn stop_model(app: tauri::AppHandle, model_path: String) -> Result<(), String> {
    let model_name = model_path
        .split('/')
        .last()
        .unwrap_or(&model_path)
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    let unload_body = serde_json::json!({
        "model": model_name,
        "keep_alive": "0",
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .post(format!("{}/api/generate", OLLAMA_BASE_URL))
        .json(&unload_body)
        .send()
        .await;

    let _ = resp;

    let status = ServerStatus {
        model_id: model_path,
        status: "stopped".to_string(),
        port: 0,
        message: "模型已卸载".to_string(),
    };
    let _ = app.emit("model-server-status", status);

    Ok(())
}

#[tauri::command]
pub async fn delete_model(_app: tauri::AppHandle, local_dir: String) -> Result<(), String> {
    let model_name = local_dir
        .split('/')
        .last()
        .unwrap_or(&local_dir)
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .delete(format!("{}/api/delete", OLLAMA_BASE_URL))
        .json(&serde_json::json!({ "model": model_name }))
        .send()
        .await;

    if let Ok(r) = resp {
        if r.status().is_success() {
            println!("[delete_model] Deleted Ollama model: {}", model_name);
        } else {
            println!(
                "[delete_model] Failed to delete Ollama model: HTTP {}",
                r.status()
            );
        }
    }

    let project_root = crate::get_project_root()?;
    let abs_dir = project_root.join(&local_dir);

    if abs_dir.exists() {
        std::fs::remove_dir_all(&abs_dir)
            .map_err(|e| format!("删除模型目录失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_model_info(local_dir: String) -> Result<serde_json::Value, String> {
    let project_root = crate::get_project_root()?;
    let dir = project_root.join(&local_dir);
    if !dir.exists() {
        return Err(format!("目录不存在: {}", local_dir));
    }

    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;
    let mut gguf_files: Vec<String> = Vec::new();
    let mut safetensors_files: Vec<String> = Vec::new();

    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            file_count += 1;
            if let Some(ext) = entry.path().extension() {
                if ext.eq_ignore_ascii_case("gguf") {
                    if let Some(name) = entry.file_name().to_str() {
                        gguf_files.push(name.to_string());
                    }
                } else if ext.eq_ignore_ascii_case("safetensors") {
                    if let Some(name) = entry.file_name().to_str() {
                        safetensors_files.push(name.to_string());
                    }
                }
            }
        }
    }

    let has_model_files = !gguf_files.is_empty() || !safetensors_files.is_empty();
    let model_format = if !gguf_files.is_empty() {
        "gguf"
    } else if !safetensors_files.is_empty() {
        "safetensors"
    } else {
        "unknown"
    };

    let result = serde_json::json!({
        "size_bytes": total_size,
        "gguf_files": gguf_files,
        "safetensors_files": safetensors_files,
        "file_count": file_count,
        "has_model_files": has_model_files,
        "model_format": model_format,
    });

    Ok(result)
}

#[tauri::command]
pub async fn install_ollama(app: tauri::AppHandle) -> Result<String, String> {
    let progress = InstallProgress {
        status: "installing".to_string(),
        message: "正在下载 Ollama 安装包...".to_string(),
    };
    let _ = app.emit("ollama-install-progress", progress.clone());

    let download_url = "https://ollama.com/download/OllamaSetup.exe";
    let data_dir = crate::get_data_dir(&app).map_err(|e| format!("无法获取数据目录: {}", e))?;
    let ollama_dir = data_dir.join("ollama");
    if !ollama_dir.exists() {
        let _ = fs::create_dir_all(&ollama_dir);
    }
    let installer_path = ollama_dir.join("OllamaSetup.exe");

    let app_clone = app.clone();
    let installer_path_clone = installer_path.clone();

    let mut child = Command::new("cmd");
    child.args([
        "/C",
        "curl",
        "-L",
        "-o",
        &installer_path.to_string_lossy(),
        download_url,
    ])
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    child.creation_flags(CREATE_NO_WINDOW);
    let mut child = child
        .spawn()
        .map_err(|e| format!("启动下载进程失败: {}", e))?;

    tokio::spawn(async move {
        let result = child.wait().await;
        match result {
            Ok(exit_status) if exit_status.success() => {
                if installer_path_clone.exists() {
                    let progress = InstallProgress {
                        status: "completed".to_string(),
                        message: "Ollama 安装包已下载，正在启动安装程序...".to_string(),
                    };
                    let _ = app_clone.emit("ollama-install-progress", progress);

                    let mut install_cmd = std::process::Command::new(
                        installer_path_clone.to_string_lossy().as_ref(),
                    );
                    #[cfg(target_os = "windows")]
                    install_cmd.creation_flags(CREATE_NO_WINDOW);
                    let _ = install_cmd.spawn();

                    let progress = InstallProgress {
                        status: "installing".to_string(),
                        message: "请在弹出的安装窗口中完成 Ollama 安装，安装完成后重启应用".to_string(),
                    };
                    let _ = app_clone.emit("ollama-install-progress", progress);
                } else {
                    let progress = InstallProgress {
                        status: "error".to_string(),
                        message: "下载的安装包不存在".to_string(),
                    };
                    let _ = app_clone.emit("ollama-install-progress", progress);
                }
            }
            Ok(exit_status) => {
                let progress = InstallProgress {
                    status: "error".to_string(),
                    message: format!("下载失败，退出码: {}", exit_status),
                };
                let _ = app_clone.emit("ollama-install-progress", progress);
            }
            Err(e) => {
                let progress = InstallProgress {
                    status: "error".to_string(),
                    message: format!("下载进程异常: {}", e),
                };
                let _ = app_clone.emit("ollama-install-progress", progress);
            }
        }
    });

    Ok("下载已启动".to_string())
}

#[tauri::command]
pub async fn get_ollama_models() -> Result<serde_json::Value, String> {
    ollama_api_get("/api/tags").await
}
