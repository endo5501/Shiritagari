use async_trait::async_trait;
use log::debug;
use reqwest::Client;
use serde_json::json;

use super::json_extract::extract_json;
use super::types::*;

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
        }
    }

    async fn call_api(&self, messages: &[serde_json::Value]) -> Result<String, String> {
        let body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": 1024,
        });

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("OpenAI API error ({}): {}", status, text));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

        parsed["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in OpenAI response".to_string())
    }
}

// Reuse the same inference prompt building logic
impl OpenAiProvider {
    fn build_inference_prompt(&self, input: &InferenceInput) -> String {
        let events_text = format_grouped_events(&input.events);

        format!(
            r#"以下のユーザのPC操作ログから、ユーザが何をしているか推測してください。

## 直近の操作ログ
{}

以下のJSON形式のみで回答してください（説明不要）:
{{
  "inference": "ユーザが何をしているかの推測",
  "confidence": 0.0〜1.0の確信度,
  "should_ask": true/false,
  "suggested_question": "質問文またはnull"
}}"#,
            events_text,
        )
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn infer(&self, input: &InferenceInput) -> Result<InferenceOutput, String> {
        let prompt = self.build_inference_prompt(input);
        debug!("Prompt size: {} chars", prompt.len());
        let messages = vec![json!({"role": "user", "content": prompt})];
        let response_text = self.call_api(&messages).await?;
        let json_text = extract_json(&response_text);

        serde_json::from_str(json_text).map_err(|e| {
            format!("Failed to parse inference output: {}. Raw: {}", e, response_text)
        })
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                json!({
                    "role": match m.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "system",
                    },
                    "content": m.content,
                })
            })
            .collect();

        let content = self.call_api(&api_messages).await?;
        Ok(ChatResponse { content })
    }

    fn name(&self) -> &str {
        "openai"
    }
}
