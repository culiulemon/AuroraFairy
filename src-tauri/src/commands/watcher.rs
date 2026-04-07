use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn start_tool_watcher(app: AppHandle) -> Result<(), String> {
    let fairy_tool_path = crate::get_fairy_tool_path_internal(&app)?;

    let (tx, rx) = mpsc::channel::<Vec<String>>();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let ts_paths: Vec<String> = event.paths
                    .iter()
                    .filter(|p| p.extension().map_or(false, |ext| ext == "ts"))
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                if !ts_paths.is_empty() {
                    let _ = tx.send(ts_paths);
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("创建监听器失败: {}", e))?;

    watcher.watch(&fairy_tool_path, RecursiveMode::Recursive)
        .map_err(|e| format!("监听目录失败: {}", e))?;

    std::thread::spawn(move || {
        let _watcher = watcher;
        let mut last_emit = std::time::Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(ts_paths) => {
                    let now = std::time::Instant::now();
                    if now.duration_since(last_emit) < Duration::from_millis(500) {
                        continue;
                    }
                    last_emit = now;
                    let _ = app.emit("fairy-tool-changed", ts_paths);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Ok(())
}
