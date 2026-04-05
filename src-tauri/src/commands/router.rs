use log::warn;
use std::collections::HashMap;

use super::types::{CommandContext, CommandPlugin, CommandResult, PluginInfo};

pub struct CommandRouter {
    plugins: HashMap<String, Box<dyn CommandPlugin>>,
}

impl CommandRouter {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn CommandPlugin>) {
        let name = plugin.name().to_string();
        if self.plugins.contains_key(&name) {
            warn!("Command '{}' is already registered, overwriting", name);
        }
        self.plugins.insert(name, plugin);
    }

    pub fn plugin_list(&self) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .map(|p| PluginInfo {
                name: p.name().to_string(),
                description: p.description().to_string(),
                usage: p.usage().to_string(),
            })
            .collect()
    }

    /// Parse a slash command input into (command_name, args).
    /// Returns None if the input doesn't start with '/'.
    pub fn parse(input: &str) -> Option<(&str, &str)> {
        let input = input.trim();
        if !input.starts_with('/') {
            return None;
        }
        let without_slash = &input[1..];
        if without_slash.is_empty() {
            return Some(("", ""));
        }
        match without_slash.split_once(char::is_whitespace) {
            Some((cmd, args)) => Some((cmd, args.trim())),
            None => Some((without_slash, "")),
        }
    }

    pub async fn dispatch(
        &self,
        input: &str,
        ctx: &CommandContext,
    ) -> Result<CommandResult, String> {
        let (cmd_name, args) = Self::parse(input)
            .ok_or_else(|| "コマンドは / で始めてください".to_string())?;

        if cmd_name.is_empty() {
            return Err(
                "不明なコマンドです。/help で利用可能なコマンドを確認してください".to_string(),
            );
        }

        match self.plugins.get(cmd_name) {
            Some(plugin) => plugin.execute(args, ctx).await,
            None => Err(format!(
                "不明なコマンドです: /{}。/help で利用可能なコマンドを確認してください",
                cmd_name
            )),
        }
    }

    /// Look up a plugin by command name (used for testing without CommandContext).
    pub fn has_command(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct EchoPlugin;

    #[async_trait]
    impl CommandPlugin for EchoPlugin {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "エコーバック"
        }
        fn usage(&self) -> &str {
            "/echo <message>"
        }
        async fn execute(
            &self,
            args: &str,
            _ctx: &CommandContext,
        ) -> Result<CommandResult, String> {
            Ok(CommandResult {
                response: args.to_string(),
            })
        }
    }

    // --- parse tests ---

    #[test]
    fn test_parse_command_with_args() {
        let result = CommandRouter::parse("/timer 3時間");
        assert_eq!(result, Some(("timer", "3時間")));
    }

    #[test]
    fn test_parse_command_without_args() {
        let result = CommandRouter::parse("/help");
        assert_eq!(result, Some(("help", "")));
    }

    #[test]
    fn test_parse_command_with_extra_whitespace() {
        let result = CommandRouter::parse("/timer   3時間  ");
        assert_eq!(result, Some(("timer", "3時間")));
    }

    #[test]
    fn test_parse_slash_only() {
        let result = CommandRouter::parse("/");
        assert_eq!(result, Some(("", "")));
    }

    #[test]
    fn test_parse_non_command() {
        let result = CommandRouter::parse("hello");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_empty_string() {
        let result = CommandRouter::parse("");
        assert_eq!(result, None);
    }

    // --- registration tests ---

    #[test]
    fn test_register_and_has_command() {
        let mut router = CommandRouter::new();
        assert!(!router.has_command("echo"));
        router.register(Box::new(EchoPlugin));
        assert!(router.has_command("echo"));
    }

    #[test]
    fn test_unknown_command_not_found() {
        let router = CommandRouter::new();
        assert!(!router.has_command("unknown"));
    }

    #[test]
    fn test_plugin_list() {
        let mut router = CommandRouter::new();
        router.register(Box::new(EchoPlugin));
        let list = router.plugin_list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "echo");
        assert_eq!(list[0].description, "エコーバック");
        assert_eq!(list[0].usage, "/echo <message>");
    }
}
