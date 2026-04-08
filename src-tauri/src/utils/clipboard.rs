// 剪贴板工具
use std::io::Write;
use std::process::{Command, Stdio};

/// 剪贴板操作结果
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardResult {
    Success,
    Failure(String),
    Unsupported,
}

/// 剪贴板管理器
pub struct ClipboardManager;

#[derive(Debug, Clone, PartialEq)]
enum Platform {
    Windows,
    MacOs,
    Linux,
    Unknown,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    pub fn new() -> Self {
        let _platform = if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            Platform::Unknown
        };

        Self
    }

    /// 将文本复制到剪贴板
    pub fn copy_text(&self, text: &str) -> ClipboardResult {
        #[cfg(target_os = "macos")]
        {
            self.copy_text_macos(text)
        }

        #[cfg(target_os = "linux")]
        {
            self.copy_text_linux(text)
        }

        #[cfg(target_os = "windows")]
        {
            self.copy_text_windows(text)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            ClipboardResult::Unsupported
        }
    }

    /// 从剪贴板获取文本
    pub fn get_text(&self) -> Result<String, String> {
        #[cfg(target_os = "macos")]
        {
            self.get_text_macos()
        }

        #[cfg(target_os = "linux")]
        {
            self.get_text_linux()
        }

        #[cfg(target_os = "windows")]
        {
            self.get_text_windows()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err("Unsupported platform".to_string())
        }
    }

    /// 清空剪贴板
    pub fn clear(&self) -> ClipboardResult {
        self.copy_text("")
    }

    // macOS 实现
    #[cfg(target_os = "macos")]
    fn copy_text_macos(&self, text: &str) -> ClipboardResult {
        let mut child = match Command::new("pbcopy")
            .env("LANG", "en_US.UTF-8")
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return ClipboardResult::Failure(e.to_string()),
        };

        if let Some(mut stdin) = child.stdin.take() {
            match stdin.write_all(text.as_bytes()) {
                Ok(_) => (),
                Err(e) => return ClipboardResult::Failure(e.to_string()),
            }
        }

        match child.wait() {
            Ok(_) => ClipboardResult::Success,
            Err(e) => ClipboardResult::Failure(e.to_string()),
        }
    }

    #[cfg(target_os = "macos")]
    fn get_text_macos(&self) -> Result<String, String> {
        let output = match Command::new("pbpaste").env("LANG", "en_US.UTF-8").output() {
            Ok(output) => output,
            Err(e) => return Err(e.to_string()),
        };

        if !output.status.success() {
            return Err("Command failed".to_string());
        }

        match String::from_utf8(output.stdout) {
            Ok(text) => Ok(text),
            Err(e) => Err(e.to_string()),
        }
    }

    // Linux 实现
    #[cfg(target_os = "linux")]
    fn copy_text_linux(&self, text: &str) -> ClipboardResult {
        // 尝试使用 xclip
        let result = self.copy_with_command(text, "xclip", &["-selection", "clipboard"]);
        if result == ClipboardResult::Success {
            return result;
        }

        // 尝试使用 xsel
        self.copy_with_command(text, "xsel", &["--clipboard", "--input"])
    }

    #[cfg(target_os = "linux")]
    fn get_text_linux(&self) -> Result<String, String> {
        // 尝试使用 xclip
        let result = self.get_with_command("xclip", &["-selection", "clipboard", "-o"]);
        if result.is_ok() {
            return result;
        }

        // 尝试使用 xsel
        self.get_with_command("xsel", &["--clipboard", "--output"])
    }

    #[cfg(target_os = "linux")]
    fn copy_with_command(&self, text: &str, command: &str, args: &[&str]) -> ClipboardResult {
        let mut child = match Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return ClipboardResult::Failure(format!("Command not found: {}", command)),
        };

        if let Some(mut stdin) = child.stdin.take() {
            match stdin.write_all(text.as_bytes()) {
                Ok(_) => (),
                Err(e) => return ClipboardResult::Failure(e.to_string()),
            }
        }

        match child.wait() {
            Ok(_) => ClipboardResult::Success,
            Err(e) => ClipboardResult::Failure(e.to_string()),
        }
    }

    #[cfg(target_os = "linux")]
    fn get_with_command(&self, command: &str, args: &[&str]) -> Result<String, String> {
        let output = match Command::new(command).args(args).output() {
            Ok(output) => output,
            Err(_) => return Err(format!("Command not found: {}", command)),
        };

        if !output.status.success() {
            return Err("Command failed".to_string());
        }

        match String::from_utf8(output.stdout) {
            Ok(text) => Ok(text),
            Err(e) => Err(e.to_string()),
        }
    }

    // Windows 实现
    #[cfg(target_os = "windows")]
    fn copy_text_windows(&self, text: &str) -> ClipboardResult {
        // Windows 实现会使用 winapi 或其他方式，这里提供一个简化版本
        // 实际项目中应该使用更合适的Windows剪贴板API
        let mut child = match Command::new("cmd")
            .args(["/c", "echo", text, "|", "clip"])
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return ClipboardResult::Failure(e.to_string()),
        };

        match child.wait() {
            Ok(_) => ClipboardResult::Success,
            Err(e) => ClipboardResult::Failure(e.to_string()),
        }
    }

    #[cfg(target_os = "windows")]
    fn get_text_windows(&self) -> Result<String, String> {
        // Windows 实现会使用 winapi 或其他方式
        // 实际项目中应该使用更合适的Windows剪贴板API
        Err("Not implemented for Windows".to_string())
    }

    // 默认实现（非特定平台）
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn copy_text_macos(&self, _text: &str) -> ClipboardResult {
        ClipboardResult::Unsupported
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn get_text_macos(&self) -> Result<String, String> {
        Err("Unsupported platform".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn copy_text_linux(&self, _text: &str) -> ClipboardResult {
        ClipboardResult::Unsupported
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn get_text_linux(&self) -> Result<String, String> {
        Err("Unsupported platform".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn copy_text_windows(&self, _text: &str) -> ClipboardResult {
        ClipboardResult::Unsupported
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn get_text_windows(&self) -> Result<String, String> {
        Err("Unsupported platform".to_string())
    }
}

/// 便捷函数：复制文本到剪贴板
#[allow(dead_code)]
pub fn copy_to_clipboard(text: &str) -> bool {
    let clipboard = ClipboardManager::new();
    clipboard.copy_text(text) == ClipboardResult::Success
}

/// 便捷函数：从剪贴板获取文本
#[allow(dead_code)]
pub fn get_from_clipboard() -> Option<String> {
    let clipboard = ClipboardManager::new();
    clipboard.get_text().ok()
}
