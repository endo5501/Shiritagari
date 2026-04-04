use async_trait::async_trait;
use log::debug;
use reqwest::Client;
use serde_json::json;

use super::json_extract::extract_json;
use super::types::*;

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        }
    }

    fn build_inference_prompt(&self, input: &InferenceInput) -> String {
        let events_text = format_grouped_events(&input.events);

        let patterns_text: Vec<String> = input
            .patterns
            .iter()
            .map(|p| format!("- {} + {} → {} (confidence: {})", p.trigger_app, p.trigger_title, p.meaning, p.confidence))
            .collect();

        let episodes_text: Vec<String> = input
            .recent_episodes
            .iter()
            .map(|e| format!("- [{}] Q: {} A: {}", e.context, e.question, e.answer))
            .collect();

        format!(
            r#"以下のユーザのPC操作ログから、ユーザが何をしているか推測してください。

## 直近の操作ログ
{}

## 既知のパターン
{}

## 最近のエピソード（過去の質問と回答）
{}

## ユーザプロファイル
{}

以下のJSON形式で回答してください:
{{
  "inference": "ユーザが何をしているかの推測（デスクトップマスコットの吹き出しに表示します。短く、独り言風に、40文字以内で。例:「Reactのコンポーネントを書いてるな...」）",
  "confidence": 0.0〜1.0の確信度,
  "should_ask": true/false（ユーザに質問すべきか）,
  "suggested_question": "質問文（should_askがtrueの場合）"
}}"#,
            events_text,
            if patterns_text.is_empty() { "なし".to_string() } else { patterns_text.join("\n") },
            if episodes_text.is_empty() { "なし".to_string() } else { episodes_text.join("\n") },
            input.user_profile.as_deref().unwrap_or("不明"),
        )
    }

    async fn call_api(&self, messages: &[serde_json::Value]) -> Result<String, String> {
        let body = json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": messages,
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Claude API request failed: {}", e))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("Claude API error ({}): {}", status, text));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

        parsed["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No text in Claude response".to_string())
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn infer(&self, input: &InferenceInput) -> Result<InferenceOutput, String> {
        let prompt = self.build_inference_prompt(input);
        debug!("Prompt size: {} chars", prompt.len());
        let messages = vec![json!({"role": "user", "content": prompt})];
        let response_text = self.call_api(&messages).await?;
        let json_text = extract_json(&response_text);

        serde_json::from_str(json_text).map_err(|e| {
            format!(
                "Failed to parse inference output: {}. Raw: {}",
                e, response_text
            )
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
                        MessageRole::System => "user",
                    },
                    "content": m.content,
                })
            })
            .collect();

        let content = self.call_api(&api_messages).await?;
        Ok(ChatResponse { content })
    }

    fn name(&self) -> &str {
        "claude"
    }
}
