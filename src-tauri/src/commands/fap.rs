use serde_json::Value;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, ChildStderr, Command};
use tokio::sync::Mutex;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct BridgeManager {
    child: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    stderr: Arc<Mutex<Option<BufReader<ChildStderr>>>>,
}

impl BridgeManager {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            stderr: Arc::new(Mutex::new(None)),
        }
    }
}

fn get_fairy_action_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let binary = resource_dir.join("binaries").join("fairy-action.exe");
        if binary.exists() {
            return Ok(binary);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let binary = resource_dir.join("binaries").join("fairy-action");
            if binary.exists() {
                return Ok(binary);
            }
        }
    }

    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
        .parent()
        .ok_or("无法获取父目录")?
        .to_path_buf();

    let candidates: Vec<PathBuf> = vec![
        exe_dir.join("binaries").join("fairy-action.exe"),
        exe_dir.join("binaries").join("fairy-action"),
        exe_dir.join("fairy-action.exe"),
        exe_dir.join("fairy-action"),
        exe_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("binaries").join("fairy-action.exe"))
            .unwrap_or_default(),
        exe_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("binaries").join("fairy-action"))
            .unwrap_or_default(),
    ];

    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    let mut cmd = std::process::Command::new("fairy-action");
    cmd.arg("--version");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            return Ok(PathBuf::from("fairy-action"));
        }
    }

    let searched = candidates
        .iter()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n  ");
    Err(format!(
        "找不到 fairy-action 二进制文件。已搜索以下路径:\n  {}\n请将其放置在 src-tauri/binaries/ 目录中",
        searched
    ))
}

async fn write_frame(
    stdin: &Arc<Mutex<Option<ChildStdin>>>,
    message: &str,
) -> Result<(), String> {
    let frame = format!("{} {}\n", message.len(), message);
    let mut stdin_guard = stdin.lock().await;
    let stdin_handle = stdin_guard.as_mut().ok_or("bridge stdin 不可用")?;
    stdin_handle
        .write_all(frame.as_bytes())
        .await
        .map_err(|e| format!("写入 bridge stdin 失败: {}", e))?;
    stdin_handle
        .flush()
        .await
        .map_err(|e| format!("刷新 bridge stdin 失败: {}", e))?;
    Ok(())
}

async fn read_frame(
    stdout: &Arc<Mutex<Option<BufReader<ChildStdout>>>>,
) -> Result<String, String> {
    let mut line = String::new();
    let mut stdout_guard = stdout.lock().await;
    let reader = stdout_guard.as_mut().ok_or("bridge stdout 不可用")?;
    for _ in 0..100 {
        line.clear();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("读取 bridge stdout 失败: {}", e))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let space_pos = trimmed.find(' ').ok_or("帧格式错误: 缺少长度前缀")?;
        let message = trimmed[space_pos + 1..].to_string();
        return Ok(message);
    }
    Err("未收到有效帧响应".to_string())
}

