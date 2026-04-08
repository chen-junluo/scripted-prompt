// 模糊搜索实现
use crate::data::{Script, Template};
use regex::Regex;

/// 模糊搜索脚本
pub fn fuzzy_search_scripts<'a>(
    scripts: &'a [Script],
    query: &str,
    tag_filter: Option<&str>,
) -> Vec<&'a Script> {
    if query.is_empty() && tag_filter.is_none() {
        return scripts.iter().collect();
    }

    let results: Vec<&Script> = scripts
        .iter()
        .filter(|script| {
            // 标签过滤
            if let Some(tag) = tag_filter {
                if !script.matches_tags(tag) {
                    return false;
                }
            }

            // 关键词搜索
            if query.is_empty() {
                return true;
            }

            fuzzy_match(&script.name, query)
                || fuzzy_match(&script.tags, query)
                || fuzzy_match(&script.content, query)
        })
        .collect();

    // 简单排序：按使用次数和名称排序
    let mut sorted_results = results;
    sorted_results.sort_by(|a, b| {
        if a.use_count != b.use_count {
            b.use_count.cmp(&a.use_count)
        } else {
            a.name.cmp(&b.name)
        }
    });

    sorted_results
}

/// 模糊搜索模板
pub fn fuzzy_search_templates<'a>(
    templates: &'a [Template],
    query: &str,
    tag_filter: Option<&str>,
) -> Vec<&'a Template> {
    if query.is_empty() && tag_filter.is_none() {
        return templates.iter().collect();
    }

    let results: Vec<&Template> = templates
        .iter()
        .filter(|template| {
            // 标签过滤
            if let Some(tag) = tag_filter {
                if !template.tags.starts_with(tag) && template.tags != tag {
                    return false;
                }
            }

            // 关键词搜索
            if query.is_empty() {
                return true;
            }

            fuzzy_match(&template.name, query) ||
            fuzzy_match(&template.tags, query) ||
            // 搜索变量名和值
            template.variable_values.iter().any(|(key, value)| {
                fuzzy_match(key, query) || fuzzy_match(value, query)
            })
        })
        .collect();

    // 简单排序：按使用次数和名称排序
    let mut sorted_results = results;
    sorted_results.sort_by(|a, b| {
        if a.use_count != b.use_count {
            b.use_count.cmp(&a.use_count)
        } else {
            a.name.cmp(&b.name)
        }
    });

    sorted_results
}

/// 简单的模糊匹配算法
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    // 转换为小写进行大小写不敏感匹配
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    // 完全匹配
    if text_lower.contains(&pattern_lower) {
        return true;
    }

    // 单词匹配：检查pattern的每个单词是否都出现在text中
    let pattern_words: Vec<&str> = pattern_lower.split_whitespace().collect();
    if pattern_words.len() > 1 {
        return pattern_words.iter().all(|word| text_lower.contains(word));
    }

    // 首字母匹配
    let text_words: Vec<&str> = text_lower.split_whitespace().collect();
    for word in text_words {
        if word.starts_with(&pattern_lower) {
            return true;
        }
    }

    false
}

/// 按标签层次过滤脚本
pub fn filter_scripts_by_tag_hierarchy<'a>(
    scripts: &'a [Script],
    tag_prefix: &str,
) -> Vec<&'a Script> {
    scripts
        .iter()
        .filter(|script| {
            if tag_prefix.is_empty() {
                return true;
            }

            // 精确匹配完整标签路径
            script.tags == tag_prefix ||
            // 匹配标签前缀
            script.tags.starts_with(&format!("{}/", tag_prefix))
        })
        .collect()
}

/// 搜索并高亮匹配的文本
pub fn highlight_match(text: &str, query: &str) -> String {
    if query.is_empty() {
        return text.to_string();
    }

    let pattern = regex::escape(query);
    let re = Regex::new(&format!("({})", pattern)).expect("Invalid regex pattern");

    re.replace_all(text, "[$1]").to_string()
}
