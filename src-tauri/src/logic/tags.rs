// 标签管理（嵌套、重命名、级联删除）
use crate::data::{Script, Template};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 标签树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagNode {
    pub name: String,
    pub full_path: String,
    pub children: HashMap<String, TagNode>,
    pub usage_count: usize,
}

impl TagNode {
    pub fn new(name: String, full_path: String) -> Self {
        Self {
            name,
            full_path,
            children: HashMap::new(),
            usage_count: 0,
        }
    }
}

/// 标签管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagManager {
    root: TagNode,
    all_tags: HashMap<String, usize>, // 所有标签及其使用次数
}

impl Default for TagManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TagManager {
    pub fn new() -> Self {
        Self {
            root: TagNode::new("root".to_string(), "".to_string()),
            all_tags: HashMap::new(),
        }
    }

    /// 从脚本和模板中构建标签树
    pub fn build_tag_tree(&mut self, scripts: &[Script], templates: &[Template]) {
        // 重置标签树
        self.root = TagNode::new("root".to_string(), "".to_string());
        self.all_tags.clear();

        // 添加脚本标签
        for script in scripts {
            self.add_tag_path(&script.tags);
        }

        // 添加模板标签
        for template in templates {
            self.add_tag_path(&template.tags);
        }
    }

    /// 从 Script 更新标签树
    pub fn update_from_scripts(&mut self, scripts: &[Script]) {
        self.root = TagNode::new("root".to_string(), "".to_string());
        self.all_tags.clear();

        for script in scripts {
            self.add_tag_path(&script.tags);
        }
    }

    /// 从 Template 更新标签树
    pub fn update_from_templates(&mut self, templates: &[Template]) {
        for template in templates {
            self.add_tag_path(&template.tags);
        }
    }

    /// 添加标签路径到树中
    fn add_tag_path(&mut self, tag_path: &str) {
        if tag_path.is_empty() {
            return;
        }

        // 更新使用次数
        *self.all_tags.entry(tag_path.to_string()).or_insert(0) += 1;

        // 分割路径
        let parts: Vec<&str> = tag_path.split('/').collect();
        if parts.is_empty() {
            return;
        }

        let mut current = &mut self.root;
        let mut current_path = String::new();

        for part in parts {
            if part.is_empty() {
                continue;
            }

            // 更新当前路径
            if current_path.is_empty() {
                current_path = part.to_string();
            } else {
                current_path = format!("{}/{}", current_path, part);
            }

            // 获取或创建子节点
            current = current
                .children
                .entry(part.to_string())
                .or_insert_with(|| TagNode::new(part.to_string(), current_path.clone()));

            // 增加使用次数
            current.usage_count += 1;
        }
    }

    /// 获取所有标签
    pub fn get_all_tags(&self) -> Vec<(String, usize)> {
        let mut tags: Vec<(String, usize)> = self
            .all_tags
            .iter()
            .map(|(tag, count)| (tag.clone(), *count))
            .collect();

        // 按使用次数降序排序
        tags.sort_by(|a, b| b.1.cmp(&a.1));

        tags
    }

    /// 获取标签的子标签
    pub fn get_child_tags(&self, parent_tag: &str) -> Vec<(String, usize)> {
        let mut result = Vec::new();

        // 查找父标签节点
        let parent_path = if parent_tag.is_empty() {
            &self.root
        } else {
            let parts: Vec<&str> = parent_tag.split('/').collect();
            let mut current = &self.root;

            for part in parts {
                if let Some(child) = current.children.get(part) {
                    current = child;
                } else {
                    return result;
                }
            }

            current
        };

        // 收集子标签
        for (_name, node) in &parent_path.children {
            result.push((node.full_path.clone(), node.usage_count));
        }

        // 按使用次数排序
        result.sort_by(|a, b| b.1.cmp(&a.1));

        result
    }

