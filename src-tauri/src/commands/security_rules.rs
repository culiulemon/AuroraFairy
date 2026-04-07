use std::path::PathBuf;
use std::fs;

fn get_security_rules_path() -> Result<PathBuf, String> {
    let root = crate::get_project_root()?;
    Ok(root.join("security-rules.json"))
}

pub fn load_security_rules() -> Result<String, String> {
    let path = get_security_rules_path()?;
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&path).map_err(|e| format!("读取安全规则文件失败: {}", e))
}

pub fn save_security_rules(content: String) -> Result<(), String> {
    let path = get_security_rules_path()?;
    fs::write(&path, &content).map_err(|e| format!("写入安全规则文件失败: {}", e))
}

pub fn delete_security_rules() -> Result<(), String> {
    let path = get_security_rules_path()?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("删除安全规则文件失败: {}", e))?;
    }
    Ok(())
}
