use std::sync::LazyLock;
use std::time::Duration;

use async_trait::async_trait;
use regex::Regex;

use super::types::{CommandContext, CommandPlugin, CommandResult};
use crate::events;

static RE_HOURS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)時間").unwrap());
static RE_MINUTES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)分").unwrap());
static RE_SECONDS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)秒").unwrap());

pub fn parse_duration(input: &str) -> Result<Duration, String> {
    let input = input.replace(char::is_whitespace, "");
    if input.is_empty() {
        return Err("時間を指定してください".to_string());
    }

    let cap_hours = RE_HOURS.captures(&input);
    let cap_minutes = RE_MINUTES.captures(&input);
    let cap_seconds = RE_SECONDS.captures(&input);

    let hours: u64 = cap_hours
        .as_ref()
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(0);
    let minutes: u64 = cap_minutes
        .as_ref()
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(0);
    let seconds: u64 = cap_seconds
        .as_ref()
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(0);

    let has_unit = cap_hours.is_some() || cap_minutes.is_some() || cap_seconds.is_some();

    let total_seconds = if has_unit {
        hours * 3600 + minutes * 60 + seconds
    } else {
        let bare: u64 = input
            .parse()
            .map_err(|_| "時間を解析できませんでした".to_string())?;
        bare * 60
    };

    if total_seconds == 0 {
        return Err("0より大きい時間を指定してください".to_string());
    }

    Ok(Duration::from_secs(total_seconds))
}

pub fn format_duration(d: &Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{}時間", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}分", minutes));
    }
    if seconds > 0 {
        parts.push(format!("{}秒", seconds));
    }
    parts.join("")
}

pub struct TimerPlugin;

#[async_trait]
impl CommandPlugin for TimerPlugin {
    fn name(&self) -> &str {
        "timer"
    }

    fn description(&self) -> &str {
        "指定時間後に通知するタイマー"
    }

    fn usage(&self) -> &str {
        "/timer <時間> (例: /timer 30分, /timer 1時間30分, /timer 90秒)"
    }

    async fn execute(&self, args: &str, ctx: &CommandContext) -> Result<CommandResult, String> {
        let args = args.trim();
        if args.is_empty() {
            return Err(format!("使い方: {}", self.usage()));
        }

        let duration = parse_duration(args)
            .map_err(|e| format!("{}\n使い方: {}", e, self.usage()))?;
        let label = format_duration(&duration);
        let app_handle = ctx.app_handle.clone();

        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            events::emit_thought(&app_handle, "⏰ タイマーが完了したよ！", 1.0);
            events::bring_window_to_front(&app_handle);
        });

        Ok(CommandResult {
            response: format!("タイマーを{}に設定したよ！", label),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hours() {
        let d = parse_duration("3時間").unwrap();
        assert_eq!(d, Duration::from_secs(3 * 3600));
    }

    #[test]
    fn test_parse_minutes() {
        let d = parse_duration("30分").unwrap();
        assert_eq!(d, Duration::from_secs(30 * 60));
    }

    #[test]
    fn test_parse_seconds() {
        let d = parse_duration("90秒").unwrap();
        assert_eq!(d, Duration::from_secs(90));
    }

    #[test]
    fn test_parse_composite() {
        let d = parse_duration("1時間30分").unwrap();
        assert_eq!(d, Duration::from_secs(3600 + 30 * 60));
    }

    #[test]
    fn test_parse_full_composite() {
        let d = parse_duration("1時間30分15秒").unwrap();
        assert_eq!(d, Duration::from_secs(3600 + 30 * 60 + 15));
    }

    #[test]
    fn test_parse_bare_number() {
        let d = parse_duration("30").unwrap();
        assert_eq!(d, Duration::from_secs(30 * 60));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let d = parse_duration("1時間 30分").unwrap();
        assert_eq!(d, Duration::from_secs(3600 + 30 * 60));
    }

    #[test]
    fn test_parse_zero_error() {
        let result = parse_duration("0分");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("0より大きい"));
    }

    #[test]
    fn test_parse_empty_error() {
        let result = parse_duration("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_non_numeric_error() {
        let result = parse_duration("abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("解析できませんでした"));
    }

    #[test]
    fn test_format_duration_hours_minutes() {
        let d = Duration::from_secs(3600 + 30 * 60);
        assert_eq!(format_duration(&d), "1時間30分");
    }

    #[test]
    fn test_format_duration_minutes_only() {
        let d = Duration::from_secs(30 * 60);
        assert_eq!(format_duration(&d), "30分");
    }

    #[test]
    fn test_format_duration_seconds_only() {
        let d = Duration::from_secs(45);
        assert_eq!(format_duration(&d), "45秒");
    }
}
