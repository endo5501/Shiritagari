use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub polling: PollingConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub privacy: PrivacyConfig,
    #[serde(default)]
    pub confidence: ConfidenceConfig,
    #[serde(default)]
    pub mascot: MascotConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PollingConfig {
    #[serde(default = "default_polling_interval")]
    pub interval_minutes: u64,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval_minutes: default_polling_interval(),
        }
    }
}

fn default_polling_interval() -> u64 {
    10
}

#[derive(Debug, Deserialize, Clone)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub inference_provider: Option<String>,
    #[serde(default)]
    pub inference_model: Option<String>,
    #[serde(default)]
    pub inference_api_key_env: Option<String>,
    #[serde(default)]
    pub chat_provider: Option<String>,
    #[serde(default)]
    pub chat_model: Option<String>,
    #[serde(default)]
    pub chat_api_key_env: Option<String>,
    #[serde(default)]
    pub ollama_base_url: Option<String>,
    #[serde(default)]
    pub openai_base_url: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: None,
            api_key_env: None,
            inference_provider: None,
            inference_model: None,
            inference_api_key_env: None,
            chat_provider: None,
            chat_model: None,
            chat_api_key_env: None,
            ollama_base_url: None,
            openai_base_url: None,
        }
    }
}

fn default_provider() -> String {
    "ollama".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct PrivacyConfig {
    #[serde(default)]
    pub allowlist_apps: Vec<String>,
    #[serde(default)]
    pub blocklist_apps: Vec<String>,
    #[serde(default)]
    pub redaction_patterns: Vec<String>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            allowlist_apps: Vec::new(),
            blocklist_apps: Vec::new(),
            redaction_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MascotConfig {
    #[serde(default)]
    pub character_image: Option<String>,
}

impl Default for MascotConfig {
    fn default() -> Self {
        Self {
            character_image: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfidenceConfig {
    #[serde(default = "default_decay_rate")]
    pub decay_rate: f64,
    #[serde(default = "default_threshold_silent")]
    pub threshold_silent: f64,
    #[serde(default = "default_threshold_re_ask")]
    pub threshold_re_ask: f64,
    #[serde(default = "default_threshold_soft_delete")]
    pub threshold_soft_delete: f64,
}

impl Default for ConfidenceConfig {
    fn default() -> Self {
        Self {
            decay_rate: default_decay_rate(),
            threshold_silent: default_threshold_silent(),
            threshold_re_ask: default_threshold_re_ask(),
            threshold_soft_delete: default_threshold_soft_delete(),
        }
    }
}

fn default_decay_rate() -> f64 {
    0.99
}
fn default_threshold_silent() -> f64 {
    0.8
}
fn default_threshold_re_ask() -> f64 {
    0.5
}
fn default_threshold_soft_delete() -> f64 {
    0.3
}

impl AppConfig {
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            polling: PollingConfig::default(),
            llm: LlmConfig::default(),
            privacy: PrivacyConfig::default(),
            confidence: ConfidenceConfig::default(),
            mascot: MascotConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.polling.interval_minutes, 10);
        assert_eq!(config.llm.provider, "ollama");
        assert_eq!(config.confidence.decay_rate, 0.99);
        assert_eq!(config.confidence.threshold_silent, 0.8);
        assert_eq!(config.confidence.threshold_re_ask, 0.5);
        assert_eq!(config.confidence.threshold_soft_delete, 0.3);
    }

    #[test]
    fn test_load_config_from_toml() {
        let toml_content = r#"
[polling]
interval_minutes = 30

[llm]
provider = "claude"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[privacy]
blocklist_apps = ["Signal", "1Password"]
redaction_patterns = ["[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"]

[confidence]
decay_rate = 0.98
"#;
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(toml_content.as_bytes()).unwrap();
        let path = tmpfile.path().to_path_buf();

        let config = AppConfig::load(&path).unwrap();
        assert_eq!(config.polling.interval_minutes, 30);
        assert_eq!(config.llm.provider, "claude");
        assert_eq!(config.privacy.blocklist_apps, vec!["Signal", "1Password"]);
        assert_eq!(config.confidence.decay_rate, 0.98);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let path = PathBuf::from("/nonexistent/config.toml");
        let config = AppConfig::load(&path).unwrap();
        assert_eq!(config.polling.interval_minutes, 10);
        assert_eq!(config.llm.provider, "ollama");
    }

    #[test]
    fn test_default_mascot_config() {
        let config = AppConfig::default();
        assert!(config.mascot.character_image.is_none());
    }

    #[test]
    fn test_mascot_config_serializes_to_json() {
        let config = MascotConfig {
            character_image: Some("/path/to/char.png".to_string()),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["character_image"], "/path/to/char.png");

        let default_config = MascotConfig::default();
        let json = serde_json::to_value(&default_config).unwrap();
        assert!(json["character_image"].is_null());
    }

    #[test]
    fn test_default_openai_base_url_is_none() {
        let config = AppConfig::default();
        assert!(config.llm.openai_base_url.is_none());
    }

    #[test]
    fn test_load_openai_base_url_from_toml() {
        let toml_content = r#"
[llm]
provider = "openai"
openai_base_url = "http://localhost:1234"
"#;
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(toml_content.as_bytes()).unwrap();
        let path = tmpfile.path().to_path_buf();

        let config = AppConfig::load(&path).unwrap();
        assert_eq!(
            config.llm.openai_base_url,
            Some("http://localhost:1234".to_string())
        );
    }

    #[test]
    fn test_load_mascot_config_from_toml() {
        let toml_content = r#"
[mascot]
character_image = "/path/to/character.png"
"#;
        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(toml_content.as_bytes()).unwrap();
        let path = tmpfile.path().to_path_buf();

        let config = AppConfig::load(&path).unwrap();
        assert_eq!(
            config.mascot.character_image,
            Some("/path/to/character.png".to_string())
        );
    }
}
