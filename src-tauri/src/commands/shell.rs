use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[cfg(target_os = "windows")]
const SHELL: &[&str] = &["cmd", "/C"];
#[cfg(not(target_os = "windows"))]
const SHELL: &[&str] = &["sh", "-c"];

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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

pub async fn shell_execute(command: String, timeout_secs: u64) -> Result<String, String> {
    if command.trim().is_empty() {
        return Err("命令不能为空".to_string());
    }

    for pattern in BLOCKED_PATTERNS {
        if command.to_lowercase().contains(&pattern.to_lowercase()) {
            return Err(format!("禁止执行危险命令: {}", pattern));
        }
    }

    let mut cmd = Command::new(SHELL[0]);
    cmd.arg(SHELL[1])
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = timeout(
        Duration::from_secs(timeout_secs),
        cmd.output()
    )
    .await;

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
        assert!(shell_execute("rm -rf /".to_string(), 1).await.is_err());
        assert!(shell_execute("mkfs".to_string(), 1).await.is_err());
    }

    #[tokio::test]
    async fn test_valid_command() {
        let result = shell_execute("echo hello".to_string(), 5).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }
}
