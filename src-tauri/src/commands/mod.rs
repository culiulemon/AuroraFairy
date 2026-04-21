pub mod shell;
pub mod file;
pub mod watcher;
pub mod security_rules;
pub mod tts;
pub mod local_models;
pub mod browser;
pub mod fbm_fs;
pub mod qdrant_manager;
pub mod feishu;
pub mod weixin;
pub mod proxy;

pub use shell::shell_execute;
pub use file::{file_read, file_write, file_delete, file_edit, file_glob, file_grep};
pub use watcher::start_tool_watcher;
pub use security_rules::{load_security_rules, save_security_rules, delete_security_rules};
pub use tts::{tts_generate, tts_list_voices};
pub use local_models::{
    check_environment, search_models, download_model, cancel_download,
    deploy_model, stop_model, delete_model, get_model_info, convert_model_to_ir,
    install_dependency,
};
pub use browser::{BrowserManager, browser_start, browser_execute, browser_stop};
pub use proxy::{proxy_chat, proxy_chat_stream};
pub mod fap;