async fn fap_bridge_start_inner(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, BridgeManager>,
) -> Result<(), String> {
    let exe_path = get_fairy_action_path(app)?;
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    let fap_install_dir = data_dir.join("fap");

    let mut cmd = Command::new(&exe_path);
    cmd.arg("bridge")
        .arg("--fap-install-dir").arg(&fap_install_dir)
        .arg("--fap-host-data-dir").arg(&data_dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 fairy-action bridge 失败: {}", e))?;

    let stdin_handle = child.stdin.take().ok_or("无法获取 bridge stdin")?;
    let stdout_handle = child.stdout.take().ok_or("无法获取 bridge stdout")?;
    let stderr_handle = child.stderr.take().ok_or("无法获取 bridge stderr")?;
    let reader = BufReader::new(stdout_handle);
    let stderr_reader = BufReader::new(stderr_handle);

    *state.stdin.lock().await = Some(stdin_handle);
    *state.stdout.lock().await = Some(reader);
    *state.stderr.lock().await = Some(stderr_reader);
    *state.child.lock().await = Some(child);

    let stderr_arc = state.stderr.clone();
    tokio::spawn(async move {
        let mut guard = stderr_arc.lock().await;
        if let Some(reader) = guard.as_mut() {
            let mut line = String::new();
            loop {
                line.clear();
                match tokio::io::AsyncBufReadExt::read_line(reader, &mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            println!("[fairy-action-bridge] {}", trimmed);
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn fap_bridge_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, BridgeManager>,
) -> Result<Value, String> {
    {
        let stdin_guard = state.stdin.lock().await;
        if stdin_guard.is_some() {
            return Ok(serde_json::json!({
                "success": true,
                "message": "触桥已在运行中"
            }));
        }
    }

    fap_bridge_start_inner(&app, &state).await?;

    Ok(serde_json::json!({
        "success": true,
        "message": "触桥已启动"
    }))
}

#[tauri::command]
pub async fn fap_bridge_send(
    state: tauri::State<'_, BridgeManager>,
    message: String,
) -> Result<Value, String> {
    {
        let stdin_guard = state.stdin.lock().await;
        if stdin_guard.is_none() {
            return Err("触桥未启动，请先调用 fap_bridge_start".to_string());
        }
    }

    write_frame(&state.stdin, &message).await?;
    let response = read_frame(&state.stdout).await?;

    let payload_start = response.find('#');
    let json_str = if let Some(pos) = payload_start {
        &response[pos + 1..]
    } else {
        &response
    };

    if json_str.is_empty() {
        return Ok(serde_json::json!({
            "success": true,
            "raw": response
        }));
    }

    match serde_json::from_str::<Value>(json_str) {
        Ok(v) => Ok(v),
        Err(_) => Ok(serde_json::json!({
            "success": true,
            "raw": response
        })),
    }
}

#[tauri::command]
pub async fn fap_bridge_stop(
    state: tauri::State<'_, BridgeManager>,
) -> Result<Value, String> {
    *state.stdin.lock().await = None;
    *state.stdout.lock().await = None;
    *state.stderr.lock().await = None;

    {
        let mut child_guard = state.child.lock().await;
        if let Some(ref mut child) = *child_guard {
            let _ = child.kill().await;
        }
        *child_guard = None;
    }

    Ok(serde_json::json!({
        "success": true,
        "message": "触桥已关闭"
    }))
}

fn normalize_fap(fap_path: &Path) -> Result<PathBuf, String> {
    let file = std::fs::File::open(fap_path)
        .map_err(|e| format!("无法打开 FAP 文件: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("无法读取 FAP 为 ZIP: {}", e))?;

    if archive.by_name("manifest.json").is_ok() {
        return Ok(fap_path.to_path_buf());
    }

    let mut prefix: Option<String> = None;
    for i in 0..archive.len() {
        let file = archive.by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = file.name().to_string();
        let normalized = name.replace('\\', "/");
        if normalized.ends_with("/manifest.json") {
            let slash_pos = normalized.find('/').unwrap();
            prefix = Some(normalized[..=slash_pos].to_string());
            break;
        }
    }

    let prefix = prefix.ok_or("FAP 包中未找到 manifest.json")?;

    let file = std::fs::File::open(fap_path)
        .map_err(|e| format!("无法重新打开 FAP 文件: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("无法重新读取 ZIP: {}", e))?;

    let temp_dir = std::env::temp_dir().join("aurorafairy_fap_normalize");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let original_name = fap_path.file_name()
        .ok_or("无效的文件名")?
        .to_string_lossy()
        .to_string();
    let temp_fap = temp_dir.join(format!("normalized_{}", original_name));

    let temp_file = std::fs::File::create(&temp_fap)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let mut writer = zip::ZipWriter::new(temp_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let name = file.name().to_string();
        let normalized = name.replace('\\', "/");

        if !normalized.starts_with(&prefix) {
            continue;
        }

        let stripped = &normalized[prefix.len()..];
        if stripped.is_empty() {
            continue;
        }

        if file.is_dir() {
            writer.add_directory(stripped.to_string(), options)
                .map_err(|e| format!("写入目录失败: {}", e))?;
        } else {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("读取文件内容失败: {}", e))?;
            writer.start_file(stripped.to_string(), options)
                .map_err(|e| format!("写入文件失败: {}", e))?;
            writer.write_all(&buf)
                .map_err(|e| format!("写入数据失败: {}", e))?;
        }
    }

    writer.finish()
        .map_err(|e| format!("完成 ZIP 写入失败: {}", e))?;

    Ok(temp_fap)
}

async fn run_fap_cli(
    app: &tauri::AppHandle,
    args: Vec<&str>,
) -> Result<String, String> {
    let exe_path = get_fairy_action_path(app)?;
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    let fap_install_dir = data_dir.join("fap");

    let mut cmd = Command::new(&exe_path);
    cmd.args(&args)
        .env("FA_FAP_INSTALL_DIR", &fap_install_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("执行 fairy-action 命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!("命令执行失败: {}", stderr.trim()));
    }

    Ok(stdout)
}

async fn refresh_bridge_manifests(
    stdin: &Arc<Mutex<Option<ChildStdin>>>,
    stdout: &Arc<Mutex<Option<BufReader<ChildStdout>>>>,
) -> Result<(), String> {
    let stdin_guard = stdin.lock().await;
    if stdin_guard.is_none() {
        return Ok(());
    }
    drop(stdin_guard);

    let configure_msg = "bridge://configure#{\"fap.refresh_manifests\":true}";
    write_frame(stdin, configure_msg).await?;
    let _ = read_frame(stdout).await?;
    Ok(())
}

#[tauri::command]
pub async fn fap_install(
    app: tauri::AppHandle,
    state: tauri::State<'_, BridgeManager>,
    fap_path: String,
) -> Result<Value, String> {
    let normalized_path = normalize_fap(Path::new(&fap_path))?;
    let path_str = normalized_path.to_string_lossy().to_string();
    let output = run_fap_cli(&app, vec!["fap", "install", &path_str]).await?;

    let _ = refresh_bridge_manifests(&state.stdin, &state.stdout).await;

    let trimmed = output.trim();
    match serde_json::from_str::<Value>(trimmed) {
        Ok(v) => Ok(v),
        Err(_) => Ok(serde_json::json!({
            "success": true,
            "message": trimmed
        })),
    }
}

#[tauri::command]
pub async fn fap_uninstall(
    app: tauri::AppHandle,
    state: tauri::State<'_, BridgeManager>,
    package_name: String,
) -> Result<Value, String> {
    let output = run_fap_cli(&app, vec!["fap", "uninstall", &package_name]).await?;

    let _ = refresh_bridge_manifests(&state.stdin, &state.stdout).await;

    let trimmed = output.trim();
    match serde_json::from_str::<Value>(trimmed) {
        Ok(v) => Ok(v),
        Err(_) => Ok(serde_json::json!({
            "success": true,
            "message": trimmed
        })),
    }
}

#[tauri::command]
pub async fn fap_list(
    app: tauri::AppHandle,
) -> Result<Value, String> {
    let list_output = run_fap_cli(&app, vec!["fap", "list"]).await?;
    let lines: Vec<&str> = list_output.trim().lines().collect();

    if lines.is_empty() || (lines.len() == 1 && lines[0].contains("No FAP packages")) {
        return Ok(serde_json::json!({
            "success": true,
            "packages": []
        }));
    }

    let mut packages = Vec::new();
    for line in &lines {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        let package_id = parts[0];
        let version = parts[1];
        let name = parts[2].trim_start_matches('(').trim_end_matches(')');

        let manifest_json = match run_fap_cli(&app, vec!["fap", "inspect", package_id]).await {
            Ok(output) => {
                match serde_json::from_str::<Value>(output.trim()) {
                    Ok(v) => v,
                    Err(_) => serde_json::json!({}),
                }
            }
            Err(_) => serde_json::json!({}),
        };

        packages.push(serde_json::json!({
            "package": package_id,
            "name": name,
            "version": version,
            "mode": manifest_json.get("mode").and_then(|v| v.as_str()).unwrap_or("manifest"),
            "lifecycle": manifest_json.get("lifecycle").and_then(|v| v.as_str()).unwrap_or("oneshot"),
            "platforms": manifest_json.get("platforms").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()).unwrap_or_default(),
            "capabilities": manifest_json.get("capabilities").unwrap_or(&serde_json::json!({})),
            "permissions": manifest_json.get("permissions").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()).unwrap_or_default(),
            "signature": manifest_json.get("signature").unwrap_or(&serde_json::Value::Null),
        }));
    }

    Ok(serde_json::json!({
        "success": true,
        "packages": packages
    }))
}
