use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;

const COREFILE_NAMES: &[&str] = &[
    "SOUL.md", "HABIT.md", "SYSPROMPT.md", "ABOUTUSER.md", "REBIRTH.md",
];

fn is_corefile_path(path: &std::path::PathBuf) -> bool {
    let path_str = path.to_string_lossy();
    if !path_str.contains("corefile") {
        return false;
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        return COREFILE_NAMES.contains(&name);
    }
    false
}

pub fn file_read(path: &str, offset: Option<usize>, limit: Option<usize>, working_dir: &str, data_dir: Option<&str>, extra_allowed_paths: Option<&[String]>) -> Result<String, String> {
    let safe_path = validate_path(path, working_dir, data_dir, extra_allowed_paths)?;
    
    let content = std::fs::read_to_string(&safe_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    
    let start = match offset {
        Some(o) => o.saturating_sub(1).min(total_lines),
        None => 0,
    };
    
    let end = match limit {
        Some(l) => (start + l).min(total_lines),
        None => total_lines,
    };
    
    let selected: Vec<&str> = lines[start..end].to_vec();
    
    let width = total_lines.to_string().len();
    let result: Vec<String> = selected
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{:>width$}→{}", start + i + 1, line, width = width))
        .collect();
    
    Ok(result.join("\n"))
}

pub fn file_write(path: &str, content: &str, working_dir: &str, data_dir: Option<&str>, extra_allowed_paths: Option<&[String]>) -> Result<(), String> {
    let safe_path = validate_path(path, working_dir, data_dir, extra_allowed_paths)?;
    
    if is_corefile_path(&safe_path) {
        return Err("COREFILE_PROTECTED:核心文件受保护，禁止通过工具写入。请使用角色设置页面修改。".to_string());
    }
    
    if let Some(parent) = safe_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    std::fs::write(&safe_path, content)
        .map_err(|e| format!("写入文件失败: {}", e))
}

pub fn file_delete(path: &str, working_dir: &str, data_dir: Option<&str>, extra_allowed_paths: Option<&[String]>) -> Result<(), String> {
    let safe_path = validate_path(path, working_dir, data_dir, extra_allowed_paths)?;
    
    if is_corefile_path(&safe_path) {
        return Err("COREFILE_PROTECTED:核心文件受保护，禁止通过工具删除。请使用角色设置页面管理。".to_string());
    }
    
    if !safe_path.exists() {
        return Err("文件不存在".to_string());
    }
    
    if safe_path.is_dir() {
        std::fs::remove_dir_all(&safe_path)
            .map_err(|e| format!("删除目录失败: {}", e))
    } else {
        std::fs::remove_file(&safe_path)
            .map_err(|e| format!("删除文件失败: {}", e))
    }
}

pub fn file_edit(path: &str, old_str: &str, new_str: &str, working_dir: &str, data_dir: Option<&str>, extra_allowed_paths: Option<&[String]>) -> Result<String, String> {
    let safe_path = validate_path(path, working_dir, data_dir, extra_allowed_paths)?;
    
    if is_corefile_path(&safe_path) {
        return Err("COREFILE_PROTECTED:核心文件受保护，禁止通过工具编辑。请使用角色设置页面修改。".to_string());
    }
    
    let content = std::fs::read_to_string(&safe_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    if !content.contains(old_str) {
        return Err(format!("未找到要替换的内容"));
    }
    
    let count = content.matches(old_str).count();
    if count > 1 {
        return Err(format!("发现 {} 处匹配，请提供更多上下文确保唯一性", count));
    }
    
    let new_content = content.replacen(old_str, new_str, 1);
    
    std::fs::write(&safe_path, &new_content)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(format!("成功替换 1 处"))
}

pub fn file_glob(pattern: &str, working_dir: &str, data_dir: Option<&str>) -> Result<Vec<String>, String> {
    let (base_path, glob_pattern): (std::path::PathBuf, String) = if Path::new(pattern).is_absolute() {
        let mut base_components = Vec::new();
        let mut glob_parts = Vec::new();
        let mut found_glob = false;
        for component in Path::new(pattern).components() {
            if !found_glob {
                let s = component.as_os_str().to_string_lossy();
                if s.contains('*') || s.contains('?') {
                    found_glob = true;
                    glob_parts.push(s.to_string());
                } else {
                    base_components.push(component);
                }
            } else {
                glob_parts.push(component.as_os_str().to_string_lossy().to_string());
            }
        }
        let base: std::path::PathBuf = base_components.into_iter().collect();
        let gp = glob_parts.join(&std::path::MAIN_SEPARATOR.to_string());
        let base_str = base.to_string_lossy().to_string();
        let working_dir_normalized = Path::new(working_dir).canonicalize()
            .unwrap_or_else(|_| Path::new(working_dir).to_path_buf())
            .to_string_lossy().to_string();
        let is_in_working_dir = base_str == working_dir_normalized
            || base_str.starts_with(&format!("{}{}", working_dir_normalized, std::path::MAIN_SEPARATOR));
        let is_in_data_dir = data_dir.map_or(false, |dd| {
            let dd_normalized = Path::new(dd).canonicalize()
                .unwrap_or_else(|_| Path::new(dd).to_path_buf())
                .to_string_lossy().to_string();
            base_str == dd_normalized
                || base_str.starts_with(&format!("{}{}", dd_normalized, std::path::MAIN_SEPARATOR))
        });
        if !is_in_working_dir && !is_in_data_dir {
            return Err(format!("glob 路径不在允许的目录范围内: {}", base_str));
        }
        (base, if gp.is_empty() { "*".to_string() } else { gp })
    } else {
        (Path::new(working_dir).to_path_buf(), pattern.to_string())
    };

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    for entry in WalkDir::new(&base_path)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let relative = path.strip_prefix(&base_path)
            .unwrap_or(path)
            .to_string_lossy();

        if matches_glob(&relative, &glob_pattern) {
            results.push(path.to_string_lossy().to_string());
        }
    }

    Ok(results)
}

pub fn file_grep(path: &str, pattern: &str, working_dir: &str, data_dir: Option<&str>, extra_allowed_paths: Option<&[String]>) -> Result<Vec<String>, String> {
    let search_dir = if path == "." || path.is_empty() {
        Path::new(working_dir).to_path_buf()
    } else {
        validate_path(path, working_dir, data_dir, extra_allowed_paths)?
    };

    if !search_dir.exists() {
        return Err("搜索路径不存在".to_string());
    }

    let regex = Regex::new(pattern)
        .map_err(|e| format!("无效的正则表达式: {}", e))?;

    let mut results = Vec::new();

    for entry in WalkDir::new(&search_dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();
        
        if file_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (line_num, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        let relative = file_path.strip_prefix(&search_dir)
                            .unwrap_or(file_path)
                            .to_string_lossy();
                        results.push(format!("{}:{}:{}", relative, line_num + 1, line));
                    }
                }
            }
        }
    }

    Ok(results)
}

