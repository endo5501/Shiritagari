use async_trait::async_trait;

use super::types::{CommandContext, CommandPlugin, CommandResult};

pub struct HelpPlugin;

#[async_trait]
impl CommandPlugin for HelpPlugin {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "���用可能なコマンド一覧を表示"
    }

    fn usage(&self) -> &str {
        "/help"
    }

    async fn execute(&self, _args: &str, ctx: &CommandContext) -> Result<CommandResult, String> {
        let mut lines = vec!["利用可能なコマンド:".to_string()];
        for info in &ctx.plugin_list {
            lines.push(format!("  /{} - {}", info.name, info.description));
            lines.push(format!("    使い方: {}", info.usage));
        }
        Ok(CommandResult {
            response: lines.join("\n"),
        })
    }
}
