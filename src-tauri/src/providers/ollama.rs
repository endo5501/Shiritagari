use async_trait::async_trait;
use log::debug;
use reqwest::Client;
use serde_json::json;

use super::json_extract::extract_json;
use super::types::*;

pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| DEFAULT_OLLAMA_BASE_URL.to_string()),
            model: model.unwrap_or_else(|| "llama3".to_string()),
        }
    }

    fn build_inference_prompt(&self, input: &InferenceInput) -> String {
        let events_text = format_grouped_events(&input.events);

        format!(
            r#"以下のユーザのPC操作ログから、ユーザが何をしているか推測してください。

## 直近の操作ログ
{}

以下のJSON形式のみで回答してください（説明不要）:
{{
  "inference": "ユーザが何をしているかの推測（デスクトップマスコットの吹き出しに表示します。短く、独り言風に、40文字以内で。例:「Reactのコンポーネントを書いてるな...」）",
  "confidence": 0.0〜1.0の確信度,
  "should_ask": true/false,
  "suggested_question": "質問文またはnull"
}}"#,
            events_text,
        )
    }

    async fn call_api(&self, messages: &[serde_json::Value]) -> Result<String, String> {
        let body = json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
        });

        let resp = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("Ollama error ({}): {}", status, text));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

        parsed["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in Ollama response".to_string())
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
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
        "ollama"
    }
}