pub fn validate_path(path: &str, working_dir: &str, data_dir: Option<&str>, extra_allowed_paths: Option<&[String]>) -> Result<std::path::PathBuf, String> {
    if path.contains("..") {
        return Err("禁止路径穿越".to_string());
    }

    let working = Path::new(working_dir);
    let full_path = if Path::new(path).is_absolute() {
        Path::new(path).to_path_buf()
    } else {
        working.join(path)
    };

    let working_canonical = working.canonicalize()
        .map_err(|e| format!("工作目录无效: {}", e))?;

    let data_canonical = if let Some(dd) = data_dir {
        Path::new(dd).canonicalize().ok()
    } else {
        None
    };

    let check_allowed_path = |canonical: &std::path::PathBuf| -> bool {
        if canonical.starts_with(&working_canonical) {
            return true;
        }
        if let Some(ref dc) = data_canonical {
            if canonical.starts_with(dc) {
                return true;
            }
        }
        if let Some(allowed) = extra_allowed_paths {
            for allowed_path_str in allowed {
                if let Ok(allowed_canonical) = Path::new(allowed_path_str).canonicalize() {
                    if canonical.starts_with(&allowed_canonical) {
                        return true;
                    }
                } else {
                    let allowed_path = Path::new(allowed_path_str);
                    if allowed_path.is_dir() && canonical.starts_with(allowed_path) {
                        return true;
                    }
                    if allowed_path.file_name().is_some() && canonical == allowed_path {
                        return true;
                    }
                }
            }
        }
        false
    };

    if full_path.exists() {
        let canonical = full_path.canonicalize()
            .map_err(|e| format!("路径解析失败: {}", e))?;
        if !check_allowed_path(&canonical) {
            return Err(format!("OUT_OF_WORKDIR:{}", canonical.to_string_lossy()));
        }
        Ok(canonical)
    } else {
        let resolved = full_path;
        if resolved.starts_with(&working_canonical) {
            return Ok(resolved);
        }
        if let Some(ref dc) = data_canonical {
            if resolved.starts_with(dc) {
                return Ok(resolved);
            }
        }
        if let Some(allowed) = extra_allowed_paths {
            for allowed_path_str in allowed {
                let allowed_path = Path::new(allowed_path_str);
                if resolved.starts_with(allowed_path) {
                    return Ok(resolved);
                }
            }
        }
        let mut check = resolved.as_path();
        let mut found_valid = false;
        while let Some(parent) = check.parent() {
            if parent.as_os_str().is_empty() {
                break;
            }
            if let Ok(parent_canonical) = parent.canonicalize() {
                if check_allowed_path(&parent_canonical) {
                    found_valid = true;
                }
                break;
            }
            check = parent;
        }
        if !found_valid {
            return Err(format!("OUT_OF_WORKDIR:{}", resolved.to_string_lossy()));
        }
        Ok(resolved)
    }
}

