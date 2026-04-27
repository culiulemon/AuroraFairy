use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use walkdir::WalkDir;

mod commands;

use commands::start_tool_watcher;
use commands::{tts_generate, tts_list_voices};
use commands::{
    check_environment, search_models, download_model, cancel_download,
    deploy_model, stop_model, delete_model, get_model_info,
};
use commands::convert_model_to_ir;
use commands::install_dependency;
use commands::{BrowserManager, browser_start, browser_execute, browser_stop};
use commands::fbm_fs::{
    fbm_mkdir, fbm_write_file, fbm_read_file, fbm_unlink, fbm_readdir,
    fbm_readdir_detailed, fbm_stat, fbm_exists, fbm_join, fbm_basename,
    fbm_extname, fbm_start_watch, fbm_stop_watch, fbm_write_file_binary,
    fbm_read_file_binary, FbmWatcherState,
};
use commands::qdrant_manager::{QdrantState, qdrant_start, qdrant_stop, qdrant_status};
use commands::feishu::{FeishuManager, feishu_connect, feishu_disconnect, feishu_reply_message, feishu_get_status};
use commands::weixin::{WeixinManager, weixin_get_qrcode, weixin_connect, weixin_disconnect, weixin_reply_message, weixin_get_status, weixin_has_credentials};
use commands::fap::{BridgeManager, fap_bridge_start, fap_bridge_send, fap_bridge_stop, fap_install, fap_uninstall, fap_list};
use commands::proxy::{proxy_chat, proxy_chat_stream};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    #[serde(rename = "globalWorkingDir")]
    global_working_dir: Option<String>,
    #[serde(rename = "perConversationWorkingDir")]
    per_conversation_working_dir: Option<String>,
    #[serde(rename = "modelsDir")]
    models_dir: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            global_working_dir: None,
            per_conversation_working_dir: None,
            models_dir: None,
        }
    }
}

fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_data_dir(app_handle)?;
    Ok(data_dir.join("config.json"))
}

fn load_app_config(app_handle: &tauri::AppHandle) -> AppConfig {
    let config_path = match get_config_path(app_handle) {
        Ok(p) => p,
        Err(_) => return AppConfig::default(),
    };
    if !config_path.exists() {
        return AppConfig::default();
    }
    match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub(crate) fn get_configured_models_dir(app_handle: &tauri::AppHandle) -> Option<String> {
    let config = load_app_config(app_handle);
    config.models_dir
}

fn save_app_config(app_handle: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))
}

const COMPILE_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub(crate) fn get_app_dir() -> PathBuf {
    let compile_dir = PathBuf::from(COMPILE_MANIFEST_DIR);
    if compile_dir.join("Cargo.toml").exists()
        || compile_dir.join("target").exists() {
        return compile_dir;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if exe_dir.join("Cargo.toml").exists() {
                return exe_dir.to_path_buf();
            }
        }
    }
    compile_dir
}

fn get_working_dir(app_handle: &tauri::AppHandle) -> String {
    let config = load_app_config(app_handle);
    if let Some(ref dir) = config.global_working_dir {
        if std::path::Path::new(dir).exists() {
            return dir.clone();
        }
    }
    get_data_dir(app_handle)
        .map(|d| d.to_string_lossy().to_string())
        .unwrap_or_else(|_| get_app_dir().to_string_lossy().to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolFile {
    pub name: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub tools: Vec<ToolFile>,
    pub fairy_tool_path: String,
}

pub(crate) fn get_project_root() -> Result<PathBuf, String> {
    let app_dir = get_app_dir();
    if app_dir.file_name().map_or(false, |n| n == "src-tauri") {
        if let Some(parent) = app_dir.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(app_dir)
}

pub(crate) fn get_data_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("无法创建应用数据目录: {}", e))?;
    }
    Ok(data_dir)
}

pub(crate) fn get_fairy_tool_path_internal(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_data_dir(app_handle)?;
    let fairy_tool_path = data_dir.join("FairyTool");

    if !fairy_tool_path.exists() {
        fs::create_dir_all(&fairy_tool_path)
            .map_err(|e| format!("无法创建 FairyTool 目录: {}", e))?;
    }

    Ok(fairy_tool_path)
}

