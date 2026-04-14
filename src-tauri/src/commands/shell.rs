use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[cfg(target_os = "windows")]
const DEFAULT_SHELL: &[&str] = &["cmd", "/C"];
#[cfg(not(target_os = "windows"))]
const DEFAULT_SHELL: &[&str] = &["sh", "-c"];

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn resolve_shell(shell_type: &Option<String>) -> Result<Vec<&str>, String> {
    match shell_type.as_deref() {
        None | Some("default") => Ok(DEFAULT_SHELL.to_vec()),
        Some("cmd") => Ok(vec!["cmd", "/C"]),
        Some("powershell") | Some("pwsh") => Ok(vec!["powershell", "-Command"]),
        Some("bash") => {
            #[cfg(target_os = "windows")]
            {
                for candidate in &["bash", "git", "C:\\Program Files\\Git\\bin\\bash.exe", "C:\\Program Files (x86)\\Git\\bin\\bash.exe"] {
                    if which_shell(candidate).is_some() {
                        return Ok(vec![which_shell(candidate).unwrap(), "-c"]);
                    }
                }
                Err("bash 不可用: 未找到 bash 或 Git Bash，请安装 Git for Windows".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            { Ok(vec!["bash", "-c"]) }
        }
        Some("sh") => {
            #[cfg(target_os = "windows")]
            {
                Err("sh 不可用: Windows 上不支持 sh，请使用 bash (需安装 Git Bash)".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            { Ok(vec!["sh", "-c"]) }
        }
        Some(other) => Err(format!("不支持的 shell 类型: {}，可选: default, cmd, powershell, bash, sh", other)),
    }
}

#[cfg(target_os = "windows")]
fn which_shell(name: &str) -> Option<&str> {
    use std::path::Path;
    if Path::new(name).exists() {
        return Some(name);
    }
    if which_binary_exists(name) {
        return Some(name);
    }
    None
}

#[cfg(target_os = "windows")]
fn which_binary_exists(name: &str) -> bool {
    std::process::Command::new("where")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    ":(){ :|:& };:",
    "chmod -R 777 /",
    "dd if=/dev/zero of=/dev/sda",
    "> /dev/sda",
    "wget.*curl.*sh",
];

fn decode_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            #[cfg(target_os = "windows")]
            {
                let (decoded, _, had_errors) = encoding_rs::GBK.decode(bytes);
                if had_errors {
                    let fallback = String::from_utf8_lossy(bytes).to_string();
                    fallback
                } else {
                    decoded.into_owned()
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                String::from_utf8_lossy(bytes).to_string()
            }
        }
    }
}

fn needs_cmd_percent_escape(shell_parts: &[&str]) -> bool {
    shell_parts.get(0).map(|s| *s == "cmd").unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn write_temp_bat(command: &str) -> Result<std::path::PathBuf, String> {
    let temp_dir = std::env::temp_dir();
    let file_name = format!("aurora_shell_{}.bat", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());
    let bat_path = temp_dir.join(&file_name);
    let escaped = command.replace('%', "%%");
    std::fs::write(&bat_path, escaped.as_bytes())
        .map_err(|e| format!("写入临时批处理文件失败: {}", e))?;
    Ok(bat_path)
}

pub async fn shell_execute(command: String, timeout_secs: u64, shell_type: Option<String>) -> Result<String, String> {
    if command.trim().is_empty() {
        return Err("命令不能为空".to_string());
    }

    for pattern in BLOCKED_PATTERNS {
        if command.to_lowercase().contains(&pattern.to_lowercase()) {
            return Err(format!("禁止执行危险命令: {}", pattern));
        }
    }

    let shell_parts = resolve_shell(&shell_type)?;
    let has_percent = command.contains('%') && needs_cmd_percent_escape(&shell_parts);

    #[cfg(target_os = "windows")]
    let mut _temp_bat_to_clean: Option<std::path::PathBuf> = None;

    let mut cmd = Command::new(shell_parts[0]);
    if shell_parts.len() > 1 {
        cmd.arg(shell_parts[1]);
    }

    #[cfg(target_os = "windows")]
    {
        if has_percent {
            let bat_path = write_temp_bat(&command)?;
            let bat_str = bat_path.to_string_lossy().to_string();
            cmd.arg(&bat_str);
            _temp_bat_to_clean = Some(bat_path);
        } else {
            cmd.arg(&command);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd.arg(&command);
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = timeout(
        Duration::from_secs(timeout_secs),
        cmd.output()
    )
    .await;

    #[cfg(target_os = "windows")]
    if let Some(ref bat) = _temp_bat_to_clean {
        let _ = std::fs::remove_file(bat);
    }

    match output {
        Ok(Ok(output)) => {
            let stdout = decode_output(&output.stdout);
            let stderr = decode_output(&output.stderr);

            if output.status.success() {
                Ok(stdout)
            } else {
                if !stderr.is_empty() {
                    Err(format!("命令执行失败: {}\n{}", stdout, stderr))
                } else {
                    Err(format!("命令执行失败，退出码: {:?}", output.status.code()))
                }
            }
        }
        Ok(Err(e)) => Err(format!("命令执行错误: {}", e)),
        Err(_) => Err(format!("命令执行超时 ({} 秒)", timeout_secs)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blocked_commands() {
        assert!(shell_execute("rm -rf /".to_string(), 1, None).await.is_err());
        assert!(shell_execute("mkfs".to_string(), 1, None).await.is_err());
    }

    #[tokio::test]
    async fn test_valid_command() {
        let result = shell_execute("echo hello".to_string(), 5, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }
}