fn matches_glob(path: &str, pattern: &str) -> bool {
    let pattern = pattern.trim_start_matches("**/");
    
    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        if let Some(path_ext) = Path::new(path).extension() {
            return path_ext.to_string_lossy() == ext;
        }
        return false;
    }

    if pattern.contains('*') {
        let regex_pattern = pattern
            .replace("**/", ".*/")
            .replace("**", ".*")
            .replace(".", "\\.")
            .replace("*", "[^/]*");
        
        if let Ok(re) = Regex::new(&format!("^{}$", regex_pattern)) {
            return re.is_match(path);
        }
    }

    path.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_blocks_parent() {
        let dir = std::env::temp_dir().join("aurora_test_workspace");
        std::fs::create_dir_all(&dir).ok();
        let result = validate_path("../etc/passwd", dir.to_str().unwrap(), None, None);
        assert!(result.is_err());
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_validate_path_out_of_workdir_error_format() {
        let dir = std::env::temp_dir().join("aurora_test_workspace2");
        std::fs::create_dir_all(&dir).ok();
        let outside = std::env::temp_dir().join("aurora_test_outside");
        std::fs::create_dir_all(&outside).ok();
        let outside_file = outside.join("secret.txt");
        std::fs::write(&outside_file, "secret").ok();

        let result = validate_path(
            outside_file.to_str().unwrap(),
            dir.to_str().unwrap(),
            None,
            None
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.starts_with("OUT_OF_WORKDIR:"), "错误应以 OUT_OF_WORKDIR: 开头，实际: {}", err);
        std::fs::remove_file(&outside_file).ok();
        std::fs::remove_dir(&outside).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_validate_path_extra_allowed_allows_path() {
        let dir = std::env::temp_dir().join("aurora_test_workspace3");
        std::fs::create_dir_all(&dir).ok();
        let allowed_dir = std::env::temp_dir().join("aurora_test_allowed");
        std::fs::create_dir_all(&allowed_dir).ok();
        let test_file = allowed_dir.join("test.txt");
        std::fs::write(&test_file, "hello").ok();

        let result = validate_path(
            test_file.to_str().unwrap(),
            dir.to_str().unwrap(),
            None,
            Some(&[allowed_dir.to_string_lossy().to_string()])
        );
        assert!(result.is_ok(), "应该允许额外路径中的文件: {:?}", result);
        std::fs::remove_file(&test_file).ok();
        std::fs::remove_dir(&allowed_dir).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
