use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::memory::Database;

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
}

pub struct CommandContext {
    pub app_handle: tauri::AppHandle,
    pub db: Arc<Mutex<Database>>,
    pub plugin_list: Vec<PluginInfo>,
}

#[derive(Debug)]
pub struct CommandResult {
    pub response: String,
}

#[async_trait]
pub trait CommandPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    async fn execute(&self, args: &str, ctx: &CommandContext) -> Result<CommandResult, String>;
}
