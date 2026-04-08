// 变量解析、验证、替换逻辑
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 变量格式：{{variable_name}} 或 {{variable_name:default_value}}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub default_value: Option<String>,
}

/// 从内容中解析所有变量，返回变量名列表
pub fn parse_variables(content: &str) -> Vec<String> {
    let re = Regex::new(r"\{\{\s*([^}:]+)(?::([^}]+))?\s*\}\}").expect("Invalid regex pattern");
    let mut variables = Vec::new();
    let mut seen = HashMap::new();

    for cap in re.captures_iter(content) {
        let name = cap[1].trim().to_string();

        // 避免重复添加相同的变量
        if !seen.contains_key(&name) {
            variables.push(name.clone());
            seen.insert(name, true);
        }
    }

    variables
}

/// 从内容中解析所有变量（包含默认值信息）
pub fn parse_variables_with_defaults(content: &str) -> Vec<Variable> {
    let re = Regex::new(r"\{\{\s*([^}:]+)(?::([^}]+))?\s*\}\}").expect("Invalid regex pattern");
    let mut variables = Vec::new();
    let mut seen = HashMap::new();

    for cap in re.captures_iter(content) {
        let name = cap[1].trim().to_string();
        let default_value = cap.get(2).map(|m| m.as_str().trim().to_string());

        // 避免重复添加相同的变量
        if !seen.contains_key(&name) {
            variables.push(Variable {
                name: name.clone(),
                default_value,
            });
            seen.insert(name, true);
        }
    }

    variables
}

/// 替换内容中的变量，支持自定义回退函数
pub fn replace_variables<F>(
    content: &str,
    variable_values: &HashMap<String, String>,
    fallback: F,
) -> String
where
    F: Fn(&str) -> String,
{
    let re = Regex::new(r"\{\{\s*([^}:]+)(?::([^}]+))?\s*\}\}").expect("Invalid regex pattern");

    re.replace_all(content, |caps: &regex::Captures| {
        let variable_name = caps[1].trim();

        // 尝试从提供的值中获取
        if let Some(value) = variable_values.get(variable_name) {
            value.to_string()
        } else {
            // 尝试使用默认值
            if let Some(default_value) = caps.get(2) {
                default_value.as_str().trim().to_string()
            } else {
                // 使用回退函数
                fallback(variable_name)
            }
        }
    })
    .to_string()
}

/// 验证变量名是否有效（支持 Unicode 和特殊字符，符合需求）
pub fn validate_variable_name(name: &str) -> bool {
    // 根据需求文档，变量命名不限制字符（支持Unicode、特殊字符等）
    // 只要不是空字符串且不包含 : 和 } 即可
    !name.is_empty() && !name.contains(':') && !name.contains('}')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn parse_variables_deduplicates_names() {
        let content = "Hello {{ name }} and {{name}} and {{project}}";
        let variables = parse_variables(content);

        assert_eq!(variables, vec!["name".to_string(), "project".to_string()]);
    }

    #[test]
    fn parse_variables_with_defaults_extracts_default_values() {
        let content = "Hello {{name}} from {{project:Scripted Prompt}}";
        let variables = parse_variables_with_defaults(content);

        assert_eq!(variables.len(), 2);
        assert_eq!(variables[0].name, "name");
        assert_eq!(variables[0].default_value, None);
        assert_eq!(variables[1].name, "project");
        assert_eq!(
            variables[1].default_value.as_deref(),
            Some("Scripted Prompt")
        );
    }

    #[test]
    fn replace_variables_prefers_provided_values_then_defaults_then_fallback() {
        let content = "Hello {{name}}, welcome to {{project:Scripted Prompt}} in {{city}}.";
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Dylan".to_string());

        let replaced = replace_variables(content, &values, |var| format!("<{}>", var));

        assert_eq!(
            replaced,
            "Hello Dylan, welcome to Scripted Prompt in <city>."
        );
    }

    #[test]
    fn validate_variable_name_allows_unicode_and_special_characters_except_forbidden_ones() {
        assert!(validate_variable_name("变量名-测试_123"));
        assert!(validate_variable_name("project name"));
        assert!(!validate_variable_name(""));
        assert!(!validate_variable_name("bad:name"));
        assert!(!validate_variable_name("bad}name"));
    }
}
