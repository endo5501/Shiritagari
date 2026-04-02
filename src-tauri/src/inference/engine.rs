use chrono::Utc;

use crate::config::AppConfig;
use crate::memory::confidence::{calculate_effective_confidence, determine_action, ConfidenceAction};
use crate::memory::{Database, NewSpeculation};
use crate::polling::AwEvent;
use crate::providers::redaction::{redact_text, should_include_app};
use crate::providers::types::*;

pub struct InferenceEngine {
    config: AppConfig,
}

#[derive(Debug)]
pub struct InferenceResult {
    pub inference: String,
    pub confidence: f64,
    pub action: ConfidenceAction,
    pub question: Option<String>,
}

impl InferenceEngine {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Process a batch of window events and determine the appropriate action.
    pub async fn process_events(
        &self,
        events: &[AwEvent],
        db: &Database,
        provider: &dyn LlmProvider,
    ) -> Result<Option<InferenceResult>, String> {
        if events.is_empty() {
            return Ok(None);
        }

        // Filter events by privacy settings
        let filtered: Vec<&AwEvent> = events
            .iter()
            .filter(|e| {
                e.app()
                    .map(|app| should_include_app(app, &self.config.privacy))
                    .unwrap_or(false)
            })
            .collect();

        if filtered.is_empty() {
            return Ok(None);
        }

        // Try pattern matching first
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        for event in &filtered {
            let app = match event.app() {
                Some(a) => a,
                None => continue,
            };
            let title = event.title().unwrap_or("");

            if let Ok(Some(pattern)) = db.find_matching_pattern(app, title) {
                let eff_conf = calculate_effective_confidence(
                    pattern.confidence,
                    &pattern.last_confirmed,
                    &now,
                    &self.config.confidence,
                );

                let action = determine_action(eff_conf, &self.config.confidence, true);

                match action {
                    ConfidenceAction::Silent => {
                        // Update confidence and move on
                        let new_conf = (pattern.confidence + 0.01).min(1.0);
                        db.update_pattern_confidence(pattern.id, new_conf, &now).ok();
                        continue;
                    }
                    ConfidenceAction::SoftDelete => {
                        db.soft_delete_pattern(pattern.id, &now).ok();
                    }
                    ConfidenceAction::ReAsk => {
                        let question = format!(
                            "以前「{}」と教えてもらった行動パターンですが、今も同じですか？（{}を使用中）",
                            pattern.meaning, app
                        );
                        return Ok(Some(InferenceResult {
                            inference: pattern.meaning.clone(),
                            confidence: eff_conf,
                            action: ConfidenceAction::ReAsk,
                            question: Some(question),
                        }));
                    }
                    _ => {}
                }
            }
        }

        // No pattern match or all were silent — use LLM inference
        let event_summaries: Vec<EventSummary> = filtered
            .iter()
            .map(|e| EventSummary {
                app: e.app().unwrap_or("unknown").to_string(),
                title: redact_text(e.title().unwrap_or(""), &self.config.privacy),
                duration_seconds: e.duration,
            })
            .collect();

        let recent_episodes = db
            .get_recent_episodes(5)
            .unwrap_or_default()
            .into_iter()
            .map(|ep| EpisodeSummary {
                context: format!("{} - {}", ep.context_app, ep.context_title),
                question: ep.question,
                answer: ep.answer,
            })
            .collect();

        let patterns = db
            .get_all_active_patterns()
            .unwrap_or_default()
            .into_iter()
            .map(|p| PatternSummary {
                trigger_app: p.trigger_app,
                trigger_title: p.trigger_title_contains,
                meaning: p.meaning,
                confidence: p.confidence,
            })
            .collect();

        let profile = db
            .get_user_profile()
            .ok()
            .flatten()
            .map(|p| {
                format!(
                    "職業: {}, スキル: {}, 関心: {}",
                    p.occupation.unwrap_or_else(|| "不明".to_string()),
                    p.skills.join(", "),
                    p.interests.join(", "),
                )
            });

        let input = InferenceInput {
            events: event_summaries,
            patterns,
            recent_episodes,
            user_profile: profile,
        };

        let output = provider.infer(&input).await?;

        let action = determine_action(output.confidence, &self.config.confidence, false);

        // Save speculation
        let primary_event = &filtered[0];
        let expires = (Utc::now() + chrono::Duration::days(3))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        db.create_speculation(&NewSpeculation {
            timestamp: now.clone(),
            observed_app: primary_event.app().unwrap_or("unknown").to_string(),
            observed_title: primary_event.title().unwrap_or("").to_string(),
            inference: output.inference.clone(),
            confidence: output.confidence,
            asked_user: output.should_ask,
            matched_pattern_id: None,
            expires_at: expires,
        })
        .ok();

        let question = if matches!(action, ConfidenceAction::Ask) {
            output.suggested_question.clone()
        } else {
            None
        };

        Ok(Some(InferenceResult {
            inference: output.inference,
            confidence: output.confidence,
            action,
            question,
        }))
    }
}
