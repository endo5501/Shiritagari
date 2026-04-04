use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceInput {
    pub events: Vec<AggregatedEvent>,
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
pub struct AggregatedEvent {
    pub app: String,
    pub title: String,
    pub total_duration_seconds: f64,
    pub last_active: String,
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

/// Format aggregated events as app-grouped text for LLM prompts.
/// Preserves insertion order (events should be pre-sorted by last_active desc).
pub fn format_grouped_events(events: &[AggregatedEvent]) -> String {
    // Preserve insertion order using Vec of (app, total, titles)
    let mut app_order: Vec<String> = Vec::new();
    let mut app_groups: std::collections::HashMap<String, (f64, Vec<&AggregatedEvent>)> =
        std::collections::HashMap::new();

    for event in events {
        let entry = app_groups
            .entry(event.app.clone())
            .or_insert_with(|| {
                app_order.push(event.app.clone());
                (0.0, Vec::new())
            });
        entry.0 += event.total_duration_seconds;
        entry.1.push(event);
    }

    let mut lines = Vec::new();
    for app in &app_order {
        let (total, titles) = &app_groups[app];
        lines.push(format!("{} (合計{:.0}秒)", app, total));
        for t in titles {
            lines.push(format!("  - {} ({:.0}秒)", t.title, t.total_duration_seconds));
        }
    }

    lines.join("\n")
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn infer(&self, input: &InferenceInput) -> Result<InferenceOutput, String>;
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, String>;
    fn name(&self) -> &str;
}
