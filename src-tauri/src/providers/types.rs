use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceInput {
    pub events: Vec<EventSummary>,
    pub patterns: Vec<PatternSummary>,
    pub recent_episodes: Vec<EpisodeSummary>,
    pub user_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub app: String,
    pub title: String,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSummary {
    pub trigger_app: String,
    pub trigger_title: String,
    pub meaning: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeSummary {
    pub context: String,
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOutput {
    pub inference: String,
    pub confidence: f64,
    pub should_ask: bool,
    pub suggested_question: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn infer(&self, input: &InferenceInput) -> Result<InferenceOutput, String>;
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, String>;
    fn name(&self) -> &str;
}
