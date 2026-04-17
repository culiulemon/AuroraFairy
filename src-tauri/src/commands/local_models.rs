use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::Emitter;

#[cfg(target_os = "windows")]
#[allow(unused_imports)]
use std::os::windows::process::CommandExt;

use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{interval, timeout, Duration};
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

static DOWNLOAD_PROCESSES: std::sync::OnceLock<Arc<Mutex<HashMap<String, Child>>>> =
    std::sync::OnceLock::new();

fn get_download_processes() -> Arc<Mutex<HashMap<String, Child>>> {
    DOWNLOAD_PROCESSES
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

static OPENVINO_SERVER: std::sync::OnceLock<Arc<Mutex<Option<OpenVinoServer>>>> =
    std::sync::OnceLock::new();

fn get_openvino_server() -> Arc<Mutex<Option<OpenVinoServer>>> {
    OPENVINO_SERVER
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

struct OpenVinoServer {
    child: Child,
    port: u16,
}

impl Drop for OpenVinoServer {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

fn get_models_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = crate::get_data_dir(app)?;
    let models_dir = data_dir.join("models");
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| format!("无法创建模型目录: {}", e))?;
    }
    Ok(models_dir)
}

fn cleanup_modelscope_lock(model_id: &str) {
    let lock_name = model_id.replace('/', "___");
    let lock_dir = dirs::cache_dir()
        .unwrap_or_default()
        .join("modelscope")
        .join("hub")
        .join(".lock");
    let lock_file = lock_dir.join(&lock_name);
    if lock_file.exists() {
        let _ = fs::remove_file(&lock_file);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct InstallProgress {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentStatus {
    pub python: bool,
    pub python_version: Option<String>,
    pub modelscope: bool,
    pub openvino: bool,
    pub openvino_version: Option<String>,
    pub openvino_genai: bool,
    pub optimum: bool,
    pub intel_gpu: bool,
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
    pub device: String,
    pub port: u16,
}

#[allow(dead_code)]
async fn openvino_api_get(port: u16, path: &str) -> Result<serde_json::Value, String> {
    let url = format!("http://127.0.0.1:{}{}", port, path);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("OpenVINO 服务器请求失败: {} (请确认服务器正在运行)", e))?;
    if !resp.status().is_success() {
        return Err(format!("OpenVINO 服务器返回错误: HTTP {}", resp.status()));
    }
    resp.json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))
}

async fn openvino_api_post_json(
    port: u16,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let url = format!("http://127.0.0.1:{}{}", port, path);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("OpenVINO 服务器请求失败: {}", e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("OpenVINO 服务器返回错误: HTTP {} - {}", status, text));
    }
    resp.json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))
}

