// 变量格式验证工具
use crate::logic::variable::validate_variable_name;

/// 验证结果枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

/// 验证器结构体
#[derive(Debug, Clone)]
pub struct Validator {
    // 这里可以添加配置项
}

impl Validator {
    /// 创建新的验证器实例
    pub fn new() -> Self {
        Validator {}
    }

    /// 验证变量格式
    pub fn validate_variable(&self, variable: &str) -> ValidationResult {
        if !variable.starts_with("{{") || !variable.ends_with("}}") {
            return ValidationResult::Invalid(
                "变量格式无效。请使用 {{variable_name}} 或 {{variable_name:default_value}} 格式。"
                    .to_string(),
            );
        }

        let inner = &variable[2..variable.len() - 2];
        let trimmed = inner.trim();
        if trimmed.is_empty() {
            return ValidationResult::Invalid("变量名不能为空".to_string());
        }

        let name = trimmed
            .split_once(':')
            .map(|(name, _)| name)
            .unwrap_or(trimmed)
            .trim();
        if !validate_variable_name(name) {
            return ValidationResult::Invalid(
                "变量格式无效。变量名不能为空，且不能包含 : 或 } 字符。".to_string(),
            );
        }

        ValidationResult::Valid
    }

    /// 验证变量名
    #[allow(dead_code)]
    pub fn validate_name(&self, name: &str) -> ValidationResult {
        if name.trim().is_empty() {
            return ValidationResult::Invalid("名称不能为空".to_string());
        }

        if name.contains('<')
            || name.contains('>')
            || name.contains('&')
            || name.contains('"')
            || name.contains('\\')
        {
            return ValidationResult::Invalid("名称包含非法字符".to_string());
        }

        if name.len() > 100 {
            return ValidationResult::Invalid("名称长度不能超过100个字符".to_string());
        }

        ValidationResult::Valid
    }

    /// 验证标签格式
    #[allow(dead_code)]
    pub fn validate_tags(&self, tags: &str) -> ValidationResult {
        let tag_segments: Vec<&str> = tags.split('/').collect();

        for segment in tag_segments {
            if !segment.is_empty() {
                match self.validate_name(segment) {
                    ValidationResult::Invalid(msg) => {
                        return ValidationResult::Invalid(format!(
                            "标签段'{}'无效: {}",
                            segment, msg
                        ));
                    }
                    _ => continue,
                }
            }
        }

        ValidationResult::Valid
    }
}

/// 检查未闭合的变量
#[allow(dead_code)]
pub fn check_unclosed_variables(content: &str) -> Vec<String> {
    let mut unclosed = Vec::new();
    let mut in_variable = false;
    let mut variable_start = 0;

    for (i, c) in content.char_indices() {
        if in_variable {
            if c == '}' && i + 1 < content.len() && content.chars().nth(i + 1) == Some('}') {
                in_variable = false;
            }
        } else if c == '{' && i + 1 < content.len() && content.chars().nth(i + 1) == Some('{') {
            in_variable = true;
            variable_start = i;
        }
    }

    if in_variable {
        unclosed.push(content[variable_start..].to_string());
    }

    unclosed
}
