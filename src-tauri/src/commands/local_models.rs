use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};

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

static MODEL_SERVER: std::sync::OnceLock<Arc<Mutex<Option<ModelServer>>>> =
    std::sync::OnceLock::new();

fn get_model_server() -> Arc<Mutex<Option<ModelServer>>> {
    MODEL_SERVER
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    #[serde(rename = "openvino")]
    OpenVINO,
    #[serde(rename = "llama-cpp")]
    LlamaCpp,
    #[serde(rename = "tensorrt-llm")]
    TensorRTLLM,
    #[serde(rename = "transformers")]
    Transformers,
}

struct ModelServer {
    child: Child,
    port: u16,
    #[allow(dead_code)]
    backend: Backend,
}

impl Drop for ModelServer {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

fn get_models_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    if let Some(ref dir) = crate::get_configured_models_dir(app) {
        let models_dir = std::path::PathBuf::from(dir);
        if !models_dir.exists() {
            fs::create_dir_all(&models_dir)
                .map_err(|e| format!("无法创建模型目录: {}", e))?;
        }
        return Ok(models_dir);
    }
    let data_dir = crate::get_data_dir(app)?;
    let models_dir = data_dir.join("models");
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| format!("无法创建模型目录: {}", e))?;
    }
    Ok(models_dir)
}

fn find_script(app: &tauri::AppHandle, script_name: &str) -> Result<std::path::PathBuf, String> {
    if let Ok(script) = app.path().resolve(
        std::path::Path::new("../scripts").join(script_name),
        tauri::path::BaseDirectory::Resource,
    ) {
        if script.exists() {
            eprintln!("[Scripts] Found {} via resolve: {:?}", script_name, script);
            return Ok(script);
        }
        eprintln!("[Scripts] resolve path not found at: {:?}", script);
    } else {
        eprintln!("[Scripts] resolve() failed for {}", script_name);
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        let script = resource_dir.join("scripts").join(script_name);
        if script.exists() {
            eprintln!("[Scripts] Found {} via resource_dir/scripts: {:?}", script_name, script);
            return Ok(script);
        }
        eprintln!("[Scripts] resource_dir/scripts not found at: {:?}", script);
        let script_up = resource_dir.join("_up_").join("scripts").join(script_name);
        if script_up.exists() {
            eprintln!("[Scripts] Found {} via resource_dir/_up_/scripts: {:?}", script_name, script_up);
            return Ok(script_up);
        }
        eprintln!("[Scripts] resource_dir/_up_/scripts not found at: {:?}", script_up);
    } else {
        eprintln!("[Scripts] Failed to get resource_dir");
    }

    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("无法确定 exe 路径: {}", e))?
        .parent()
        .ok_or("无法确定 exe 目录")?
        .to_path_buf();

    let script = exe_dir.join("scripts").join(script_name);
    if script.exists() {
        eprintln!("[Scripts] Found {} via exe_dir/scripts: {:?}", script_name, script);
        return Ok(script);
    }
    eprintln!("[Scripts] exe_dir/scripts not found at: {:?}", script);

    let dev_fallback = exe_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("scripts").join(script_name));
    if let Some(ref fallback) = dev_fallback {
        if fallback.exists() {
            eprintln!("[Scripts] Found {} via dev_fallback: {:?}", script_name, fallback);
            return Ok(fallback.clone());
        }
        eprintln!("[Scripts] dev_fallback not found at: {:?}", fallback);
    }

    if let Ok(project_root) = crate::get_project_root() {
        let script = project_root.join("scripts").join(script_name);
        if script.exists() {
            eprintln!("[Scripts] Found {} via project_root: {:?}", script_name, script);
            return Ok(script);
        }
        eprintln!("[Scripts] project_root not found at: {:?}", script);
    }

    Err(format!("脚本不存在: {}", script_name))
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
    pub gpus: Vec<GpuInfo>,
    pub llama_cpp: bool,
    pub oneapi: bool,
    pub transformers: bool,
    pub msvc: bool,
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
pub struct DeployLog {
    pub model_id: String,
    pub line: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeployConfig {
    pub ctx_size: u32,
    pub threads: u32,
    pub device: String,
    pub port: u16,
    #[serde(default = "default_backend")]
    pub backend: Backend,
}

fn default_backend() -> Backend {
    Backend::LlamaCpp
}

fn spawn_log_reader(app: tauri::AppHandle, model_id: String, child: &mut Child) {
    if let Some(stdout) = child.stdout.take() {
        let app_clone = app.clone();
        let mid = model_id.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        let _ = app_clone.emit("model-deploy-log", DeployLog {
                            model_id: mid.clone(),
                            line,
                            source: "stdout".to_string(),
                        });
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let app_clone = app.clone();
        let mid = model_id.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        let _ = app_clone.emit("model-deploy-log", DeployLog {
                            model_id: mid.clone(),
                            line,
                            source: "stderr".to_string(),
                        });
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });
    }
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

