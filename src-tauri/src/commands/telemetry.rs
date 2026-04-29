use reqwest::Client;
use serde::Serialize;
use std::fs;
use tauri::Manager;

const TRACKING_URL: &str = "https://rj.lovestu.com/api/v1/update/check";
const API_KEY: &str = "07e0f0dd3d865cd005273c5ff3b874e6b74312f692b55063";
const INSTALL_ID_FILE: &str = "install_id";

#[derive(Serialize)]
struct TrackingPayload {
    product: &'static str,
    app_version: String,
    version_code: u64,
    install_id: String,
    channel: &'static str,
    env: TrackingEnv,
}

#[derive(Serialize)]
struct TrackingEnv {
    os: String,
    os_version: String,
    locale: String,
}

fn get_or_create_install_id(data_dir: &std::path::Path) -> String {
    let id_file = data_dir.join(INSTALL_ID_FILE);
    if let Ok(existing) = fs::read_to_string(&id_file) {
        let trimmed = existing.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    let new_id = uuid::Uuid::new_v4().to_string();
    let _ = fs::write(&id_file, &new_id);
    new_id
}

fn parse_version_code(version: &str) -> u64 {
    let parts: Vec<&str> = version.split('.').collect();
    let major = parts.first().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    major * 100000 + minor * 1000 + patch
}

fn get_os_info() -> (String, String) {
    let os = std::env::consts::OS.to_string();
    let os_version = if cfg!(target_os = "windows") {
        get_windows_version()
    } else if cfg!(target_os = "macos") {
        get_macos_version()
    } else {
        get_linux_version()
    };
    (os, os_version)
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> String {
    let output = std::process::Command::new("cmd")
        .args(["/C", "ver"])
        .output()
        .ok();
    if let Some(out) = output {
        let ver_str = String::from_utf8_lossy(&out.stdout);
        if let Some(start) = ver_str.find("Version ") {
            let rest = &ver_str[start + 8..];
            let end = rest.find(']').unwrap_or(rest.len());
            return rest[..end].trim().to_string();
        }
    }
    "unknown".to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_windows_version() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
fn get_macos_version() -> String {
    let output = std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok();
    if let Some(out) = output {
        return String::from_utf8_lossy(&out.stdout).trim().to_string();
    }
    "unknown".to_string()
}

#[cfg(not(target_os = "macos"))]
fn get_macos_version() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "linux")]
fn get_linux_version() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=")
                    .map(|v| v.trim_matches('"').to_string())
            })
        })
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(not(target_os = "linux"))]
fn get_linux_version() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "windows")]
fn get_locale() -> String {
    let output = std::process::Command::new("cmd")
        .args(["/C", "reg query \"HKCU\\Control Panel\\International\" /v LocaleName"])
        .output()
        .ok();
    if let Some(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if let Some(idx) = line.find("LocaleName") {
                let rest = &line[idx + "LocaleName".len()..];
                let locale = rest.split_whitespace().last().unwrap_or("").trim();
                if !locale.is_empty() && locale != "REG_SZ" {
                    return locale.replace('_', "-");
                }
            }
        }
    }
    "zh-CN".to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_locale() -> String {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .map(|l| {
            l.split('.').next().unwrap_or(&l).replace('_', "-")
        })
        .unwrap_or_else(|_| "en-US".to_string())
}

pub fn report_launch(app_handle: &tauri::AppHandle) {
    let data_dir = match app_handle.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[telemetry] 无法获取数据目录: {}", e);
            return;
        }
    };
    if !data_dir.exists() {
        let _ = fs::create_dir_all(&data_dir);
    }

    let install_id = get_or_create_install_id(&data_dir);
    let app_version = app_handle.config().version.clone().unwrap_or_default();
    let version_code = parse_version_code(&app_version);
    let (os, os_version) = get_os_info();
    let locale = get_locale();

    let payload = TrackingPayload {
        product: "fairy",
        app_version,
        version_code,
        install_id,
        channel: "stable",
        env: TrackingEnv {
            os,
            os_version,
            locale,
        },
    };

    tauri::async_runtime::spawn(async move {
        let client = Client::new();
        let result = client
            .post(TRACKING_URL)
            .header("X-Api-Key", API_KEY)
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;
        match result {
            Ok(resp) => {
                eprintln!("[telemetry] 上报成功, status: {}", resp.status());
            }
            Err(e) => {
                eprintln!("[telemetry] 上报失败: {}", e);
            }
        }
    });
}
