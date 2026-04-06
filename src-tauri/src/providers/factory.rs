use std::env;

use crate::config::LlmConfig;
use super::claude::ClaudeProvider;
use super::ollama::OllamaProvider;
use super::openai::OpenAiProvider;
use super::types::LlmProvider;

pub fn create_provider(provider_name: &str, model: Option<&str>, api_key_env: Option<&str>, config: &LlmConfig) -> Result<Box<dyn LlmProvider>, String> {
    match provider_name {
        "claude" => {
            let env_var = api_key_env.unwrap_or("ANTHROPIC_API_KEY");
            let api_key = env::var(env_var)
                .map_err(|_| format!("Environment variable {} not set", env_var))?;
            Ok(Box::new(ClaudeProvider::new(api_key, model.map(String::from))))
        }
        "openai" => {
            let env_var = api_key_env.unwrap_or("OPENAI_API_KEY");
            let api_key = if config.openai_base_url.is_some() {
                env::var(env_var).unwrap_or_default()
            } else {
                env::var(env_var)
                    .map_err(|_| format!("Environment variable {} not set", env_var))?
            };
            Ok(Box::new(OpenAiProvider::new(api_key, model.map(String::from), config.openai_base_url.clone())))
        }
        "ollama" => {
            Ok(Box::new(OllamaProvider::new(
                config.ollama_base_url.clone(),
                model.map(String::from),
            )))
        }
        _ => Err(format!("Unknown LLM provider: {}", provider_name)),
    }
}

pub fn create_inference_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, String> {
    let provider = config.inference_provider.as_deref().unwrap_or(&config.provider);
    let model = config.inference_model.as_deref().or(config.model.as_deref());
    let api_key_env = config.inference_api_key_env.as_deref().or(config.api_key_env.as_deref());
    create_provider(provider, model, api_key_env, config)
}

pub fn create_chat_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, String> {
    let provider = config.chat_provider.as_deref().unwrap_or(&config.provider);
    let model = config.chat_model.as_deref().or(config.model.as_deref());
    let api_key_env = config.chat_api_key_env.as_deref().or(config.api_key_env.as_deref());
    create_provider(provider, model, api_key_env, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ollama_provider() {
        let config = LlmConfig::default();
        let provider = create_provider("ollama", None, None, &config).unwrap();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_create_unknown_provider() {
        let config = LlmConfig::default();
        let result = create_provider("unknown", None, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config_creates_ollama() {
        let config = LlmConfig::default();
        let provider = create_inference_provider(&config).unwrap();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_create_openai_with_custom_base_url() {
        let config = LlmConfig {
            openai_base_url: Some("http://localhost:1234".to_string()),
            ..LlmConfig::default()
        };
        let provider = create_provider("openai", None, None, &config).unwrap();
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_create_openai_config_with_base_url() {
        let config = LlmConfig {
            provider: "openai".to_string(),
            openai_base_url: Some("http://localhost:1234".to_string()),
            ..LlmConfig::default()
        };
        let provider = create_inference_provider(&config).unwrap();
        assert_eq!(provider.name(), "openai");

        let provider = create_chat_provider(&config).unwrap();
        assert_eq!(provider.name(), "openai");
    }
}