async fn start_model_server(
    app: &tauri::AppHandle,
    backend: &Backend,
    port: u16,
    model_id: &str,
) -> Result<(), String> {
    let server_state = get_model_server();
    {
        let guard = server_state.lock().await;
        if let Some(ref server) = *guard {
            if let Ok(client) = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
            {
                let endpoints = ["/health", "/v1/models"];
                for ep in &endpoints {
                    let url = format!("http://127.0.0.1:{}{}", server.port, ep);
                    if let Ok(resp) = client.get(&url).send().await {
                        if resp.status().is_success() {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    let mut cmd = match backend {
        Backend::OpenVINO => {
            let script_path = find_script(app, "openvino_server.py")?;
            let port_str = port.to_string();
            let script_str = script_path.to_string_lossy().to_string();
            let mut c = Command::new("python");
            c.args([&script_str, "--port", &port_str]);
            c
        }
        Backend::LlamaCpp => {
            let port_str = port.to_string();
            let mut c = Command::new("python");
            c.args([
                "-m",
                "llama_cpp.server",
                "--port",
                &port_str,
                "--host",
                "127.0.0.1",
            ]);
            c
        }
        Backend::TensorRTLLM => {
            return Err("TensorRT-LLM 后端尚未实现".to_string());
        }
        Backend::Transformers => {
            let script_path = find_script(app, "transformers_server.py")?;
            let port_str = port.to_string();
            let script_str = script_path.to_string_lossy().to_string();
            let mut c = Command::new("python");
            c.args([&script_str, "--port", &port_str]);
            c
        }
    };

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动服务器失败: {}", e))?;

    spawn_log_reader(app.clone(), model_id.to_string(), &mut child);

    {
        let mut guard = server_state.lock().await;
        *guard = Some(ModelServer {
            child,
            port,
            backend: backend.clone(),
        });
    }

    let health_endpoints = ["/health", "/v1/models"];
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut retries = 0;
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
        let mut healthy = false;
        for ep in &health_endpoints {
            let url = format!("http://127.0.0.1:{}{}", port, ep);
            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    healthy = true;
                    break;
                }
            }
        }
        if healthy {
            break;
        }
        if retries > 30 {
            return Err("服务器启动超时 (15秒)".to_string());
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn stop_model_server() -> Result<(), String> {
    let server_state = get_model_server();
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

    let gpus = detect_gpus().await;

    let llama_cpp_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "python", "-c", "import importlib.util; print(importlib.util.find_spec('llama_cpp') is not None)"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    let llama_cpp = match llama_cpp_check {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    };

    let oneapi = std::path::Path::new(r"C:\Program Files (x86)\Intel\oneAPI\setvars.bat").exists();

    let transformers_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "python", "-c", "import importlib.util; print(importlib.util.find_spec('transformers') is not None)"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    let transformers = match transformers_check {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    };

    let msvc = detect_msvc();

    Ok(EnvironmentStatus {
        python,
        python_version,
        modelscope,
        openvino,
        openvino_version,
        openvino_genai,
        optimum,
        gpus,
        llama_cpp,
        oneapi,
        transformers,
        msvc,
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    pub vendor: String,
    pub name: String,
    /// GPU类型：`"discrete"` 独立显卡，`"integrated"` 集成显卡，`"unknown"` 未知类型
    pub gpu_type: String,
}

async fn detect_gpus() -> Vec<GpuInfo> {
    #[cfg(target_os = "windows")]
    {
        let mut gpus = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        detect_gpus_via_nvidia_smi(&mut gpus, &mut seen_names).await;
        detect_gpus_via_powershell(&mut gpus, &mut seen_names).await;

        gpus
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "windows")]
async fn detect_gpus_via_nvidia_smi(
    gpus: &mut Vec<GpuInfo>,
    seen_names: &mut std::collections::HashSet<String>,
) {
    let nvidia_smi_check = timeout(Duration::from_secs(10), async {
        let mut cmd = Command::new("cmd");
        cmd.args([
            "/C",
            "nvidia-smi",
            "--query-gpu=name",
            "--format=csv,noheader",
        ]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    if let Ok(Ok(output)) = nvidia_smi_check {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().trim_matches('"').trim();
                if name.is_empty() || name.starts_with("No devices") {
                    continue;
                }
                let key = name.to_lowercase();
                if seen_names.insert(key) {
                    gpus.push(GpuInfo {
                        vendor: "NVIDIA".to_string(),
                        name: name.to_string(),
                        gpu_type: "discrete".to_string(),
                    });
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
async fn detect_gpus_via_powershell(
    gpus: &mut Vec<GpuInfo>,
    seen_names: &mut std::collections::HashSet<String>,
) {
    let ps_check = timeout(Duration::from_secs(15), async {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-CimInstance -ClassName Win32_VideoController | Select-Object -ExpandProperty Caption",
        ]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().await
    })
    .await;

    if let Ok(Ok(output)) = ps_check {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            let name_lower = name.to_lowercase();

            if name_lower.contains("virtual")
                || name_lower.contains("remote")
                || name_lower.contains("software")
                || name_lower.contains("basic render")
                || name_lower.contains("basic display")
                || name_lower.contains("microsoft basic")
                || name_lower.contains("rdp")
                || name_lower.contains("mirage")
                || name_lower.contains("teamviewer")
            {
                continue;
            }

            if seen_names.contains(&name_lower) {
                continue;
            }

            let vendor = if name_lower.contains("nvidia") || name_lower.contains("geforce") || name_lower.contains("rtx") || name_lower.contains("gtx") {
                "NVIDIA".to_string()
            } else if name_lower.contains("amd") || name_lower.contains("radeon") {
                "AMD".to_string()
            } else if name_lower.contains("intel") {
                "Intel".to_string()
            } else {
                "Unknown".to_string()
            };

            let gpu_type = if vendor == "NVIDIA" || vendor == "AMD" {
                "discrete".to_string()
            } else if vendor == "Intel" {
                let is_intel_discrete = name_lower.contains("arc")
                    || name_lower.contains("a380")
                    || name_lower.contains("a580")
                    || name_lower.contains("a770")
                    || name_lower.contains("a770m")
                    || name_lower.contains("a750")
                    || name_lower.contains("a310");
                if is_intel_discrete {
                    "discrete".to_string()
                } else {
                    "integrated".to_string()
                }
            } else {
                "unknown".to_string()
            };

            seen_names.insert(name_lower);
            gpus.push(GpuInfo {
                vendor,
                name: name.to_string(),
                gpu_type,
            });
        }
    }
}

async fn check_intel_gpu() -> bool {
    let gpus = detect_gpus().await;
    gpus.iter().any(|g| g.vendor == "Intel" && g.gpu_type == "discrete")
}

fn detect_msvc() -> bool {
    let editions = ["BuildTools", "Community", "Professional", "Enterprise"];
    for edition in &editions {
        let path = std::path::PathBuf::from(format!(
            r"C:\Program Files\Microsoft Visual Studio\2022\{}\VC\Tools\MSVC",
            edition
        ));
        if path.exists() {
            return true;
        }
    }
    for edition in &editions {
        let path = std::path::PathBuf::from(format!(
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\{}\VC\Tools\MSVC",
            edition
        ));
        if path.exists() {
            return true;
        }
    }
    false
}

fn decode_cmd_output(raw: &[u8]) -> String {
    if raw.is_empty() {
        return String::new();
    }
    if let Ok(s) = String::from_utf8(raw.to_vec()) {
        return s;
    }
    let (cow, _encoding, _had_errors) = encoding_rs::GBK.decode(raw);
    cow.into_owned()
}

fn interpret_pip_error(stderr: &str, package: &str) -> String {
    let lower = stderr.to_lowercase();
    if lower.contains("'pip'") && (lower.contains("not recognized") || lower.contains("不是内部或外部命令") || lower.contains("找不到") || lower.contains("不是可运行的")) {
        return "找不到 pip 命令。请确认 Python 已正确安装并勾选了 'Add Python to PATH' 选项，然后重启电脑再试。\n也可以在命令行中运行 'python -m ensurepip' 来安装 pip。".to_string();
    }
    if lower.contains("'python'") && (lower.contains("not recognized") || lower.contains("不是内部或外部命令") || lower.contains("找不到") || lower.contains("不是可运行的")) {
        return "找不到 Python 命令。请先安装 Python（推荐 3.10+），安装时务必勾选 'Add Python to PATH'，然后重启电脑再试。\n下载地址: https://www.python.org/downloads/".to_string();
    }
    if lower.contains("no module named pip") {
        return "Python 环境中缺少 pip 模块。请在命令行中运行 'python -m ensurepip' 来安装 pip，然后重试。".to_string();
    }
    if lower.contains("permissionerror") || lower.contains("permission denied") || lower.contains("访问被拒绝") {
        return format!("权限不足，无法安装 {}。请尝试以管理员身份运行本应用，或使用 'python -m pip install {} --user' 进行用户级安装。", package, package);
    }
    if lower.contains("network") || lower.contains("timeout") || lower.contains("connectionerror") || lower.contains("连接超时") || lower.contains("网络") {
        return format!("网络连接失败，无法下载 {}。请检查网络连接后重试。如果使用代理，请确保代理配置正确。", package);
    }
    if lower.contains("nomatch") || lower.contains("no matching distribution") || lower.contains("找不到") && lower.contains("distribution") {
        return format!("找不到 {} 的可用安装包。可能是 Python 版本不兼容，请确认使用 Python 3.8 或更高版本。", package);
    }
    if !stderr.trim().is_empty() {
        let trimmed = stderr.trim();
        if trimmed.len() > 300 {
            return format!("安装失败，错误信息:\n...{}", &trimmed[trimmed.len() - 300..]);
        }
        return format!("安装失败，错误信息:\n{}", trimmed);
    }
    format!("{} 安装失败，未知错误。请检查 Python 环境是否正常。", package)
}

#[tauri::command]
pub async fn install_dependency(
    app: tauri::AppHandle,
    package: String,
) -> Result<(), String> {
    if package != "oneapi" {
        let python_check = Command::new("cmd")
            .args(["/C", "python", "--version"])
            .output()
            .await;
        match python_check {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let stderr = decode_cmd_output(&output.stderr);
                let msg = if stderr.trim().is_empty() {
                    "Python 环境异常，命令执行失败。请确认 Python 已正确安装并勾选了 'Add Python to PATH' 选项，然后重启电脑再试。\n下载地址: https://www.python.org/downloads/".to_string()
                } else {
                    interpret_pip_error(&stderr, &package)
                };
                let progress = InstallProgress {
                    status: "error".to_string(),
                    message: msg.clone(),
                };
                let _ = app.emit("dependency-install-progress", progress);
                return Err(msg);
            }
            Err(_) => {
                let msg = "找不到 Python 命令。请先安装 Python（推荐 3.10+），安装时务必勾选 'Add Python to PATH'，然后重启电脑再试。\n下载地址: https://www.python.org/downloads/".to_string();
                let progress = InstallProgress {
                    status: "error".to_string(),
                    message: msg.clone(),
                };
                let _ = app.emit("dependency-install-progress", progress);
                return Err(msg);
            }
        }
    }

    let progress = InstallProgress {
        status: "installing".to_string(),
        message: format!("正在安装 {}...", package),
    };
    let _ = app.emit("dependency-install-progress", progress);

    let mut cmd = Command::new("cmd");

    if package == "msvc" {
        let progress = InstallProgress {
            status: "installing".to_string(),
            message: "正在打开 Visual Studio Build Tools 下载页面...".to_string(),
        };
        let _ = app.emit("dependency-install-progress", progress);
        let mut open_cmd = Command::new("cmd");
        open_cmd.args([
            "/C",
            "start",
            "https://visualstudio.microsoft.com/zh-hans/visual-cpp-build-tools/",
        ]);
        #[cfg(target_os = "windows")]
        open_cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = open_cmd.spawn();
        let progress2 = InstallProgress {
            status: "completed".to_string(),
            message: "已在浏览器中打开 Build Tools 下载页面。请下载并运行安装器，安装时勾选「使用 C++ 的桌面开发」工作负载，安装完成后重新检测环境。".to_string(),
        };
        let _ = app.emit("dependency-install-progress", progress2);
        return Ok(());
    } else if package == "oneapi" {
        let progress = InstallProgress {
            status: "installing".to_string(),
            message: "正在打开 Intel oneAPI 下载页面...".to_string(),
        };
        let _ = app.emit("dependency-install-progress", progress);

        #[cfg(target_os = "windows")]
        {
            let mut open_cmd = Command::new("cmd");
            open_cmd.args([
                "/C",
                "start",
                "https://www.intel.com/content/www/us/en/developer/tools/oneapi/base-toolkit-download.html",
            ]);
            #[cfg(target_os = "windows")]
            open_cmd.creation_flags(CREATE_NO_WINDOW);
            let _ = open_cmd.spawn();
        }

        let progress2 = InstallProgress {
            status: "completed".to_string(),
            message: "已在浏览器中打开 oneAPI 下载页面。安装时只需勾选以下组件即可：\n1. Intel oneAPI DPC++/C++ Compiler\n2. Intel oneAPI Math Kernel Library\n3. Intel oneAPI Threading Building Blocks\n安装完成后请重新检测环境。".to_string(),
        };
        let _ = app.emit("dependency-install-progress", progress2);
        return Ok(());
    } else if package == "llama-cpp-python" {
        let has_intel_gpu = check_intel_gpu().await;
        let has_oneapi = std::path::Path::new(r"C:\Program Files (x86)\Intel\oneAPI\setvars.bat").exists();
        if has_intel_gpu && has_oneapi {
            let progress = InstallProgress {
                status: "installing".to_string(),
                message: "正在下载 llama-cpp-python (Intel GPU/SYCL 预编译包)...".to_string(),
            };
            let _ = app.emit("dependency-install-progress", progress);
            let py_ver_output = Command::new("cmd")
                .args(["/C", "python", "-c", "import sys; print(f'{sys.version_info.major}{sys.version_info.minor}')"])
                .output()
                .await;
            let py_tag = match py_ver_output {
                Ok(o) if o.status.success() => {
                    String::from_utf8_lossy(&o.stdout).trim().to_string()
                }
                _ => "313".to_string(),
            };
            let wheel_dir = std::env::temp_dir().join("aurorafairy_llamacpp");
            let _ = fs::create_dir_all(&wheel_dir);
            let download_filename = format!(
                "llama_cpp_python-0.3.36+sycl-cp{}-cp{}-win_amd64.whl",
                py_tag, py_tag
            );
            let download_file = wheel_dir.join(&download_filename);
            let install_filename = format!(
                "llama_cpp_python-0.3.36-cp{}-cp{}-win_amd64.whl",
                py_tag, py_tag
            );
            let install_file = wheel_dir.join(&install_filename);
            if !download_file.exists() && !install_file.exists() {
                let mut curl_cmd = Command::new("cmd");
                curl_cmd.args([
                    "/C",
                    "curl.exe",
                    "-L",
                    "-o",
                    &download_file.to_string_lossy(),
                    &format!(
                        "https://github.com/allanmeng/llama-cpp-python-sycl-windows/releases/latest/download/{}",
                        download_filename
                    ),
                ]);
                #[cfg(target_os = "windows")]
                curl_cmd.creation_flags(CREATE_NO_WINDOW);
                let curl_result = curl_cmd.output().await;
                match curl_result {
                    Ok(o) if o.status.success() => {}
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                        return Err(format!("下载 llama-cpp-python SYCL 失败: {}", stderr));
                    }
                    Err(e) => {
                        return Err(format!("下载 llama-cpp-python SYCL 失败: {}", e));
                    }
                }
                let _ = fs::copy(&download_file, &install_file);
            }
            let progress2 = InstallProgress {
                status: "installing".to_string(),
                message: "正在安装 llama-cpp-python (SYCL)...".to_string(),
            };
            let _ = app.emit("dependency-install-progress", progress2);
            let _ = Command::new("cmd")
                .args(["/C", "python", "-m", "pip", "uninstall", "llama-cpp-python", "-y"])
                .output()
                .await;
            cmd.args([
                "/C",
                "python",
                "-m",
                "pip",
                "install",
                &install_file.to_string_lossy(),
            ]);
        } else if has_intel_gpu && !has_oneapi {
            return Err("检测到 Intel GPU 但未安装 Intel oneAPI。请先安装 oneAPI（点击上方 oneAPI 的下载按钮），然后再安装 llama.cpp。".to_string());
        } else {
            if !detect_msvc() {
                return Err("编译 llama-cpp-python 需要 Visual Studio Build Tools。\n请先点击环境检测中「MSVC」旁边的安装按钮，安装时勾选「使用 C++ 的桌面开发」工作负载，安装完成后重新检测环境再试。".to_string());
            }
            let progress2 = InstallProgress {
                status: "installing".to_string(),
                message: "正在从源码编译 llama-cpp-python（约需3-5分钟）...".to_string(),
            };
            let _ = app.emit("dependency-install-progress", progress2);
            let _ = Command::new("cmd")
                .args(["/C", "python", "-m", "pip", "uninstall", "llama-cpp-python", "-y"])
                .output()
                .await;
            cmd.env("CMAKE_ARGS", "-DCMAKE_C_FLAGS=/utf-8 -DCMAKE_CXX_FLAGS=/utf-8");
            cmd.args(["/C", "python", "-m", "pip", "install", "llama-cpp-python", "--no-binary", "llama-cpp-python", "--force-reinstall"]);
        }
    } else if package == "transformers" {
        let torch_check = Command::new("cmd")
            .args(["/C", "python", "-c", "import importlib.util; print(importlib.util.find_spec('torch') is not None)"])
            .output()
            .await;
        let has_torch = match torch_check {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout).trim() == "True"
            }
            _ => false,
        };

        if !has_torch {
            let progress = InstallProgress {
                status: "installing".to_string(),
                message: "正在安装 PyTorch (依赖，约 800MB)...".to_string(),
            };
            let _ = app.emit("dependency-install-progress", progress);
            let has_intel_gpu = check_intel_gpu().await;
            let mut torch_cmd = Command::new("cmd");
            if has_intel_gpu {
                torch_cmd.args(["/C", "python", "-m", "pip", "install", "torch", "--index-url", "https://pytorch-extension.intel.com/whl/xpu"]);
            } else {
                torch_cmd.args(["/C", "python", "-m", "pip", "install", "torch"]);
            }
            let _ = torch_cmd.output().await;
        }

        let progress2 = InstallProgress {
            status: "installing".to_string(),
            message: "正在安装 Transformers...".to_string(),
        };
        let _ = app.emit("dependency-install-progress", progress2);
        cmd.args(["/C", "python", "-m", "pip", "install", "transformers", "--upgrade"]);
    } else {
        cmd.args(["/C", "python", "-m", "pip", "install", "--upgrade", &package]);
    }
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let result = cmd.output().await;

    match result {
        Ok(output) if output.status.success() => {
            if package == "llama-cpp-python" {
                let server_deps = ["sse-starlette", "starlette-context", "uvicorn"];
                for dep in &server_deps {
                    let mut dep_cmd = Command::new("cmd");
                    dep_cmd.args(["/C", "python", "-m", "pip", "install", dep]);
                    #[cfg(target_os = "windows")]
                    dep_cmd.creation_flags(CREATE_NO_WINDOW);
                    let _ = dep_cmd.output().await;
                }
            }
            let progress = InstallProgress {
                status: "completed".to_string(),
                message: format!("{} 安装完成", package),
            };
            let _ = app.emit("dependency-install-progress", progress);
            Ok(())
        }
        Ok(output) => {
            let stderr = decode_cmd_output(&output.stderr);
            let msg = interpret_pip_error(&stderr, &package);
            let progress = InstallProgress {
                status: "error".to_string(),
                message: msg.clone(),
            };
            let _ = app.emit("dependency-install-progress", progress);
            Err(msg)
        }
        Err(e) => {
            let msg = format!("安装进程启动失败: {}。请检查 Python 环境是否正常。", e);
            let progress = InstallProgress {
                status: "error".to_string(),
                message: msg.clone(),
            };
            let _ = app.emit("dependency-install-progress", progress);
            Err(msg)
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
    let script_path = find_script(&app, "download_model.py")?;

    let progress_file_str = progress_file_path.to_string_lossy().to_string();

    let script_str = script_path.to_string_lossy().to_string();
    let local_dir_str = abs_local_dir.to_string_lossy().to_string();
    let stderr_file_path = models_dir.join(format!(
        ".download_stderr_{}.log",
        model_id.replace('/', "_")
    ));
    let stderr_file = std::fs::File::create(&stderr_file_path)
        .map_err(|e| format!("无法创建 stderr 日志文件: {}", e))?;
    let stderr_file_str = stderr_file_path.to_string_lossy().to_string();
    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(["/C", "python"]);
    #[cfg(target_os = "windows")]
    cmd.args([&script_str, &model_id, &local_dir_str, &progress_file_str]);
    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new("python");
    #[cfg(not(target_os = "windows"))]
    cmd.args([&script_str, &model_id, &local_dir_str, &progress_file_str]);
    cmd.stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::from(stderr_file));
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
        let mut cancelled = false;
        let mut no_progress_ticks: u32 = 0;

        loop {
            ticker.tick().await;

            let process_exited = {
                let mut map = processes_clone.lock().await;
                if !map.contains_key(&model_id_clone) {
                    cancelled = true;
                    break;
                }
                if let Some(child) = map.get_mut(&model_id_clone) {
                    match child.try_wait() {
                        Ok(Some(_status)) => true,
                        Ok(None) => false,
                        Err(_) => true,
                    }
                } else {
                    true
                }
            };

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
                            let _ = fs::remove_file(&stderr_file_str);
                            break;
                        }
                        last_status = status;
                        no_progress_ticks = 0;
                    }
                }
            } else {
                no_progress_ticks += 1;
            }

            if process_exited && last_status != "completed" && last_status != "error" {
                cleanup_modelscope_lock(&model_id_clone);
                let _ = fs::remove_file(&progress_file_str);
                let stderr_content = fs::read_to_string(&stderr_file_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let _ = fs::remove_file(&stderr_file_str);
                let msg = if stderr_content.is_empty() {
                    "下载进程异常退出，请检查 Python 和 ModelScope 是否已安装".to_string()
                } else {
                    format!("下载进程异常退出: {}", stderr_content)
                };
                let progress = DownloadProgress {
                    model_id: model_id_clone.clone(),
                    status: "error".to_string(),
                    current_file: String::new(),
                    progress_percent: 0.0,
                    message: msg,
                };
                let _ = app_clone.emit("model-download-progress", progress);
                let mut map = processes_clone.lock().await;
                map.remove(&model_id_clone);
                return;
            }

            if no_progress_ticks > 120 {
                cleanup_modelscope_lock(&model_id_clone);
                let _ = fs::remove_file(&progress_file_str);
                let _ = fs::remove_file(&stderr_file_str);
                let progress = DownloadProgress {
                    model_id: model_id_clone.clone(),
                    status: "error".to_string(),
                    current_file: String::new(),
                    progress_percent: 0.0,
                    message: "下载超时，60秒内无进度更新".to_string(),
                };
                let _ = app_clone.emit("model-download-progress", progress);
                let mut map = processes_clone.lock().await;
                if let Some(mut child) = map.remove(&model_id_clone) {
                    let _ = child.kill().await;
                }
                return;
            }
        }

        if cancelled {
            cleanup_modelscope_lock(&model_id_clone);
            let _ = fs::remove_file(&progress_file_str);
            let _ = fs::remove_file(&stderr_file_str);

            let mut map = processes_clone.lock().await;
            if let Some(mut child) = map.remove(&model_id_clone) {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
            drop(map);
            return;
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
        let _ = fs::remove_file(&stderr_file_str);
    });

    Ok(local_dir)
}

#[tauri::command]
pub async fn cancel_download(app: tauri::AppHandle, model_id: String) -> Result<(), String> {
    let processes = get_download_processes();
    let mut map = processes.lock().await;

    if let Some(mut child) = map.remove(&model_id) {
        let pid = child.id();
        let _ = child.kill().await;
        if let Some(pid) = pid {
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("taskkill")
                    .args(["/T", "/F", "/PID", &pid.to_string()])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .await;
            }
        }
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
    let stderr_file = models_dir.join(format!(
        ".download_stderr_{}.log",
        model_id.replace('/', "_")
    ));
    let _ = fs::remove_file(&progress_file);
    let _ = fs::remove_file(&stderr_file);
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

    let mut has_config = false;
    let mut has_gguf = false;
    for entry in WalkDir::new(&abs_model_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if name == "config.json" {
                    has_config = true;
                }
                if name.to_lowercase().ends_with(".gguf") {
                    has_gguf = true;
                }
            }
        }
    }
    if !has_config {
        if has_gguf {
            return Err("GGUF 格式的模型无法转换为 OpenVINO IR。GGUF 是 llama.cpp 专用格式，请使用 llama.cpp 后端部署。如需使用 OpenVINO，请下载 HuggingFace 原始格式（safetensors）的模型。".to_string());
        } else {
            return Err("模型目录中未找到 config.json，无法进行 OpenVINO 转换。请确保模型是 HuggingFace 原始格式（safetensors）。".to_string());
        }
    }

    let ir_model_name = model_path.split('/').last().unwrap_or(&model_path);
    let ir_output_dir = format!("{}_ov", ir_model_name);
    let abs_ir_dir = models_dir.join(&ir_output_dir);

    let ov_xml = abs_ir_dir.join("openvino_model.xml");
    let ov_bin = abs_ir_dir.join("openvino_model.bin");
    if ov_xml.exists() && ov_bin.exists() {
        return Ok(ir_output_dir);
    }

    let convert_script = find_script(&app, "convert_model.py")?;

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

    let _ = app.emit("model-deploy-log", DeployLog {
        model_id: model_path.clone(),
        line: "开始转换模型到 OpenVINO IR 格式...".to_string(),
        source: "stdout".to_string(),
    });

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

    let mut child = cmd.spawn().map_err(|e| format!("启动转换脚本失败: {}", e))?;
    spawn_log_reader(app.clone(), model_path.clone(), &mut child);
    let result = child.wait_with_output().await;

    let progress_content = if progress_file.exists() {
        fs::read_to_string(&progress_file).ok()
    } else {
        None
    };
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
            if ov_xml.exists() && ov_bin.exists() {
                let status = ServerStatus {
                    model_id: model_path.clone(),
                    status: "imported".to_string(),
                    port: 0,
                    message: "模型转换完成".to_string(),
                };
                let _ = app.emit("model-server-status", status);
                return Ok(ir_output_dir);
            }
            let error_msg = if let Some(ref content) = progress_content {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(content) {
                    data.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let error_lines: Vec<&str> = stderr.lines().filter(|l| {
                let l = l.trim();
                !l.is_empty()
                    && !l.contains("DeprecationWarning")
                    && !l.contains("UserWarning")
                    && !l.contains("TracerWarning")
                    && !l.starts_with("Loading checkpoint shards")
                    && !l.starts_with("`torch_dtype`")
                    && !l.starts_with("`use_fast`")
                    && !l.starts_with("Using a slow")
                    && !l.starts_with("loss_type=None")
                    && !l.contains("it/s]")
            }).collect();
            let final_error = if !error_msg.is_empty() {
                error_msg
            } else if !error_lines.is_empty() {
                error_lines.last().unwrap_or(&"未知错误").to_string()
            } else {
                format!("模型转换失败 (退出码: {})", output.status)
            };
            let _ = app.emit("model-deploy-log", DeployLog {
                model_id: model_path.clone(),
                line: format!("转换失败: {}", final_error),
                source: "stderr".to_string(),
            });
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: final_error.clone(),
            };
            let _ = app.emit("model-server-status", status);
            Err(final_error)
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
pub async fn uninstall_dependency(app: tauri::AppHandle, package: String) -> Result<(), String> {
    if package == "msvc" || package == "oneapi" {
        return Err(format!("{} 不支持通过命令行卸载，请手动卸载。", package));
    }

    let packages_to_remove: Vec<&str> = match package.as_str() {
        "openvino" => vec!["openvino", "openvino-genai", "optimum"],
        _ => vec![&package],
    };

    let progress = InstallProgress {
        status: "installing".to_string(),
        message: format!("正在卸载 {}...", package),
    };
    let _ = app.emit("dependency-install-progress", progress);

    for pkg in &packages_to_remove {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "python", "-m", "pip", "uninstall", pkg, "-y"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.output().await;
    }

    let progress2 = InstallProgress {
        status: "completed".to_string(),
        message: format!("{} 已卸载", package),
    };
    let _ = app.emit("dependency-install-progress", progress2);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingleDepStatus {
    pub key: String,
    pub installed: bool,
    pub version: Option<String>,
}

#[tauri::command]
pub async fn check_single_dep(package: String) -> Result<SingleDepStatus, String> {
    match package.as_str() {
        "python" => {
            let result = timeout(Duration::from_secs(10), async {
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", "python", "--version"]);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(CREATE_NO_WINDOW);
                cmd.output().await
            })
            .await;
            let installed = match &result {
                Ok(Ok(output)) => output.status.success(),
                _ => false,
            };
            let version = match &result {
                Ok(Ok(output)) => {
                    let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if v.is_empty() {
                        String::from_utf8_lossy(&output.stderr).trim().to_string()
                    } else {
                        v
                    }
                }
                _ => String::new(),
            };
            Ok(SingleDepStatus {
                key: package,
                installed,
                version: if installed { Some(version) } else { None },
            })
        }
        "modelscope" | "llama-cpp-python" | "transformers" => {
            let module = if package == "llama-cpp-python" {
                "llama_cpp"
            } else {
                &package
            };
            let result = timeout(Duration::from_secs(10), async {
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", "python", "-c", &format!("import importlib.util; spec = importlib.util.find_spec('{}'); print(spec is not None)", module)]);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(CREATE_NO_WINDOW);
                cmd.output().await
            })
            .await;
            let installed = match &result {
                Ok(Ok(output)) => {
                    output.status.success()
                        && String::from_utf8_lossy(&output.stdout).trim() == "True"
                }
                _ => false,
            };
            Ok(SingleDepStatus {
                key: package,
                installed,
                version: None,
            })
        }
        "openvino" => {
            let result = timeout(Duration::from_secs(10), async {
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", "python", "-c", "import openvino; print(openvino.__version__)"]);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(CREATE_NO_WINDOW);
                cmd.output().await
            })
            .await;
            let installed = match &result {
                Ok(Ok(output)) => output.status.success(),
                _ => false,
            };
            let version = match &result {
                Ok(Ok(output)) => {
                    let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if v.is_empty() { None } else { Some(v) }
                }
                _ => None,
            };
            Ok(SingleDepStatus {
                key: package,
                installed,
                version,
            })
        }
        "openvino-genai" => {
            let result = timeout(Duration::from_secs(10), async {
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", "python", "-c", "import openvino_genai"]);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(CREATE_NO_WINDOW);
                cmd.output().await
            })
            .await;
            let installed = match &result {
                Ok(Ok(output)) => output.status.success(),
                _ => false,
            };
            Ok(SingleDepStatus {
                key: package,
                installed,
                version: None,
            })
        }
        "optimum" => {
            let result = timeout(Duration::from_secs(10), async {
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", "python", "-c", "import optimum.intel"]);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(CREATE_NO_WINDOW);
                cmd.output().await
            })
            .await;
            let installed = match &result {
                Ok(Ok(output)) => output.status.success(),
                _ => false,
            };
            Ok(SingleDepStatus {
                key: package,
                installed,
                version: None,
            })
        }
        "oneapi" => {
            let installed = std::path::Path::new(r"C:\Program Files (x86)\Intel\oneAPI\setvars.bat").exists();
            Ok(SingleDepStatus {
                key: package,
                installed,
                version: None,
            })
        }
        "msvc" => {
            let installed = detect_msvc();
            Ok(SingleDepStatus {
                key: package,
                installed,
                version: None,
            })
        }
        _ => Err(format!("未知的依赖项: {}", package)),
    }
}

#[tauri::command]
pub async fn deploy_model(
    app: tauri::AppHandle,
    model_path: String,
    model_name: String,
    gguf_file: String,
    config: DeployConfig,
) -> Result<u16, String> {
    let models_dir = get_models_dir(&app)?;
    let port = if config.port > 0 { config.port } else { 8000 };

    match &config.backend {
        Backend::LlamaCpp => {
            let model_name_dir = model_path.split('/').last().unwrap_or(&model_path);
            let abs_model_dir = models_dir.join(model_name_dir);

            let gguf_path = if gguf_file.is_empty() {
                let mut found: Option<std::path::PathBuf> = None;
                if abs_model_dir.exists() {
                    for entry in WalkDir::new(&abs_model_dir).into_iter().filter_map(|e| e.ok()) {
                        if let Some(name) = entry.file_name().to_str() {
                            let name_lower = name.to_lowercase();
                            if name_lower.ends_with(".gguf")
                                && !name_lower.contains("mmproj")
                            {
                                found = Some(entry.path().to_path_buf());
                                break;
                            }
                        }
                    }
                }
                found.ok_or_else(|| "未找到 GGUF 模型文件（已排除 mmproj 等非主模型文件）".to_string())?
            } else {
                abs_model_dir.join(&gguf_file)
            };

            if !gguf_path.exists() {
                return Err(format!("GGUF 文件不存在: {:?}", gguf_path));
            }

            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "starting".to_string(),
                port,
                message: "正在启动 llama.cpp 推理服务器...".to_string(),
            };
            let _ = app.emit("model-server-status", status);

            let server_state = get_model_server();
            {
                let mut guard = server_state.lock().await;
                let port_str = port.to_string();
                let model_str = gguf_path.to_string_lossy().to_string();
                let n_gpu_layers = if config.device == "GPU" { "-1" } else { "0" }.to_string();
                let mut cmd = Command::new("python");
                cmd.args([
                    "-m",
                    "llama_cpp.server",
                    "--model",
                    &model_str,
                    "--port",
                    &port_str,
                    "--host",
                    "127.0.0.1",
                    "--n_gpu_layers",
                    &n_gpu_layers,
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
                #[cfg(target_os = "windows")]
                cmd.creation_flags(CREATE_NO_WINDOW);
                let mut child = cmd
                    .spawn()
                    .map_err(|e| format!("启动 llama.cpp 服务器失败: {}", e))?;
                spawn_log_reader(app.clone(), model_path.clone(), &mut child);
                *guard = Some(ModelServer {
                    child,
                    port,
                    backend: Backend::LlamaCpp,
                });
            }

            let health_endpoints = ["/health", "/v1/models"];
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

            let mut retries = 0;
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                retries += 1;
                let mut healthy = false;
                for ep in &health_endpoints {
                    let url = format!("http://127.0.0.1:{}{}", port, ep);
                    if let Ok(resp) = client.get(&url).send().await {
                        if resp.status().is_success() {
                            healthy = true;
                            break;
                        }
                    }
                }
                if healthy {
                    break;
                }
                if retries > 60 {
                    return Err("llama.cpp 服务器启动超时 (30秒)".to_string());
                }
            }

            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "running".to_string(),
                port,
                message: "模型已加载并运行".to_string(),
            };
            let _ = app.emit("model-server-status", status);

            Ok(port)
        }
        Backend::OpenVINO => {
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

                    if let Err(e) = convert_result {
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

            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "starting".to_string(),
                port,
                message: "正在启动 OpenVINO 推理服务器...".to_string(),
            };
            let _ = app.emit("model-server-status", status);

            start_model_server(&app, &Backend::OpenVINO, port, &model_path).await?;

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
        Backend::TensorRTLLM => {
            Err("TensorRT-LLM 后端尚未实现".to_string())
        }
        Backend::Transformers => {
            let model_name_dir = model_path.split('/').last().unwrap_or(&model_path);
            let abs_model_dir = models_dir.join(model_name_dir);

            if !abs_model_dir.exists() {
                return Err(format!("模型目录不存在: {:?}", abs_model_dir));
            }

            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "starting".to_string(),
                port,
                message: "正在启动 Transformers 推理服务器...".to_string(),
            };
            let _ = app.emit("model-server-status", status);

            start_model_server(&app, &Backend::Transformers, port, &model_path).await?;

            let load_status = ServerStatus {
                model_id: model_path.clone(),
                status: "importing".to_string(),
                port,
                message: "正在加载模型（首次加载可能较慢）...".to_string(),
            };
            let _ = app.emit("model-server-status", load_status);

            let model_path_str = abs_model_dir.to_string_lossy().to_string();
            let load_body = serde_json::json!({
                "model_path": model_path_str,
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
    }
}

#[tauri::command]
pub async fn stop_model(app: tauri::AppHandle, model_path: String) -> Result<(), String> {
    let server_state = get_model_server();
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
