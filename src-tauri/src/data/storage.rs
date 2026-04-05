// JSON文件读写、导入导出
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashSet;
use uuid::Uuid;
use serde_json::Value;
use crate::data::{Script, Template};
use crate::logic::{history::HistoryManager, tags::TagManager};

// 自定义错误类型，用于处理不同类型的错误
#[derive(Debug)]
struct ImportError(String);

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ImportError {}

// 移除冲突的 From 实现，直接使用 into()
// impl From<ImportError> for Box<dyn Error> {
//     fn from(error: ImportError) -> Self {
//         Box::new(error)
//     }
// }

#[derive(Debug)]
pub struct Storage {
    scripts_file: PathBuf,
    templates_file: PathBuf,
    history_file: PathBuf,
    tags_file: PathBuf,
}


impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        // 确保目录存在
        if let Err(e) = fs::create_dir_all(&data_dir) {
            eprintln!("警告: 无法创建数据目录: {}", e);
        }
        
        let scripts_file = data_dir.join("scripts.json");
        let templates_file = data_dir.join("templates.json");
        let history_file = data_dir.join("history.json");
        let tags_file = data_dir.join("tags.json");
        
        // 初始化默认数据文件
        if !scripts_file.exists() {
            if let Err(e) = Self::create_default_scripts_file(&scripts_file) {
                eprintln!("警告: 无法创建默认脚本文件: {}", e);
            }
        }
        
        if !templates_file.exists() {
            if let Err(e) = Self::create_default_templates_file(&templates_file) {
                eprintln!("警告: 无法创建默认模板文件: {}", e);
            }
        }
        
        if !history_file.exists() {
            if let Err(e) = Self::create_default_history_file(&history_file) {
                eprintln!("警告: 无法创建默认历史文件: {}", e);
            }
        }
        
        if !tags_file.exists() {
            if let Err(e) = Self::create_default_tags_file(&tags_file) {
                eprintln!("警告: 无法创建默认标签文件: {}", e);
            }
        }
        
        Storage {
            scripts_file,
            templates_file,
            history_file,
            tags_file,
        }
    }
    
    // 兼容旧版本的new方法
    pub fn new_with_dev_mode(is_dev_mode: bool) -> Result<Self, Box<dyn Error>> {
        let data_path = if is_dev_mode {
            // 开发模式使用当前目录
            PathBuf::from("./dev_data")
        } else {
            // 生产模式使用用户目录
            let home_dir = dirs::home_dir().ok_or_else(|| ImportError("无法获取用户目录".to_string()))?;
            home_dir.join(".scripted-prompt")
        };
        
        Ok(Self::new(data_path))
    }
    
    fn create_default_scripts_file(path: &PathBuf) -> Result<(), Box<dyn Error>> {
        // 创建多个示例脚本
        let now = chrono::Utc::now();

        let scripts = vec![
            Script {
                id: "example_001".to_string(),
                name: "Code Review Prompt".to_string(),
                tags: "development/review".to_string(),
                content: "Please review the following {{language:Python}} code from {{project_name}}:\n\n{{code}}\n\nFocus on: {{focus_areas:performance, security, readability}}".to_string(),
                created_at: "2025-10-24T00:00:00Z".to_string(),
                updated_at: "2025-10-24T00:00:00Z".to_string(),
                use_count: 15,
                last_used: now,
                is_favorite: true,
            },
            Script {
                id: "example_002".to_string(),
                name: "Bug Fix Analysis".to_string(),
                tags: "development/debugging".to_string(),
                content: "I have a bug in my {{language:JavaScript}} code:\n\nBug Description: {{bug_description}}\n\nRelevant Code:\n{{code}}\n\nExpected Behavior: {{expected_behavior}}\nActual Behavior: {{actual_behavior}}\n\nPlease help me identify and fix the issue.".to_string(),
                created_at: "2025-10-24T00:00:00Z".to_string(),
                updated_at: "2025-10-24T00:00:00Z".to_string(),
                use_count: 8,
                last_used: now,
                is_favorite: false,
            },
            Script {
                id: "example_003".to_string(),
                name: "API Documentation Request".to_string(),
                tags: "development/documentation".to_string(),
                content: "Generate comprehensive API documentation for the following {{language:TypeScript}} function:\n\n{{code}}\n\nInclude:\n- Function description\n- Parameters with types and descriptions\n- Return value\n- Usage examples\n- Potential errors".to_string(),
                created_at: "2025-10-24T00:00:00Z".to_string(),
                updated_at: "2025-10-24T00:00:00Z".to_string(),
                use_count: 12,
                last_used: now,
                is_favorite: false,
            },
            Script {
                id: "example_004".to_string(),
                name: "Test Case Generator".to_string(),
                tags: "development/testing".to_string(),
                content: "Create comprehensive test cases for:\n\nFunction/Module: {{module_name}}\nLanguage: {{language:Python}}\nFramework: {{test_framework:pytest}}\n\nCode:\n{{code}}\n\nPlease include:\n- Unit tests for normal cases\n- Edge case tests\n- Error handling tests\n- Mock dependencies as needed".to_string(),
                created_at: "2025-10-24T00:00:00Z".to_string(),
                updated_at: "2025-10-24T00:00:00Z".to_string(),
                use_count: 5,
                last_used: now,
                is_favorite: false,
            },
            Script {
                id: "example_005".to_string(),
                name: "Learning Resource Generator".to_string(),
                tags: "learning/tutorials".to_string(),
                content: "Create a comprehensive learning guide for {{topic}}.\n\nTarget Audience: {{audience:Beginner}}\nFormat: {{format:Step-by-step tutorial}}\n\nInclude:\n1. Introduction and prerequisites\n2. Core concepts explanation\n3. Practical examples\n4. Common pitfalls and best practices\n5. Additional resources".to_string(),
                created_at: "2025-10-24T00:00:00Z".to_string(),
                updated_at: "2025-10-24T00:00:00Z".to_string(),
                use_count: 10,
                last_used: now,
                is_favorite: false,
            },
        ];

        let json_content = serde_json::to_string_pretty(&scripts)?;
        fs::write(path, json_content)?;
        Ok(())
    }
    
    fn create_default_templates_file(path: &PathBuf) -> Result<(), Box<dyn Error>> {
        // 创建多个示例模板
        let now = chrono::Utc::now();

        let templates = vec![
            Template {
                id: "template_example_001".to_string(),
                name: "Complete Code Review Workflow".to_string(),
                tags: "workflow/review".to_string(),
                script_ids: vec!["example_001".to_string(), "example_003".to_string(), "example_004".to_string()],
                variable_values: std::collections::HashMap::new(),
                created_at: "2025-10-24T00:00:00Z".to_string(),
                updated_at: "2025-10-24T00:00:00Z".to_string(),
                use_count: 8,
                last_used: now,
                is_favorite: true,
                category: String::new(),
            },
            Template {
                id: "template_example_002".to_string(),
                name: "Bug Investigation Process".to_string(),
                tags: "workflow/debugging".to_string(),
                script_ids: vec!["example_002".to_string(), "example_004".to_string()],
                variable_values: std::collections::HashMap::new(),
                created_at: "2025-10-24T00:00:00Z".to_string(),
                updated_at: "2025-10-24T00:00:00Z".to_string(),
                use_count: 5,
                last_used: now,
                is_favorite: false,
                category: String::new(),
            },
        ];

        let json_content = serde_json::to_string_pretty(&templates)?;
        fs::write(path, json_content)?;
        Ok(())
    }
    
    fn create_default_history_file(path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let history = HistoryManager::new();
        let json_content = serde_json::to_string_pretty(&history)?;
        fs::write(path, json_content)?;
        Ok(())
    }

    fn create_default_tags_file(path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let tags = TagManager::new();
        let json_content = serde_json::to_string_pretty(&tags)?;
        fs::write(path, json_content)?;
        Ok(())
    }
    
    pub fn load_scripts(&self) -> Result<Vec<Script>, Box<dyn Error>> {
        let content = fs::read_to_string(&self.scripts_file)
            .map_err(|e| ImportError(format!("Failed to read scripts file: {}", e)))?;
        let scripts: Vec<Script> = serde_json::from_str(&content)?;
        Ok(scripts)
    }
    
    pub fn save_scripts(&self, scripts: &[Script]) -> Result<(), Box<dyn Error>> {
        let json_content = serde_json::to_string_pretty(scripts)?;
        fs::write(&self.scripts_file, json_content)
            .map_err(|e| ImportError(format!("Failed to write scripts file: {}", e)))?;
        Ok(())
    }
    
    pub fn load_templates(&self) -> Result<Vec<Template>, Box<dyn Error>> {
        let content = fs::read_to_string(&self.templates_file)
            .map_err(|e| ImportError(format!("Failed to read templates file: {}", e)))?;
        let templates: Vec<Template> = serde_json::from_str(&content)?;
        Ok(templates)
    }
    
    pub fn save_templates(&self, templates: &[Template]) -> Result<(), Box<dyn Error>> {
        let json_content = serde_json::to_string_pretty(templates)?;
        fs::write(&self.templates_file, json_content)
            .map_err(|e| ImportError(format!("Failed to write templates file: {}", e)))?;
        Ok(())
    }
    
    pub fn load_history(&self) -> Result<HistoryManager, Box<dyn Error>> {
        if !self.history_file.exists() {
            return Ok(HistoryManager::new());
        }
        
        let content = fs::read_to_string(&self.history_file)
            .map_err(|e| ImportError(format!("Failed to read history file: {}", e)))?;
        let history: HistoryManager = serde_json::from_str(&content)?;
        Ok(history)
    }
    
    pub fn save_history(&self, history: &HistoryManager) -> Result<(), Box<dyn Error>> {
        let json_content = serde_json::to_string_pretty(history)?;
        fs::write(&self.history_file, json_content)
            .map_err(|e| ImportError(format!("Failed to write history file: {}", e)))?;
        Ok(())
    }
    
    pub fn load_tags(&self) -> Result<TagManager, Box<dyn Error>> {
        if !self.tags_file.exists() {
            return Ok(TagManager::new());
        }
        
        let content = fs::read_to_string(&self.tags_file)
            .map_err(|e| ImportError(format!("Failed to read tags file: {}", e)))?;
        let tags: TagManager = serde_json::from_str(&content)?;
        Ok(tags)
    }
    
    pub fn save_tags(&self, tags: &TagManager) -> Result<(), Box<dyn Error>> {
        let json_content = serde_json::to_string_pretty(tags)?;
        fs::write(&self.tags_file, json_content)
            .map_err(|e| ImportError(format!("Failed to write tags file: {}", e)))?;
        Ok(())
    }
    
    pub fn export_all(&self, export_path: &str) -> Result<(), Box<dyn Error>> {
        // 导出所有数据到一个文件
        let scripts = self.load_scripts()?;
        let templates = self.load_templates()?;

        let mut export_data = std::collections::HashMap::new();
        export_data.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));
        export_data.insert("scripts".to_string(), serde_json::to_value(scripts)?);
        export_data.insert("templates".to_string(), serde_json::to_value(templates)?);

        let json_content = serde_json::to_string_pretty(&export_data)?;
        fs::write(export_path, json_content)
            .map_err(|e| ImportError(format!("Failed to write export file: {}", e)))?;
        Ok(())
    }

    pub fn import_all(&self, import_path: &str) -> Result<(), Box<dyn Error>> {
        let content = fs::read_to_string(import_path)
            .map_err(|e| ImportError(format!("Failed to read import file: {}", e)))?;
        let import_data: Value = serde_json::from_str(&content)
            .map_err(|e| ImportError(format!("Failed to parse JSON: {}", e)))?;

        let mut existing_scripts = self.load_scripts()?;
        let mut existing_templates = self.load_templates()?;
        let existing_history = self.load_history()?;

        let imported_scripts: Vec<Script> = match import_data.get("scripts").and_then(|v| v.as_array()) {
            Some(scripts_value) => serde_json::from_value(Value::Array(scripts_value.clone()))
                .map_err(|e| ImportError(format!("Failed to deserialize scripts: {}", e)))?,
            None => Vec::new(),
        };

        let imported_templates: Vec<Template> = match import_data.get("templates").and_then(|v| v.as_array()) {
            Some(templates_value) => serde_json::from_value(Value::Array(templates_value.clone()))
                .map_err(|e| ImportError(format!("Failed to deserialize templates: {}", e)))?,
            None => Vec::new(),
        };

        let mut script_id_map = std::collections::HashMap::new();
        let existing_script_ids: HashSet<String> = existing_scripts.iter().map(|script| script.id.clone()).collect();
        let mut used_script_ids = existing_script_ids.clone();

        for mut script in imported_scripts {
            let original_id = script.id.clone();
            if used_script_ids.contains(&script.id) {
                script.id = Uuid::new_v4().to_string();
            }
            used_script_ids.insert(script.id.clone());
            script_id_map.insert(original_id, script.id.clone());
            existing_scripts.push(script);
        }

        let existing_template_ids: std::collections::HashSet<String> = existing_templates.iter().map(|template| template.id.clone()).collect();
        let mut used_template_ids = existing_template_ids.clone();

        for mut template in imported_templates {
            let original_id = template.id.clone();
            if used_template_ids.contains(&template.id) {
                template.id = Uuid::new_v4().to_string();
            }
            used_template_ids.insert(template.id.clone());

            template.script_ids = template
                .script_ids
                .into_iter()
                .map(|id| script_id_map.get(&id).cloned().unwrap_or(id))
                .collect();

            let _ = original_id;
            existing_templates.push(template);
        }

        let mut tags = TagManager::new();
        tags.update_from_scripts(&existing_scripts);
        tags.update_from_templates(&existing_templates);

        self.save_scripts(&existing_scripts)
            .map_err(|e| ImportError(format!("Failed to save merged scripts: {}", e)))?;
        self.save_templates(&existing_templates)
            .map_err(|e| ImportError(format!("Failed to save merged templates: {}", e)))?;
        self.save_history(&existing_history)
            .map_err(|e| ImportError(format!("Failed to preserve history: {}", e)))?;
        self.save_tags(&tags)
            .map_err(|e| ImportError(format!("Failed to save rebuilt tags: {}", e)))?;

        Ok(())
    }
    
    /// 加载所有数据
    pub fn load_all(&self) -> Result<(Vec<Script>, Vec<Template>, HistoryManager, TagManager), Box<dyn Error>> {
        let scripts = self.load_scripts()?;
        let templates = self.load_templates()?;
        let history = self.load_history()?;
        let tags = self.load_tags()?;
        
        Ok((scripts, templates, history, tags))
    }
    
    /// 保存所有数据
    pub fn save_all(&self) -> Result<(), Box<dyn Error>> {
        let (scripts, templates, history, tags) = self.load_all()?;
        
        self.save_scripts(&scripts)?;
        self.save_templates(&templates)?;
        self.save_history(&history)?;
        self.save_tags(&tags)?;
        
        Ok(())
    }
    
    /// 使用提供的数据保存所有内容
    pub fn save_all_with_data(
        &self,
        scripts: Vec<Arc<Script>>,
        templates: Vec<Arc<Template>>,
        history: HistoryManager,
        tags: TagManager
    ) -> Result<(), Box<dyn Error>> {
        // 转换Arc<Script>到Script
        let script_vec: Vec<Script> = scripts.iter().map(|s| (**s).clone()).collect();
        let template_vec: Vec<Template> = templates.iter().map(|t| (**t).clone()).collect();

        self.save_scripts(&script_vec)?;
        self.save_templates(&template_vec)?;
        self.save_history(&history)?;
        self.save_tags(&tags)?;

        Ok(())
    }
}