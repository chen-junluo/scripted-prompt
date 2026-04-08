use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub ai: AiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            base_url: String::new(),
            api_key: String::new(),
            model: String::new(),
            temperature: Some(0.2),
            max_output_tokens: Some(1200),
        }
    }
}

impl AiSettings {
    pub fn is_configured(&self) -> bool {
        !self.base_url.trim().is_empty()
            && !self.api_key.trim().is_empty()
            && !self.model.trim().is_empty()
    }
}

fn default_provider() -> String {
    "openai-compatible".to_string()
}
