// Library entry point for Tauri
pub mod data;
pub mod logic;
pub mod utils;

// Re-export commonly used types
pub use data::{AiSettings, Script, Settings, Storage, Template};
pub use logic::{
    ai,
    history::HistoryManager,
    search::{fuzzy_search_scripts, fuzzy_search_templates},
    tags::TagManager,
    variable::{parse_variables, parse_variables_with_defaults, replace_variables, Variable},
};
pub use utils::{clipboard::ClipboardManager, validator::Validator};
