use regex::Regex;

use crate::config::PrivacyConfig;

pub fn should_include_app(app: &str, config: &PrivacyConfig) -> bool {
    // If allowlist is set, only include listed apps
    if !config.allowlist_apps.is_empty() {
        return config.allowlist_apps.iter().any(|a| a.eq_ignore_ascii_case(app));
    }
    // If blocklist is set, exclude listed apps
    if config.blocklist_apps.iter().any(|a| a.eq_ignore_ascii_case(app)) {
        return false;
    }
    true
}

pub fn redact_text(text: &str, config: &PrivacyConfig) -> String {
    let mut result = text.to_string();

    // Built-in email pattern
    let email_re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    result = email_re.replace_all(&result, "[REDACTED_EMAIL]").to_string();

    // Built-in URL token pattern (query params with token/key/secret)
    let token_re = Regex::new(r"(?i)(token|key|secret|password|auth)=[^\s&]+").unwrap();
    result = token_re.replace_all(&result, "$1=[REDACTED]").to_string();

    // User-configured patterns
    for pattern in &config.redaction_patterns {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocklist_filtering() {
        let config = PrivacyConfig {
            allowlist_apps: vec![],
            blocklist_apps: vec!["Signal".to_string(), "1Password".to_string()],
            redaction_patterns: vec![],
        };

        assert!(should_include_app("Chrome", &config));
        assert!(should_include_app("VS Code", &config));
        assert!(!should_include_app("Signal", &config));
        assert!(!should_include_app("1Password", &config));
    }

    #[test]
    fn test_allowlist_filtering() {
        let config = PrivacyConfig {
            allowlist_apps: vec!["Chrome".to_string(), "VS Code".to_string()],
            blocklist_apps: vec![],
            redaction_patterns: vec![],
        };

        assert!(should_include_app("Chrome", &config));
        assert!(should_include_app("VS Code", &config));
        assert!(!should_include_app("Signal", &config));
    }

    #[test]
    fn test_email_redaction() {
        let config = PrivacyConfig::default();
        let result = redact_text("User opened user@example.com in mail", &config);
        assert_eq!(result, "User opened [REDACTED_EMAIL] in mail");
    }

    #[test]
    fn test_token_redaction() {
        let config = PrivacyConfig::default();
        let result = redact_text("https://example.com?token=abc123&foo=bar", &config);
        assert!(result.contains("token=[REDACTED]"));
        assert!(result.contains("foo=bar"));
    }

    #[test]
    fn test_custom_pattern_redaction() {
        let config = PrivacyConfig {
            allowlist_apps: vec![],
            blocklist_apps: vec![],
            redaction_patterns: vec![r"\d{3}-\d{4}-\d{4}".to_string()],
        };
        let result = redact_text("Call me at 090-1234-5678", &config);
        assert_eq!(result, "Call me at [REDACTED]");
    }
}