pub(crate) fn get_fairy_workspace_path_internal(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_data_dir(app_handle)?;
    let fairy_workspace_path = data_dir.join("FairyWorkSpace");

    if !fairy_workspace_path.exists() {
        fs::create_dir_all(&fairy_workspace_path)
            .map_err(|e| format!("无法创建 FairyWorkSpace 目录: {}", e))?;
    }

    Ok(fairy_workspace_path)
}

#[tauri::command]
fn scan_tools(app: tauri::AppHandle) -> Result<ScanResult, String> {
    let fairy_tool_path = get_fairy_tool_path_internal(&app)?;
    println!("[Rust] 扫描目录: {:?}", fairy_tool_path);

    let mut tools: Vec<ToolFile> = Vec::new();

    for entry in WalkDir::new(&fairy_tool_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "ts") {
            if let Ok(content) = fs::read_to_string(path) {
                let name = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                tools.push(ToolFile {
                    name,
                    path: path.to_string_lossy().to_string(),
                    content,
                });
            }
        }
    }

    println!("[Rust] 扫描到 {} 个工具文件", tools.len());

    Ok(ScanResult {
        tools,
        fairy_tool_path: fairy_tool_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn save_tool_file(
    app: tauri::AppHandle,
    filename: String,
    content: String,
) -> Result<String, String> {
    let fairy_tool_path = get_fairy_tool_path_internal(&app)?;

    let file_path = fairy_tool_path.join(&filename);
    fs::write(&file_path, content)
        .map_err(|e| format!("无法保存工具文件: {}", e))?;

    println!("[Rust] 工具文件已保存: {:?}", file_path);

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn delete_tool_file(
    app: tauri::AppHandle,
    filename: String,
) -> Result<(), String> {
    let fairy_tool_path = get_fairy_tool_path_internal(&app)?;
    let file_path = fairy_tool_path.join(&filename);

    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("无法删除工具文件: {}", e))?;
        println!("[Rust] 工具文件已删除: {:?}", file_path);
    }

    Ok(())
}

#[tauri::command]
fn get_fairy_tool_path(app: tauri::AppHandle) -> Result<String, String> {
    let fairy_tool_path = get_fairy_tool_path_internal(&app)?;
    Ok(fairy_tool_path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    opener::open(&path)
        .map_err(|e| format!("无法打开目录: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn shell_execute(command: String, timeout: u64, shell_type: Option<String>) -> Result<String, String> {
    commands::shell_execute(command, timeout, shell_type).await
}

#[tauri::command]
fn file_read(app: tauri::AppHandle, path: String, offset: Option<usize>, limit: Option<usize>, raw: Option<bool>, extra_allowed_paths: Option<Vec<String>>, working_dir_override: Option<String>) -> Result<String, String> {
    let working_dir = working_dir_override.filter(|d| !d.is_empty()).unwrap_or_else(|| get_working_dir(&app));
    let data_dir = get_data_dir(&app).ok().map(|d| d.to_string_lossy().to_string());
    if raw == Some(true) {
        let safe_path = commands::file::validate_path(&path, &working_dir, data_dir.as_deref(), extra_allowed_paths.as_deref())?;
        std::fs::read_to_string(&safe_path)
            .map_err(|e| format!("读取文件失败: {}", e))
    } else {
        commands::file_read(&path, offset, limit, &working_dir, data_dir.as_deref(), extra_allowed_paths.as_deref())
    }
}

#[tauri::command]
fn file_write(app: tauri::AppHandle, path: String, content: String, extra_allowed_paths: Option<Vec<String>>, working_dir_override: Option<String>) -> Result<(), String> {
    let working_dir = working_dir_override.filter(|d| !d.is_empty()).unwrap_or_else(|| get_working_dir(&app));
    let data_dir = get_data_dir(&app).ok().map(|d| d.to_string_lossy().to_string());
    let p = std::path::Path::new(&path);
    let full_path = if p.parent().map_or(true, |parent| parent.as_os_str().is_empty()) {
        let fairy_workspace = get_fairy_workspace_path_internal(&app)?;
        fairy_workspace.join(&path)
    } else {
        std::path::Path::new(&working_dir).join(&path)
    };
    let full_path_str = full_path.to_string_lossy().to_string();
    commands::file_write(&full_path_str, &content, &working_dir, data_dir.as_deref(), extra_allowed_paths.as_deref())
}

#[tauri::command]
fn file_delete(app: tauri::AppHandle, path: String, extra_allowed_paths: Option<Vec<String>>, working_dir_override: Option<String>) -> Result<(), String> {
    let working_dir = working_dir_override.filter(|d| !d.is_empty()).unwrap_or_else(|| get_working_dir(&app));
    let data_dir = get_data_dir(&app).ok().map(|d| d.to_string_lossy().to_string());
    commands::file_delete(&path, &working_dir, data_dir.as_deref(), extra_allowed_paths.as_deref())
}

#[tauri::command]
fn file_edit(app: tauri::AppHandle, path: String, old_str: String, new_str: String, extra_allowed_paths: Option<Vec<String>>, working_dir_override: Option<String>) -> Result<String, String> {
    let working_dir = working_dir_override.filter(|d| !d.is_empty()).unwrap_or_else(|| get_working_dir(&app));
    let data_dir = get_data_dir(&app).ok().map(|d| d.to_string_lossy().to_string());
    commands::file_edit(&path, &old_str, &new_str, &working_dir, data_dir.as_deref(), extra_allowed_paths.as_deref())
}

#[tauri::command]
fn file_glob(app: tauri::AppHandle, pattern: String, working_dir_override: Option<String>) -> Result<Vec<String>, String> {
    let working_dir = working_dir_override.filter(|d| !d.is_empty()).unwrap_or_else(|| get_working_dir(&app));
    let data_dir = get_data_dir(&app).ok().map(|d| d.to_string_lossy().to_string());
    commands::file_glob(&pattern, &working_dir, data_dir.as_deref())
}

#[tauri::command]
fn load_security_rules() -> Result<String, String> {
    commands::load_security_rules()
}

#[tauri::command]
fn save_security_rules(content: String) -> Result<(), String> {
    commands::save_security_rules(content)
}

#[tauri::command]
fn delete_security_rules() -> Result<(), String> {
    commands::delete_security_rules()
}

#[tauri::command]
fn file_grep(app: tauri::AppHandle, path: String, pattern: String, extra_allowed_paths: Option<Vec<String>>, working_dir_override: Option<String>) -> Result<Vec<String>, String> {
    let working_dir = working_dir_override.filter(|d| !d.is_empty()).unwrap_or_else(|| get_working_dir(&app));
    let data_dir = get_data_dir(&app).ok().map(|d| d.to_string_lossy().to_string());
    commands::file_grep(&path, &pattern, &working_dir, data_dir.as_deref(), extra_allowed_paths.as_deref())
}

#[derive(Serialize)]
struct WorkingDirConfigResponse {
    global_working_dir: Option<String>,
    default_working_dir: String,
}

#[tauri::command]
fn get_working_dir_config(app: tauri::AppHandle) -> Result<WorkingDirConfigResponse, String> {
    let config = load_app_config(&app);
    let default_dir = get_data_dir(&app)
        .map(|d| d.to_string_lossy().to_string())
        .unwrap_or_else(|_| get_app_dir().to_string_lossy().to_string());
    Ok(WorkingDirConfigResponse {
        global_working_dir: config.global_working_dir,
        default_working_dir: default_dir,
    })
}

#[tauri::command]
fn set_working_dir_config(app: tauri::AppHandle, working_dir: Option<String>) -> Result<(), String> {
    let mut config = load_app_config(&app);
    config.global_working_dir = working_dir;
    save_app_config(&app, &config)
}

#[derive(Serialize)]
struct ModelsDirConfigResponse {
    models_dir: Option<String>,
    default_models_dir: String,
}

#[tauri::command]
fn get_models_dir_config(app: tauri::AppHandle) -> Result<ModelsDirConfigResponse, String> {
    let config = load_app_config(&app);
    let default_dir = get_data_dir(&app)
        .map(|d| d.join("models").to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from(""));
    Ok(ModelsDirConfigResponse {
        models_dir: config.models_dir,
        default_models_dir: default_dir,
    })
}

#[tauri::command]
fn set_models_dir_config(app: tauri::AppHandle, models_dir: Option<String>) -> Result<(), String> {
    let mut config = load_app_config(&app);
    config.models_dir = models_dir;
    save_app_config(&app, &config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    fn migrate_legacy_data(app_handle: &tauri::AppHandle) {
        let app_dir = get_app_dir();
        let data_dir = match get_data_dir(app_handle) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[迁移] 无法获取数据目录: {}", e);
                return;
            }
        };

        for dir_name in &["FairyTool", "FairyWorkSpace"] {
            let old_path = app_dir.join(dir_name);
            let new_path = data_dir.join(dir_name);

            if !old_path.exists() || new_path.exists() {
                continue;
            }

            match copy_dir_recursive(&old_path, &new_path) {
                Ok(_) => println!("[迁移] 已将 {} 从 {:?} 迁移到 {:?}", dir_name, old_path, new_path),
                Err(e) => eprintln!("[迁移] 迁移 {} 失败: {}", dir_name, e),
            }
        }
    }

    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))
        .ok();

    tauri::Builder::default()
        .setup(move |app| {
            migrate_legacy_data(&app.handle());
            if let Some(window) = app.get_webview_window("main") {
                if let Some(ref icon) = tray_icon {
                    let _ = window.set_icon(icon.clone());
                }
            }

            let show_item = tauri::menu::MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "退出 AuroraFairy", true, None::<&str>)?;
            let tray_menu = tauri::menu::MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray = tauri::tray::TrayIconBuilder::with_id("main-tray")
                .tooltip("AuroraFairy");
            let tray = if let Some(ref icon) = tray_icon {
                tray.icon(icon.clone())
            } else {
                tray
            };
            let tray = tray.menu(&tray_menu);
            let tray = tray.show_menu_on_left_click(false);
            let tray = tray.on_menu_event(move |app, event| {
                match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_always_on_top(true);
                            let _ = window.set_focus();
                            let _ = window.set_always_on_top(false);
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            });
            let tray = tray.on_tray_icon_event(|tray, event| {
                if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_always_on_top(true);
                        let _ = window.set_focus();
                        let _ = window.set_always_on_top(false);
                    }
                }
            });
            let _ = tray.build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(BrowserManager::new())
        .manage(FbmWatcherState::default())
        .manage(Mutex::new(QdrantState::new()))
        .manage(std::sync::Arc::new(tokio::sync::Mutex::new(FeishuManager::new())))
        .manage(std::sync::Arc::new(tokio::sync::Mutex::new(WeixinManager::new())))
        .manage(BridgeManager::new())
        .invoke_handler(tauri::generate_handler![
            scan_tools,
            save_tool_file,
            delete_tool_file,
            get_fairy_tool_path,
            open_folder,
            get_working_dir_config,
            set_working_dir_config,
            get_models_dir_config,
            set_models_dir_config,
            shell_execute,
            file_read,
            file_write,
            file_delete,
            file_edit,
            file_glob,
            file_grep,
            start_tool_watcher,
            load_security_rules,
            save_security_rules,
            delete_security_rules,
            tts_generate,
            tts_list_voices,
            check_environment,
            search_models,
            download_model,
            cancel_download,
            deploy_model,
            stop_model,
            delete_model,
            get_model_info,
            convert_model_to_ir,
            install_dependency,
            browser_start,
            browser_execute,
            browser_stop,
            fbm_mkdir,
            fbm_write_file,
            fbm_read_file,
            fbm_unlink,
            fbm_readdir,
            fbm_readdir_detailed,
            fbm_stat,
            fbm_exists,
            fbm_join,
            fbm_basename,
            fbm_extname,
            fbm_start_watch,
            fbm_stop_watch,
            fbm_write_file_binary,
            fbm_read_file_binary,
            qdrant_start,
            qdrant_stop,
            qdrant_status,
            feishu_connect,
            feishu_disconnect,
            feishu_reply_message,
            feishu_get_status,
            weixin_get_qrcode,
            weixin_connect,
            weixin_disconnect,
            weixin_reply_message,
            weixin_get_status,
            weixin_has_credentials,
            fap_bridge_start,
            fap_bridge_send,
            fap_bridge_stop,
            fap_install,
            fap_uninstall,
            fap_list,
            proxy_chat,
            proxy_chat_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
