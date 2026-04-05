// Library entry point for Tauri
pub mod data;
pub mod logic;
pub mod utils;

// Re-export commonly used types
pub use data::{Script, Template, Storage};
pub use logic::{
    history::HistoryManager,
    tags::TagManager,
    variable::{parse_variables, parse_variables_with_defaults, replace_variables, Variable},
    search::{fuzzy_search_scripts, fuzzy_search_templates},
};
pub use utils::{
    clipboard::ClipboardManager,
    validator::Validator,
};
