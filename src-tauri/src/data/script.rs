// Script数据结构与方法
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub tags: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub use_count: u32,
    #[serde(default)]
    pub last_used: DateTime<Utc>,
    #[serde(default)]
    pub is_favorite: bool,
}

impl Script {
    pub fn new(name: String, tags: String, content: String) -> Self {
        let now = Utc::now();

        Script {
            id: Uuid::new_v4().to_string(),
            name,
            tags,
            content,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            use_count: 0,
            last_used: now,
            is_favorite: false,
        }
    }
    
    pub fn update(&mut self, name: Option<String>, tags: Option<String>, content: Option<String>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(tags) = tags {
            self.tags = tags;
        }
        if let Some(content) = content {
            self.content = content;
        }
        self.updated_at = Utc::now().to_rfc3339();
    }
    
    pub fn increment_use_count(&mut self) {
        self.use_count += 1;
        self.last_used = Utc::now();
    }
    
    pub fn get_tags_hierarchy(&self) -> Vec<String> {
        self.tags.split('/').filter(|s| !s.is_empty()).map(String::from).collect()
    }
    
    pub fn matches_tags(&self, tag_filter: &str) -> bool {
        if tag_filter.is_empty() {
            return true;
        }
        
        // 精确匹配完整标签路径
        if self.tags == tag_filter {
            return true;
        }
        
        // 匹配标签前缀
        if self.tags.starts_with(&format!("{}/", tag_filter)) {
            return true;
        }
        
        false
    }
    
    pub fn contains_text(&self, search_text: &str) -> bool {
        let search_lower = search_text.to_lowercase();
        self.name.to_lowercase().contains(&search_lower) ||
        self.content.to_lowercase().contains(&search_lower) ||
        self.tags.to_lowercase().contains(&search_lower)
    }
}