    /// 重命名标签段（路径段精确匹配，符合需求文档）
    pub fn rename_tag_segment(
        &mut self,
        old_segment: &str,
        new_segment: &str,
        scripts: &mut [Arc<Script>],
        templates: &mut [Arc<Template>],
    ) -> bool {
        if old_segment == new_segment || old_segment.is_empty() {
            return false;
        }

        // 更新脚本标签 - 使用路径段匹配
        for script in scripts.iter_mut() {
            let new_tags = Self::rename_tag_segment_in_path(&script.tags, old_segment, new_segment);
            if new_tags != script.tags {
                let mut new_script = (**script).clone();
                new_script.tags = new_tags;
                *script = Arc::new(new_script);
            }
        }

        // 更新模板标签 - 使用路径段匹配
        for template in templates.iter_mut() {
            let new_tags =
                Self::rename_tag_segment_in_path(&template.tags, old_segment, new_segment);
            if new_tags != template.tags {
                let mut new_template = (**template).clone();
                new_template.tags = new_tags;
                *template = Arc::new(new_template);
            }
        }

        // 重建标签树
        let script_refs: Vec<&Script> = scripts.iter().map(|s| &**s).collect();
        let template_refs: Vec<&Template> = templates.iter().map(|t| &**t).collect();
        self.build_tag_tree_from_refs(&script_refs, &template_refs);

        true
    }

