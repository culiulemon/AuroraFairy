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
    OpenVINO,
    LlamaCpp,
    TensorRTLLM,
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
    #[serde(default = "default_backend")]
    pub backend: Backend,
}

fn default_backend() -> Backend {
    Backend::LlamaCpp
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
    _app: &tauri::AppHandle,
    backend: &Backend,
    port: u16,
) -> Result<(), String> {
    let server_state = get_model_server();
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

    let mut cmd = match backend {
        Backend::OpenVINO => {
            let project_root = crate::get_project_root()?;
            let script_path = project_root.join("scripts").join("openvino_server.py");
            if !script_path.exists() {
                return Err(format!("OpenVINO 服务器脚本不存在: {:?}", script_path));
            }
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
            let project_root = crate::get_project_root()?;
            let script_path = project_root.join("scripts").join("transformers_server.py");
            if !script_path.exists() {
                return Err(format!("Transformers 服务器脚本不存在: {:?}", script_path));
            }
            let port_str = port.to_string();
            let script_str = script_path.to_string_lossy().to_string();
            let mut c = Command::new("python");
            c.args([&script_str, "--port", &port_str]);
            c
        }
    };

    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let child = cmd
        .spawn()
        .map_err(|e| format!("启动服务器失败: {}", e))?;

    {
        let mut guard = server_state.lock().await;
        *guard = Some(ModelServer {
            child,
            port,
            backend: backend.clone(),
        });
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
                    return Err("服务器启动超时 (15秒)".to_string());
                }
            }
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
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    pub vendor: String,
    pub name: String,
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

    if package == "oneapi" {
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
                .args(["/C", "pip", "uninstall", "llama-cpp-python", "-y"])
                .output()
                .await;
            cmd.args([
                "/C",
                "pip",
                "install",
                &install_file.to_string_lossy(),
            ]);
        } else if has_intel_gpu && !has_oneapi {
            return Err("检测到 Intel GPU 但未安装 Intel oneAPI。请先安装 oneAPI（点击上方 oneAPI 的下载按钮），然后再安装 llama.cpp。".to_string());
        } else {
            cmd.args(["/C", "pip", "install", &package]);
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
                torch_cmd.args(["/C", "pip", "install", "torch", "--index-url", "https://pytorch-extension.intel.com/whl/xpu"]);
            } else {
                torch_cmd.args(["/C", "pip", "install", "torch"]);
            }
            let _ = torch_cmd.output().await;
        }

        let progress2 = InstallProgress {
            status: "installing".to_string(),
            message: "正在安装 Transformers...".to_string(),
        };
        let _ = app.emit("dependency-install-progress", progress2);
        cmd.args(["/C", "pip", "install", "transformers"]);
    } else {
        cmd.args(["/C", "pip", "install", &package]);
    }
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
        let mut cancelled = false;

        loop {
            ticker.tick().await;

            {
                let map = processes_clone.lock().await;
                if !map.contains_key(&model_id_clone) {
                    cancelled = true;
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

        if cancelled {
            cleanup_modelscope_lock(&model_id_clone);
            let _ = fs::remove_file(&progress_file_str);

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
            let error_msg = if error_lines.is_empty() {
                format!("模型转换失败 (退出码: {})", output.status)
            } else {
                error_lines.last().unwrap_or(&"未知错误").to_string()
            };
            let status = ServerStatus {
                model_id: model_path.clone(),
                status: "error".to_string(),
                port: 0,
                message: error_msg.clone(),
            };
            let _ = app.emit("model-server-status", status);
            Err(error_msg)
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
                            if name.ends_with(".gguf") {
                                found = Some(entry.path().to_path_buf());
                                break;
                            }
                        }
                    }
                }
                found.ok_or_else(|| "未找到 GGUF 模型文件".to_string())?
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
                    "--n-gpu-layers",
                    &n_gpu_layers,
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
                #[cfg(target_os = "windows")]
                cmd.creation_flags(CREATE_NO_WINDOW);
                let child = cmd
                    .spawn()
                    .map_err(|e| format!("启动 llama.cpp 服务器失败: {}", e))?;
                *guard = Some(ModelServer {
                    child,
                    port,
                    backend: Backend::LlamaCpp,
                });
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
                        if retries > 60 {
                            return Err("llama.cpp 服务器启动超时 (30秒)".to_string());
                        }
                    }
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

            start_model_server(&app, &Backend::OpenVINO, port).await?;

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

            start_model_server(&app, &Backend::Transformers, port).await?;

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
