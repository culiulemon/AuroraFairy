use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, ChildStderr, Command};
use tokio::sync::Mutex;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct BrowserManager {
    child: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    stderr: Arc<Mutex<Option<BufReader<ChildStderr>>>>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            stderr: Arc::new(Mutex::new(None)),
        }
    }
}

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn get_fairy_action_path() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
        .parent()
        .ok_or("无法获取父目录")?
        .to_path_buf();

    let manifest_path = PathBuf::from(MANIFEST_DIR);

    let candidates: Vec<PathBuf> = vec![
        manifest_path.join("binaries").join("fairy-action.exe"),
        manifest_path.join("binaries").join("fairy-action"),
        exe_dir.join("binaries").join("fairy-action.exe"),
        exe_dir.join("binaries").join("fairy-action"),
        exe_dir.join("fairy-action.exe"),
        exe_dir.join("fairy-action"),
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
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n  ");
    Err(format!(
        "找不到 fairy-action 二进制文件。已搜索以下路径:\n  {}\n请将其放置在 src-tauri/binaries/ 目录中",
        searched
    ))
}

async fn send_and_receive(
    stdin: &Arc<Mutex<Option<ChildStdin>>>,
    stdout: &Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    request: &Value,
) -> Result<Value, String> {
    let mut line = serde_json::to_string(request)
        .map_err(|e| format!("序列化请求失败: {}", e))?;
    line.push('\n');

    {
        let mut stdin_guard = stdin.lock().await;
        let stdin_handle = stdin_guard.as_mut().ok_or("stdin 不可用")?;
        stdin_handle
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("写入 stdin 失败: {}", e))?;
        stdin_handle
            .flush()
            .await
            .map_err(|e| format!("刷新 stdin 失败: {}", e))?;
    }

    let mut response_line = String::new();
    {
        let mut stdout_guard = stdout.lock().await;
        let reader = stdout_guard.as_mut().ok_or("stdout 不可用")?;
        for _ in 0..100 {
            response_line.clear();
            reader
                .read_line(&mut response_line)
                .await
                .map_err(|e| format!("读取 stdout 失败: {}", e))?;
            let trimmed = response_line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                break;
            }
        }
    }

    let trimmed = response_line.trim();
    if trimmed.is_empty() {
        return Err("未收到有效 JSON 响应".to_string());
    }
    serde_json::from_str(trimmed)
        .map_err(|e| format!("解析响应失败: {} - 原始内容: {}", e, trimmed))
}

#[tauri::command]
pub async fn browser_start(
    state: tauri::State<'_, BrowserManager>,
    show_browser: Option<bool>,
    default_search_engine: Option<String>,
) -> Result<Value, String> {
    {
        let child_guard = state.child.lock().await;
        if child_guard.is_some() {
            return Ok(serde_json::json!({
                "success": true,
                "message": "浏览器已在运行中"
            }));
        }
    }

    let exe_path = get_fairy_action_path()?;

    let mut cmd = Command::new(&exe_path);
    cmd.arg("run")
        .arg("--log-level")
        .arg("warn")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    if show_browser == Some(true) {
        cmd.arg("--show-browser");
    }

    if let Some(engine) = &default_search_engine {
        cmd.env("FA_DEFAULT_SEARCH_ENGINE", engine);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 fairy-action 失败: {}", e))?;

    let stdin_handle = child.stdin.take().ok_or("无法获取 stdin")?;
    let stdout_handle = child.stdout.take().ok_or("无法获取 stdout")?;
    let stderr_handle = child.stderr.take().ok_or("无法获取 stderr")?;
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
                            if trimmed.contains("panic")
                                || trimmed.contains("ERROR")
                                || trimmed.contains("error")
                            {
                                eprintln!("[fairy-action ERROR] {}", trimmed);
                            } else {
                                println!("[fairy-action] {}", trimmed);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    });

    Ok(serde_json::json!({
        "success": true,
        "message": "浏览器已启动"
    }))
}

#[tauri::command]
pub async fn browser_execute(
    state: tauri::State<'_, BrowserManager>,
    request: Value,
) -> Result<Value, String> {
    {
        let stdin_guard = state.stdin.lock().await;
        if stdin_guard.is_none() {
            return Err("浏览器未启动，请先调用 browser_start".to_string());
        }
    }

    let response = send_and_receive(&state.stdin, &state.stdout, &request).await?;

    if response.get("type").and_then(|t| t.as_str()) == Some("closed") {
        *state.stdin.lock().await = None;
        *state.stdout.lock().await = None;
        *state.stderr.lock().await = None;
        let mut child_guard = state.child.lock().await;
        if let Some(ref mut child) = *child_guard {
            let _ = child.kill().await;
        }
        *child_guard = None;
    }

    Ok(response)
}

#[tauri::command]
pub async fn browser_stop(
    state: tauri::State<'_, BrowserManager>,
) -> Result<Value, String> {
    let close_request = serde_json::json!({"type": "close"});
    let _ = send_and_receive(&state.stdin, &state.stdout, &close_request).await;

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
        "message": "浏览器已关闭"
    }))
}