    /// 在标签路径中重命名特定段（精确匹配）
    fn rename_tag_segment_in_path(tag_path: &str, old_segment: &str, new_segment: &str) -> String {
        tag_path
            .split('/')
            .map(|segment| {
                if segment == old_segment {
                    new_segment
                } else {
                    segment
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// 从引用构建标签树
    fn build_tag_tree_from_refs(&mut self, scripts: &[&Script], templates: &[&Template]) {
        // 重置标签树
        self.root = TagNode::new("root".to_string(), "".to_string());
        self.all_tags.clear();

        // 添加脚本标签
        for script in scripts {
            self.add_tag_path(&script.tags);
        }

        // 添加模板标签
        for template in templates {
            self.add_tag_path(&template.tags);
        }
    }

    /// 旧的重命名方法（保持向后兼容，但标记为已弃用）
    #[deprecated(note = "Use rename_tag_segment instead for path segment matching")]
    pub fn rename_tag(
        &mut self,
        old_tag: &str,
        new_tag: &str,
        scripts: &mut [Script],
        templates: &mut [Template],
    ) -> bool {
        if old_tag == new_tag || !self.all_tags.contains_key(old_tag) {
            return false;
        }

        // 更新脚本标签
        for script in scripts.iter_mut() {
            if script.tags == old_tag {
                script.tags = new_tag.to_string();
            } else if script.tags.starts_with(&format!("{}/", old_tag)) {
                // 更新子标签
                script.tags = format!("{}{}", new_tag, &script.tags[old_tag.len()..]);
            }
        }

        // 更新模板标签
        for template in templates.iter_mut() {
            if template.tags == old_tag {
                template.tags = new_tag.to_string();
            } else if template.tags.starts_with(&format!("{}/", old_tag)) {
                // 更新子标签
                template.tags = format!("{}{}", new_tag, &template.tags[old_tag.len()..]);
            }
        }

        // 重建标签树
        self.build_tag_tree(scripts, templates);

        true
    }

    /// 级联删除标签（同时删除所有子孙标签）
    pub fn cascade_delete_tag(
        &mut self,
        tag: &str,
        scripts: &mut Vec<Arc<Script>>,
        templates: &mut Vec<Arc<Template>>,
    ) -> bool {
        if !self.all_tags.contains_key(tag) {
            return false;
        }

        // 删除脚本中的标签（及其子标签）
        scripts.retain(|script| {
            let keep = !Self::is_tag_descendant(&script.tags, tag);
            keep
        });

        // 删除模板中的标签（及其子标签）
        templates.retain(|template| {
            let keep = !Self::is_tag_descendant(&template.tags, tag);
            keep
        });

        // 重建标签树
        let script_refs: Vec<&Script> = scripts.iter().map(|s| &**s).collect();
        let template_refs: Vec<&Template> = templates.iter().map(|t| &**t).collect();
        self.build_tag_tree_from_refs(&script_refs, &template_refs);

        true
    }

    /// 检查标签是否为某个父标签的后代
    fn is_tag_descendant(tag: &str, parent: &str) -> bool {
        tag == parent || tag.starts_with(&format!("{}/", parent))
    }

    /// 计算重命名影响范围（用于预览）
    pub fn calculate_rename_impact(
        &self,
        old_segment: &str,
        scripts: &[Arc<Script>],
        templates: &[Arc<Template>],
    ) -> RenameImpact {
        let mut affected_script_tags = HashSet::new();
        let mut affected_script_count = 0;

        for script in scripts {
            if script.tags.split('/').any(|seg| seg == old_segment) {
                affected_script_tags.insert(script.tags.clone());
                affected_script_count += 1;
            }
        }

        let mut affected_template_tags = HashSet::new();
        let mut affected_template_count = 0;

        for template in templates {
            if template.tags.split('/').any(|seg| seg == old_segment) {
                affected_template_tags.insert(template.tags.clone());
                affected_template_count += 1;
            }
        }

        RenameImpact {
            script_tags: affected_script_tags,
            script_count: affected_script_count,
            template_tags: affected_template_tags,
            template_count: affected_template_count,
        }
    }

    /// 计算删除影响范围（用于预览）
    pub fn calculate_delete_impact(
        &self,
        parent_tag: &str,
        scripts: &[Arc<Script>],
        templates: &[Arc<Template>],
    ) -> DeleteImpact {
        let affected_script_tags: Vec<String> = scripts
            .iter()
            .filter(|s| Self::is_tag_descendant(&s.tags, parent_tag))
            .map(|s| s.tags.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let script_count = scripts
            .iter()
            .filter(|s| Self::is_tag_descendant(&s.tags, parent_tag))
            .count();

        let affected_template_tags: Vec<String> = templates
            .iter()
            .filter(|t| Self::is_tag_descendant(&t.tags, parent_tag))
            .map(|t| t.tags.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let template_count = templates
            .iter()
            .filter(|t| Self::is_tag_descendant(&t.tags, parent_tag))
            .count();

        DeleteImpact {
            script_tags: affected_script_tags,
            script_count,
            template_tags: affected_template_tags,
            template_count,
        }
    }

    /// 合并标签
    pub fn merge_tags(
        &mut self,
        source_tag: &str,
        target_tag: &str,
        scripts: &mut [Script],
        templates: &mut [Template],
    ) -> bool {
        if source_tag == target_tag || !self.all_tags.contains_key(source_tag) {
            return false;
        }

        // 合并脚本标签
        for script in scripts.iter_mut() {
            if script.tags == source_tag {
                script.tags = target_tag.to_string();
            }
        }

        // 合并模板标签
        for template in templates.iter_mut() {
            if template.tags == source_tag {
                template.tags = target_tag.to_string();
            }
        }

        // 重建标签树
        self.build_tag_tree(scripts, templates);

        true
    }

    /// 获取标签建议
    pub fn get_tag_suggestions(&self, partial_tag: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 前缀匹配
        for tag in self.all_tags.keys() {
            if tag.starts_with(partial_tag) {
                suggestions.push(tag.clone());
            }
        }

        // 排序并限制数量
        suggestions.sort_by(|a, b| {
            // 先按长度排序，再按使用次数排序
            let a_len = a.len();
            let b_len = b.len();
            if a_len != b_len {
                a_len.cmp(&b_len)
            } else {
                let a_count = self.all_tags.get(a).unwrap_or(&0);
                let b_count = self.all_tags.get(b).unwrap_or(&0);
                b_count.cmp(a_count)
            }
        });

        // 最多返回10个建议
        suggestions.truncate(10);

        suggestions
    }
}

/// 标签验证函数
pub fn validate_tag(tag: &str) -> bool {
    if tag.is_empty() {
        return true;
    }

    // 检查标签格式：只允许字母、数字、下划线、连字符和斜杠
    !tag.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '/') &&
    // 不允许连续斜杠
    !tag.contains("//") &&
    // 不允许以斜杠开头或结尾
    !tag.starts_with('/') && !tag.ends_with('/')
}

/// 清理标签格式
pub fn sanitize_tag(tag: &str) -> String {
    let mut result = String::new();
    let mut last_char_was_slash = false;

    for c in tag.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            result.push(c);
            last_char_was_slash = false;
        } else if c == '/' && !last_char_was_slash {
            result.push(c);
            last_char_was_slash = true;
        }
    }

    // 移除首尾斜杠
    result.trim_matches('/').to_string()
}

/// 重命名影响结构
#[derive(Debug, Clone)]
pub struct RenameImpact {
    pub script_tags: HashSet<String>,
    pub script_count: usize,
    pub template_tags: HashSet<String>,
    pub template_count: usize,
}

/// 删除影响结构
#[derive(Debug, Clone)]
pub struct DeleteImpact {
    pub script_tags: Vec<String>,
    pub script_count: usize,
    pub template_tags: Vec<String>,
    pub template_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_script(id: &str, tags: &str) -> Script {
        Script {
            id: id.to_string(),
            name: format!("Script {id}"),
            tags: tags.to_string(),
            content: String::new(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
            use_count: 0,
            last_used: Utc::now(),
            is_favorite: false,
        }
    }

    fn make_template(id: &str, tags: &str) -> Template {
        Template {
            id: id.to_string(),
            name: format!("Template {id}"),
            tags: tags.to_string(),
            script_ids: vec![],
            variable_values: HashMap::new(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
            use_count: 0,
            last_used: Utc::now(),
            is_favorite: false,
            category: String::new(),
        }
    }

    #[test]
    fn rename_tag_segment_updates_matching_path_segments_only() {
        let mut manager = TagManager::new();
        let scripts_seed = vec![
            make_script("1", "coding/python/debug"),
            make_script("2", "tools/pycharm"),
        ];
        let templates_seed = vec![make_template("t1", "workflow/python/review")];
        manager.build_tag_tree(&scripts_seed, &templates_seed);

        let mut scripts: Vec<Arc<Script>> = scripts_seed.into_iter().map(Arc::new).collect();
        let mut templates: Vec<Arc<Template>> = templates_seed.into_iter().map(Arc::new).collect();

        let changed = manager.rename_tag_segment("python", "Python", &mut scripts, &mut templates);

        assert!(changed);
        assert_eq!(scripts[0].tags, "coding/Python/debug");
        assert_eq!(scripts[1].tags, "tools/pycharm");
        assert_eq!(templates[0].tags, "workflow/Python/review");
    }

    #[test]
    fn calculate_rename_impact_counts_unique_tags_and_total_matches() {
        let manager = TagManager::new();
        let scripts = vec![
            Arc::new(make_script("1", "coding/python")),
            Arc::new(make_script("2", "coding/python/debug")),
            Arc::new(make_script("3", "tools/rust")),
        ];
        let templates = vec![
            Arc::new(make_template("t1", "workflow/python/review")),
            Arc::new(make_template("t2", "workflow/rust/review")),
        ];

        let impact = manager.calculate_rename_impact("python", &scripts, &templates);

        assert_eq!(impact.script_count, 2);
        assert_eq!(impact.template_count, 1);
        assert!(impact.script_tags.contains("coding/python"));
        assert!(impact.script_tags.contains("coding/python/debug"));
        assert!(impact.template_tags.contains("workflow/python/review"));
    }

    #[test]
    fn cascade_delete_tag_removes_descendants_and_rebuilds_tag_tree() {
        let mut manager = TagManager::new();
        let scripts_seed = vec![
            make_script("1", "coding/python"),
            make_script("2", "coding/python/debug"),
            make_script("3", "coding/rust"),
        ];
        let templates_seed = vec![
            make_template("t1", "workflow/python/review"),
            make_template("t2", "workflow/rust/review"),
        ];
        manager.build_tag_tree(&scripts_seed, &templates_seed);

        let mut scripts: Vec<Arc<Script>> = scripts_seed.into_iter().map(Arc::new).collect();
        let mut templates: Vec<Arc<Template>> = templates_seed.into_iter().map(Arc::new).collect();

        let changed = manager.cascade_delete_tag("coding/python", &mut scripts, &mut templates);

        assert!(changed);
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].tags, "coding/rust");
        assert_eq!(templates.len(), 2);
        let all_tags = manager.get_all_tags();
        assert!(all_tags
            .iter()
            .all(|(tag, _)| !tag.starts_with("coding/python")));
    }

    #[test]
    fn calculate_delete_impact_counts_descendants() {
        let manager = TagManager::new();
        let scripts = vec![
            Arc::new(make_script("1", "coding/python")),
            Arc::new(make_script("2", "coding/python/debug")),
            Arc::new(make_script("3", "coding/rust")),
        ];
        let templates = vec![
            Arc::new(make_template("t1", "coding/python/review")),
            Arc::new(make_template("t2", "workflow/rust/review")),
        ];

        let impact = manager.calculate_delete_impact("coding/python", &scripts, &templates);

        assert_eq!(impact.script_count, 2);
        assert_eq!(impact.template_count, 1);
        assert!(impact.script_tags.iter().any(|tag| tag == "coding/python"));
        assert!(impact
            .script_tags
            .iter()
            .any(|tag| tag == "coding/python/debug"));
        assert!(impact
            .template_tags
            .iter()
            .any(|tag| tag == "coding/python/review"));
    }
}
