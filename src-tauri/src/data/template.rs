// Template数据结构与方法
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub tags: String,
    pub script_ids: Vec<String>,
    pub variable_values: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
    pub use_count: u32,
    #[serde(default)]
    pub last_used: DateTime<Utc>,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub category: String,
}

impl Template {
    pub fn new(
        name: String,
        tags: String,
        script_ids: Vec<String>,
        variable_values: HashMap<String, String>,
    ) -> Self {
        let now = Utc::now();

        Template {
            id: Uuid::new_v4().to_string(),
            name,
            tags,
            script_ids,
            variable_values,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            use_count: 0,
            last_used: now,
            is_favorite: false,
            category: String::new(),
        }
    }

    pub fn update(
        &mut self,
        name: Option<String>,
        tags: Option<String>,
        script_ids: Option<Vec<String>>,
        variable_values: Option<HashMap<String, String>>,
    ) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(tags) = tags {
            self.tags = tags;
        }
        if let Some(script_ids) = script_ids {
            self.script_ids = script_ids;
        }
        if let Some(variable_values) = variable_values {
            self.variable_values = variable_values;
        }
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn increment_use_count(&mut self) {
        self.use_count += 1;
        self.last_used = Utc::now();
    }

    pub fn add_script_id(&mut self, script_id: String) {
        if !self.script_ids.contains(&script_id) {
            self.script_ids.push(script_id);
            self.updated_at = Utc::now().to_rfc3339();
        }
    }

    pub fn remove_script_id(&mut self, script_id: &str) {
        self.script_ids.retain(|id| id != script_id);
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn set_variable_value(&mut self, name: String, value: String) {
        self.variable_values.insert(name, value);
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn get_variable_value(&self, name: &str) -> Option<&String> {
        self.variable_values.get(name)
    }

    pub fn get_tags_hierarchy(&self) -> Vec<String> {
        self.tags
            .split('/')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
    }

    pub fn contains_text(&self, search_text: &str) -> bool {
        let search_lower = search_text.to_lowercase();
        self.name.to_lowercase().contains(&search_lower) ||
        self.tags.to_lowercase().contains(&search_lower) ||
        // 搜索变量名
        self.variable_values.keys().any(|key| {
            key.to_lowercase().contains(&search_lower)
        }) ||
        // 搜索变量值
        self.variable_values.values().any(|value| {
            value.to_lowercase().contains(&search_lower)
        })
    }
}
