// 工具函数模块
pub mod clipboard;
pub mod validator;

/// 格式化时间戳
pub fn format_timestamp(timestamp: &chrono::DateTime<chrono::Local>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 格式化文件大小
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// 生成唯一标识符
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 安全地读取文件内容
pub fn safe_read_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// 安全地写入文件内容
pub fn safe_write_file(path: &std::path::Path, content: &str) -> Result<(), std::io::Error> {
    use std::io::Write;

    // 确保目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// 错误处理工具函数
pub fn format_error(error: &dyn std::error::Error) -> String {
    let mut message = String::new();
    let mut current = Some(error);
    let mut depth = 0;

    while let Some(err) = current {
        if depth > 0 {
            message.push_str("\nCaused by: ");
        }
        message.push_str(&err.to_string());
        current = err.source();
        depth += 1;

        // 限制错误链深度
        if depth > 5 {
            break;
        }
    }

    message
}
