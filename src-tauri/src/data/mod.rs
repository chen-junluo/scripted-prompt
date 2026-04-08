// data模块声明
mod script;
mod settings;
mod storage;
mod template;

pub use script::Script;
pub use settings::{AiSettings, Settings};
pub use storage::Storage;
pub use template::Template;
