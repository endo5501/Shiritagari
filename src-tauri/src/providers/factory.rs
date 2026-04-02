use std::env;

use crate::config::LlmConfig;
use super::claude::ClaudeProvider;
use super::ollama::OllamaProvider;
use super::openai::OpenAiProvider;
use super::types::LlmProvider;

pub fn create_provider(provider_name: &str, model: Option<&str>, api_key_env: Option<&str>, ollama_base_url: Option<&str>) -> Result<Box<dyn LlmProvider>, String> {
    match provider_name {
        "claude" => {
            let env_var = api_key_env.unwrap_or("ANTHROPIC_API_KEY");
            let api_key = env::var(env_var)
                .map_err(|_| format!("Environment variable {} not set", env_var))?;
            Ok(Box::new(ClaudeProvider::new(api_key, model.map(String::from))))
        }
        "openai" => {
            let env_var = api_key_env.unwrap_or("OPENAI_API_KEY");
            let api_key = env::var(env_var)
                .map_err(|_| format!("Environment variable {} not set", env_var))?;
            Ok(Box::new(OpenAiProvider::new(api_key, model.map(String::from))))
        }
        "ollama" => {
            Ok(Box::new(OllamaProvider::new(
                ollama_base_url.map(String::from),
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
    create_provider(provider, model, api_key_env, config.ollama_base_url.as_deref())
}

pub fn create_chat_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, String> {
    let provider = config.chat_provider.as_deref().unwrap_or(&config.provider);
    let model = config.chat_model.as_deref().or(config.model.as_deref());
    let api_key_env = config.chat_api_key_env.as_deref().or(config.api_key_env.as_deref());
    create_provider(provider, model, api_key_env, config.ollama_base_url.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ollama_provider() {
        let provider = create_provider("ollama", None, None, None).unwrap();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_create_unknown_provider() {
        let result = create_provider("unknown", None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config_creates_ollama() {
        let config = LlmConfig::default();
        let provider = create_inference_provider(&config).unwrap();
        assert_eq!(provider.name(), "ollama");
    }
}