async fn start_openvino_server(_app: &tauri::AppHandle, port: u16) -> Result<(), String> {
    let server_state = get_openvino_server();
    {
        let guard = server_state.lock().await;
        if let Some(ref server) = *guard {
            let health_url = format!("http://127.0.0.1:{}/health", server.port);
            if let Ok(client) = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
            {
                if let Ok(resp) = client.get(&health_url).send().await {
                    if resp.status().is_success() {
                        return Ok(());
                    }
                }
            }
        }
    }

    let project_root = crate::get_project_root()?;
    let script_path = project_root.join("scripts").join("openvino_server.py");

    if !script_path.exists() {
        return Err(format!("OpenVINO 服务器脚本不存在: {:?}", script_path));
    }

    let port_str = port.to_string();
    let script_str = script_path.to_string_lossy().to_string();
    let mut cmd = Command::new("python");
    cmd.args([&script_str, "--port", &port_str])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let child = cmd
        .spawn()
        .map_err(|e| format!("启动 OpenVINO 服务器失败: {}", e))?;

    {
        let mut guard = server_state.lock().await;
        *guard = Some(OpenVinoServer { child, port });
    }

    let health_url = format!("http://127.0.0.1:{}/health", port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut retries = 0;
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                break;
            }
            _ => {
                if retries > 30 {
                    return Err("OpenVINO 服务器启动超时 (15秒)".to_string());
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn stop_openvino_server() -> Result<(), String> {
    let server_state = get_openvino_server();
    let mut guard = server_state.lock().await;
    *guard = None;
    Ok(())
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

    let openvino_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args([
            "/C",
            "python",
            "-c",
            "import openvino; print(openvino.__version__)",
        ]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    let (openvino, openvino_version) = match openvino_check {
        Ok(Ok(output)) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(stdout))
        }
        _ => (false, None),
    };

    let openvino_genai_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "python", "-c", "import openvino_genai"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    let openvino_genai = match openvino_genai_check {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    };

    let optimum_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "python", "-c", "import optimum.intel"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    let optimum = match optimum_check {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    };

    let intel_gpu = check_intel_gpu().await;

    Ok(EnvironmentStatus {
        python,
        python_version,
        modelscope,
        openvino,
        openvino_version,
        openvino_genai,
        optimum,
        intel_gpu,
    })
}

async fn check_intel_gpu() -> bool {
    #[cfg(target_os = "windows")]
    {
        let gpu_check = timeout(Duration::from_secs(10), async {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", "python", "-c",
                "import openvino as ov; core = ov.Core(); devices = core.available_devices; print('GPU' in devices)"
            ]);
            cmd.creation_flags(CREATE_NO_WINDOW);
            cmd.output().await
        })
        .await;

        match gpu_check {
            Ok(Ok(output)) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                stdout == "True"
            }
            _ => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[tauri::command]
pub async fn install_dependency(
    app: tauri::AppHandle,
    package: String,
) -> Result<(), String> {
    let progress = InstallProgress {
        status: "installing".to_string(),
        message: format!("正在安装 {}...", package),
    };
    let _ = app.emit("dependency-install-progress", progress);

    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "pip", "install", &package]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let result = cmd.output().await;

    match result {
        Ok(output) if output.status.success() => {
            let progress = InstallProgress {
                status: "completed".to_string(),
                message: format!("{} 安装完成", package),
            };
            let _ = app.emit("dependency-install-progress", progress);
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let progress = InstallProgress {
                status: "error".to_string(),
                message: format!("{} 安装失败: {}", package, stderr),
            };
            let _ = app.emit("dependency-install-progress", progress);
            Err(format!("安装失败: {}", stderr))
        }
        Err(e) => {
            let progress = InstallProgress {
                status: "error".to_string(),
                message: format!("安装进程异常: {}", e),
            };
            let _ = app.emit("dependency-install-progress", progress);
            Err(format!("安装进程启动失败: {}", e))
        }
    }
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
    cleanup_modelscope_lock(&model_id);

    let models_dir = get_models_dir(&app)?;
    let progress_file_path = models_dir.join(format!(
        ".download_progress_{}.json",
        model_id.replace('/', "_")
    ));
    if progress_file_path.exists() {
        let _ = fs::remove_file(&progress_file_path);
    }

    let processes = get_download_processes();
    {
        let map = processes.lock().await;
        if map.contains_key(&model_id) {
            return Err(format!("模型 {} 正在下载中", model_id));
        }
    }

    let abs_local_dir = models_dir.join(local_dir.split('/').last().unwrap_or(&local_dir));
    let project_root = crate::get_project_root()?;
    let script_path = project_root.join("scripts").join("download_model.py");

    if !script_path.exists() {
        return Err(format!("下载脚本不存在: {:?}", script_path));
    }

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
    let _local_dir_for_convert = local_dir.clone();

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
                    cleanup_modelscope_lock(&model_id_clone);
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
                    cleanup_modelscope_lock(&model_id_clone);
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
        child
            .kill()
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

    let models_dir = get_models_dir(&app).unwrap_or_else(|_| std::path::PathBuf::from("."));
    let progress_file = models_dir.join(format!(
        ".download_progress_{}.json",
        model_id.replace('/', "_")
    ));
    let _ = fs::remove_file(&progress_file);
    cleanup_modelscope_lock(&model_id);

    Ok(())
}

#[tauri::command]
pub async fn convert_model_to_ir(
    app: tauri::AppHandle,
    model_path: String,
    device: String,
) -> Result<String, String> {
    let models_dir = get_models_dir(&app)?;
    let abs_model_dir = models_dir.join(model_path.split('/').last().unwrap_or(&model_path));

    if !abs_model_dir.exists() {
        return Err(format!("模型目录不存在: {}", model_path));
    }

    let ir_model_name = model_path.split('/').last().unwrap_or(&model_path);
    let ir_output_dir = format!("{}_ov", ir_model_name);
    let abs_ir_dir = models_dir.join(&ir_output_dir);

    let ov_xml = abs_ir_dir.join("openvino_model.xml");
    let ov_bin = abs_ir_dir.join("openvino_model.bin");
    if ov_xml.exists() && ov_bin.exists() {
        return Ok(ir_output_dir);
    }

    let project_root = crate::get_project_root()?;
    let convert_script = project_root.join("scripts").join("convert_model.py");
    if !convert_script.exists() {
        return Err(format!("转换脚本不存在: {:?}", convert_script));
    }

    let progress_file = models_dir.join(format!(
        ".convert_progress_{}.json",
        model_path.replace('/', "_").replace('\\', "_")
    ));

    let status = ServerStatus {
        model_id: model_path.clone(),
        status: "importing".to_string(),
        port: 0,
        message: "正在转换模型到 OpenVINO IR 格式...".to_string(),
    };
    let _ = app.emit("model-server-status", status);

    let script_str = convert_script.to_string_lossy().to_string();
    let model_dir_str = abs_model_dir.to_string_lossy().to_string();
    let output_dir_str = abs_ir_dir.to_string_lossy().to_string();
    let progress_str = progress_file.to_string_lossy().to_string();

    let mut cmd = Command::new("python");
    cmd.args([
        &script_str,
        &model_dir_str,
        &output_dir_str,
        &progress_str,
        &device,
    ])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let result = cmd.output().await;

    let _ = fs::remove_file(&progress_file);

    match result {
        Ok(output) if output.status.success() => {
            if ov_xml.exists() && ov_bin.exists() {
                let status = ServerStatus {
                    model_id: model_path.clone(),
                    status: "imported".to_string(),
                    port: 0,
                    message: "模型转换完成".to_string(),
                };
                let _ = app.emit("model-server-status", status);
                Ok(ir_output_dir)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!("转换完成但输出文件不存在: {}", stderr))
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: format!("模型转换失败"),
            };
            let _ = app.emit("model-server-status", status);
            Err(format!(
                "模型转换失败: {} {}",
                stderr, stdout
            ))
        }
        Err(e) => {
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: format!("转换进程异常: {}", e),
            };
            let _ = app.emit("model-server-status", status);
            Err(format!("转换进程启动失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn deploy_model(
    app: tauri::AppHandle,
    model_path: String,
    model_name: String,
    _gguf_file: String,
    config: DeployConfig,
) -> Result<u16, String> {
    let models_dir = get_models_dir(&app)?;

    let ir_model_name = model_path.split('/').last().unwrap_or(&model_path);
    let ir_path = format!("{}_ov", ir_model_name);
    let abs_ir_dir = models_dir.join(&ir_path);

    if !abs_ir_dir.exists() {
        let ov_xml = abs_ir_dir.join("openvino_model.xml");
        if !ov_xml.exists() {
            let convert_result = convert_model_to_ir(
                app.clone(),
                model_path.clone(),
                config.device.clone(),
            )
            .await;

            match convert_result {
                Ok(_) => {}
                Err(e) => {
                    let status = ServerStatus {
                        model_id: model_path.clone(),
                        status: "error".to_string(),
                        port: 0,
                        message: format!("模型转换失败: {}", e),
                    };
                    let _ = app.emit("model-server-status", status);
                    return Err(e);
                }
            }
        }
    }

    let port = if config.port > 0 { config.port } else { 8000 };

    let status = ServerStatus {
        model_id: model_path.clone(),
        status: "starting".to_string(),
        port,
        message: "正在启动 OpenVINO 推理服务器...".to_string(),
    };
    let _ = app.emit("model-server-status", status);

    start_openvino_server(&app, port).await?;

    let load_status = ServerStatus {
        model_id: model_path.clone(),
        status: "importing".to_string(),
        port,
        message: "正在加载模型...".to_string(),
    };
    let _ = app.emit("model-server-status", load_status);

    let ir_path_str = abs_ir_dir.to_string_lossy().to_string();
    let load_body = serde_json::json!({
        "model_path": ir_path_str,
        "model_name": model_name,
        "device": config.device,
    });

    match openvino_api_post_json(port, "/api/load", load_body).await {
        Ok(_) => {
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "running".to_string(),
                port,
                message: "模型已加载并运行".to_string(),
            };
            let _ = app.emit("model-server-status", status);
        }
        Err(e) => {
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: format!("加载模型失败: {}", e),
            };
            let _ = app.emit("model-server-status", status);
            return Err(e);
        }
    }

    Ok(port)
}

#[tauri::command]
pub async fn stop_model(app: tauri::AppHandle, model_path: String) -> Result<(), String> {
    let server_state = get_openvino_server();
    let port = {
        let guard = server_state.lock().await;
        guard.as_ref().map(|s| s.port).unwrap_or(0)
    };

    if port > 0 {
        let _ = openvino_api_post_json(
            port,
            "/api/unload",
            serde_json::json!({}),
        )
        .await;
    }

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
pub async fn delete_model(app: tauri::AppHandle, local_dir: String) -> Result<(), String> {
    let models_dir = get_models_dir(&app)?;
    let model_name = local_dir.split('/').last().unwrap_or(&local_dir);

    let abs_dir = models_dir.join(model_name);
    if abs_dir.exists() {
        std::fs::remove_dir_all(&abs_dir)
            .map_err(|e| format!("删除模型目录失败: {}", e))?;
    }

    let ir_dir = models_dir.join(format!("{}_ov", model_name));
    if ir_dir.exists() {
        let _ = std::fs::remove_dir_all(&ir_dir);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_model_info(app: tauri::AppHandle, local_dir: String) -> Result<serde_json::Value, String> {
    let models_dir = get_models_dir(&app)?;
    let model_name = local_dir.split('/').last().unwrap_or(&local_dir);
    let dir = models_dir.join(model_name);
    if !dir.exists() {
        return Err(format!("目录不存在: {}", local_dir));
    }

    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;
    let mut gguf_files: Vec<String> = Vec::new();
    let mut safetensors_files: Vec<String> = Vec::new();
    let mut ov_xml_files: Vec<String> = Vec::new();
    let mut ov_bin_files: Vec<String> = Vec::new();

    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            file_count += 1;
            if let Some(ext) = entry.path().extension() {
                let name = entry.file_name().to_string_lossy().to_string();
                if ext.eq_ignore_ascii_case("gguf") {
                    gguf_files.push(name);
                } else if ext.eq_ignore_ascii_case("safetensors") {
                    safetensors_files.push(name);
                } else if ext.eq_ignore_ascii_case("xml") && name.starts_with("openvino_model") {
                    ov_xml_files.push(name);
                } else if ext.eq_ignore_ascii_case("bin") && name.starts_with("openvino_model") {
                    ov_bin_files.push(name);
                }
            }
        }
    }

    let ir_dir = models_dir.join(format!("{}_ov", model_name));
    let mut ir_total_size: u64 = 0;
    let mut ir_file_count: u64 = 0;
    let mut has_ir = false;

    if ir_dir.exists() {
        for entry in WalkDir::new(&ir_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                ir_total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                ir_file_count += 1;
                if let Some(ext) = entry.path().extension() {
                    if ext.eq_ignore_ascii_case("xml") {
                        has_ir = true;
                    }
                }
            }
        }
    }

    let has_model_files = !gguf_files.is_empty()
        || !safetensors_files.is_empty()
        || !ov_xml_files.is_empty();
    let model_format = if !ov_xml_files.is_empty() {
        "openvino_ir"
    } else if !gguf_files.is_empty() {
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
        "ov_xml_files": ov_xml_files,
        "ov_bin_files": ov_bin_files,
        "file_count": file_count,
        "has_model_files": has_model_files,
        "model_format": model_format,
        "ir_converted": has_ir,
        "ir_size_bytes": ir_total_size,
        "ir_file_count": ir_file_count,
    });

    Ok(result)
}
