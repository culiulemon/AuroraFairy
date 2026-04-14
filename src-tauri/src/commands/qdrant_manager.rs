use serde::Serialize;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use tauri::Manager;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct QdrantState {
    process: Option<Child>,
    port: u16,
    storage_path: String,
}

impl QdrantState {
    pub fn new() -> Self {
        QdrantState {
            process: None,
            port: 0,
            storage_path: String::new(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct QdrantStatus {
    pub running: bool,
    pub port: u16,
    pub storage_path: String,
}

fn kill_orphaned_qdrant() {
    let mut cmd = std::process::Command::new("taskkill");
    cmd.args(["/F", "/IM", "qdrant.exe"]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            eprintln!("[Qdrant] Killed orphaned qdrant processes");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

fn clean_qdrant_locks(storage_path: &std::path::Path) {
    fn remove_locks_recursive(dir: &std::path::Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    remove_locks_recursive(&path);
                } else {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.ends_with(".lock") {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
    if storage_path.exists() {
        remove_locks_recursive(storage_path);
    }
}

fn find_available_port() -> Option<u16> {
    for port in 16333..16433 {
        if TcpStream::connect(("127.0.0.1", port)).is_err() {
            return Some(port);
        }
    }
    None
}

fn find_qdrant_binary_with_app(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let binary: PathBuf = resource_dir.join("binaries").join("qdrant.exe");
        if binary.exists() {
            eprintln!("[Qdrant] Found binary via resource_dir: {:?}", binary);
            return Ok(binary);
        }
        eprintln!("[Qdrant] resource_dir binary not found at: {:?}", binary);
    } else {
        eprintln!("[Qdrant] Failed to get resource_dir");
    }

    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Cannot determine exe path: {}", e))?
        .parent()
        .ok_or("Cannot determine exe directory")?
        .to_path_buf();

    let binary = exe_dir.join("qdrant.exe");
    if binary.exists() {
        eprintln!("[Qdrant] Found binary via exe_dir: {:?}", binary);
        return Ok(binary);
    }
    eprintln!("[Qdrant] exe_dir binary not found at: {:?}", binary);

    let binary2 = exe_dir.join("binaries").join("qdrant.exe");
    if binary2.exists() {
        eprintln!("[Qdrant] Found binary via exe_dir/binaries: {:?}", binary2);
        return Ok(binary2);
    }
    eprintln!("[Qdrant] exe_dir/binaries binary not found at: {:?}", binary2);

    let dev_fallback = exe_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("binaries").join("qdrant.exe"));
    if let Some(ref fallback) = dev_fallback {
        if fallback.exists() {
            eprintln!("[Qdrant] Found binary via dev_fallback: {:?}", fallback);
            return Ok(fallback.clone());
        }
        eprintln!("[Qdrant] dev_fallback binary not found at: {:?}", fallback);
    }

    Err(format!(
        "Qdrant binary not found. Searched: resource_dir/binaries/qdrant.exe, exe_dir/qdrant.exe, exe_dir/binaries/qdrant.exe, dev_fallback"
    ))
}

fn write_qdrant_config(path: &std::path::Path, port: u16, storage_path: &std::path::Path) -> Result<(), String> {
    let storage_str = storage_path.to_string_lossy().replace('\\', "/");
    let config = format!(
        r#"service:
  http_port: {}
  host: 127.0.0.1
  enable_cors: true
  grpc_port: null

storage:
  storage_path: {}

log_level: WARN
"#,
        port,
        storage_str
    );
    std::fs::write(path, config).map_err(|e| format!("Failed to write Qdrant config: {}", e))
}

fn wait_for_qdrant(port: u16, timeout_secs: u64) -> Result<(), String> {
    let start = std::time::Instant::now();

    while start.elapsed().as_secs() < timeout_secs {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(500));
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Err(format!(
        "Qdrant failed to start within {} seconds",
        timeout_secs
    ))
}

#[tauri::command]
pub async fn qdrant_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<QdrantState>>,
    working_dir: String,
) -> Result<u16, String> {
    let qstate = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    if qstate.process.is_some() {
        return Ok(qstate.port);
    }

    drop(qstate);

    kill_orphaned_qdrant();

    let port = find_available_port().ok_or("No available port in range 16333-16433")?;

    let storage_path = PathBuf::from(&working_dir).join("memories").join("qdrant_storage");
    clean_qdrant_locks(&storage_path);

    match try_start_qdrant(&app, &working_dir, port, &storage_path) {
        Ok(child) => {
            let mut qstate = state.lock().map_err(|e| format!("Lock error: {}", e))?;
            qstate.process = Some(child);
            qstate.port = port;
            qstate.storage_path = storage_path.to_string_lossy().to_string();
            Ok(port)
        }
        Err(first_err) => {
            eprintln!("[Qdrant] First start attempt failed: {}", first_err);
            clean_qdrant_locks(&storage_path);
            std::thread::sleep(std::time::Duration::from_secs(1));

            kill_orphaned_qdrant();

            let port2 = find_available_port().ok_or("No available port in range 16333-16433")?;
            match try_start_qdrant(&app, &working_dir, port2, &storage_path) {
                Ok(child) => {
                    let mut qstate = state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    qstate.process = Some(child);
                    qstate.port = port2;
                    qstate.storage_path = storage_path.to_string_lossy().to_string();
                    Ok(port2)
                }
                Err(second_err) => Err(format!(
                    "Qdrant failed to start after retry. First: {}. Second: {}",
                    first_err, second_err
                )),
            }
        }
    }
}

fn try_start_qdrant(
    app: &tauri::AppHandle,
    working_dir: &str,
    port: u16,
    storage_path: &std::path::Path,
) -> Result<Child, String> {
    std::fs::create_dir_all(storage_path)
        .map_err(|e| format!("Failed to create storage dir: {}", e))?;

    let config_path = PathBuf::from(working_dir).join("memories").join("qdrant_config.yaml");
    write_qdrant_config(&config_path, port, storage_path)?;

    let qdrant_binary = find_qdrant_binary_with_app(app)?;

    let log_path = PathBuf::from(working_dir).join("memories").join("qdrant.log");
    let log_file = std::fs::File::create(&log_path)
        .map_err(|e| format!("Failed to create qdrant.log: {}", e))?;
    let log_file_clone = log_file.try_clone()
        .map_err(|e| format!("Failed to clone log file handle: {}", e))?;

    let memories_dir = PathBuf::from(working_dir).join("memories");
    std::fs::create_dir_all(&memories_dir)
        .map_err(|e| format!("Failed to create memories dir: {}", e))?;

    let mut cmd = Command::new(&qdrant_binary);
    cmd.arg("--config-path")
        .arg(&config_path)
        .current_dir(&memories_dir)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_clone));
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start Qdrant at {:?}: {}", qdrant_binary, e))?;

    match wait_for_qdrant(port, 30) {
        Ok(()) => Ok(child),
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let log_contents = std::fs::read_to_string(&log_path).unwrap_or_default();
            Err(format!("{}. Log:\n{}", e, log_contents))
        }
    }
}

#[tauri::command]
pub async fn qdrant_stop(
    state: tauri::State<'_, Mutex<QdrantState>>,
) -> Result<(), String> {
    let mut qstate = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref mut child) = qstate.process {
        let _ = child.kill();
        let _ = child.wait();
        qstate.process = None;
        qstate.port = 0;
    }
    Ok(())
}

#[tauri::command]
pub async fn qdrant_status(
    state: tauri::State<'_, Mutex<QdrantState>>,
) -> Result<QdrantStatus, String> {
    let qstate = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(QdrantStatus {
        running: qstate.process.is_some(),
        port: qstate.port,
        storage_path: qstate.storage_path.clone(),
    })
}
