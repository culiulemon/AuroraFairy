use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

fn validate_fbm_path(path: &str) -> Result<(), String> {
    if path.contains("..") {
        return Err("path traversal not allowed".to_string());
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DirEntryInfo {
    pub name: String,
    pub is_directory: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileStatInfo {
    pub mtime_ms: u64,
    pub birthtime_ms: u64,
    pub is_file: bool,
    pub is_directory: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FbmWatchEvent {
    pub event: String,
    pub path: String,
}

pub struct FbmWatcherState {
    pub watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
}

impl Default for FbmWatcherState {
    fn default() -> Self {
        Self {
            watcher: Arc::new(Mutex::new(None)),
        }
    }
}

#[tauri::command]
pub async fn fbm_mkdir(path: String) -> Result<(), String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("mkdir failed: {}", e))
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_write_file(path: String, content: String) -> Result<(), String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create parent dir failed: {}", e))?;
        }
        std::fs::write(&path, content)
            .map_err(|e| format!("write file failed: {}", e))
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_read_file(path: String) -> Result<String, String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("read file failed: {}", e))
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_unlink(path: String) -> Result<(), String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        std::fs::remove_file(&path)
            .map_err(|e| format!("unlink failed: {}", e))
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_readdir(path: String) -> Result<Vec<String>, String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        let mut entries = std::fs::read_dir(&path)
            .map_err(|e| format!("readdir failed: {}", e))?;
        let mut names = Vec::new();
        while let Some(entry) = entries.next() {
            let entry = entry.map_err(|e| format!("readdir entry error: {}", e))?;
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();
            names.push(name);
        }
        Ok(names)
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_readdir_detailed(path: String) -> Result<Vec<DirEntryInfo>, String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        let mut entries = std::fs::read_dir(&path)
            .map_err(|e| format!("readdir failed: {}", e))?;
        let mut result = Vec::new();
        while let Some(entry) = entries.next() {
            let entry = entry.map_err(|e| format!("readdir entry error: {}", e))?;
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();
            let is_directory = entry
                .file_type()
                .map_err(|e| format!("file type error: {}", e))?
                .is_dir();
            result.push(DirEntryInfo { name, is_directory });
        }
        Ok(result)
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_stat(path: String) -> Result<FileStatInfo, String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("stat failed: {}", e))?;
        let mtime_ms = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let birthtime_ms = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Ok(FileStatInfo {
            mtime_ms,
            birthtime_ms,
            is_file: metadata.is_file(),
            is_directory: metadata.is_dir(),
        })
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_exists(path: String) -> Result<bool, String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || -> Result<bool, String> { Ok(Path::new(&path).exists()) })
        .await
        .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_join(segments: Vec<String>) -> Result<String, String> {
    if segments.is_empty() {
        return Err("no segments provided".to_string());
    }
    let mut buf = PathBuf::new();
    for seg in &segments {
        buf.push(seg);
    }
    Ok(buf.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn fbm_basename(path: String, ext: Option<String>) -> Result<String, String> {
    let p = Path::new(&path);
    let name = match ext {
        Some(_) => p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        None => p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
    };
    Ok(name)
}

#[tauri::command]
pub async fn fbm_extname(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    let ext = p
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    Ok(ext)
}

#[tauri::command]
pub async fn fbm_start_watch(app: AppHandle, dir: String) -> Result<(), String> {
    validate_fbm_path(&dir)?;

    if !Path::new(&dir).exists() {
        return Err("watch directory does not exist".to_string());
    }

    let (tx, rx) = mpsc::channel::<(String, String)>();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                for path in &event.paths {
                    if path.extension().map_or(false, |ext| ext == "md") {
                        let kind = match event.kind {
                            notify::EventKind::Create(_) => "add",
                            notify::EventKind::Modify(_) => "change",
                            notify::EventKind::Remove(_) => "remove",
                            _ => continue,
                        };
                        let _ = tx.send((kind.to_string(), path.to_string_lossy().to_string()));
                    }
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("create watcher failed: {}", e))?;

    watcher
        .watch(Path::new(&dir), RecursiveMode::Recursive)
        .map_err(|e| format!("watch directory failed: {}", e))?;

    {
        let state = app.state::<FbmWatcherState>();
        let mut guard = state.watcher.lock().map_err(|e| format!("lock error: {}", e))?;
        if guard.is_some() {
            drop(guard);
            drop(watcher);
            return Err("watcher already running".to_string());
        }
        *guard = Some(watcher);
    }

    std::thread::spawn(move || {
        let mut last_emit = std::time::Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok((kind, path)) => {
                    let now = std::time::Instant::now();
                    if now.duration_since(last_emit) < Duration::from_millis(500) {
                        continue;
                    }
                    last_emit = now;
                    let _ = app.emit(
                        "fbm-fs-watch",
                        FbmWatchEvent {
                            event: kind,
                            path,
                        },
                    );
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn fbm_stop_watch(app: AppHandle) -> Result<(), String> {
    let state = app.state::<FbmWatcherState>();
    let mut guard = state
        .watcher
        .lock()
        .map_err(|e| format!("lock error: {}", e))?;
    *guard = None;
    Ok(())
}

#[tauri::command]
pub async fn fbm_write_file_binary(path: String, data: String) -> Result<(), String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&data)
            .map_err(|e| format!("base64 decode failed: {}", e))?;
        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create parent dir failed: {}", e))?;
        }
        std::fs::write(&path, bytes)
            .map_err(|e| format!("write binary file failed: {}", e))
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn fbm_read_file_binary(path: String) -> Result<String, String> {
    validate_fbm_path(&path)?;
    tokio::task::spawn_blocking(move || {
        use base64::Engine;
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("read binary file failed: {}", e))?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
    })
    .await
    .map_err(|e| format!("task join error: {}", e))?
}
