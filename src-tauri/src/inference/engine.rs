use chrono::Utc;
use log::{debug, info};

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

/// Intermediate data collected from DB (all sync)
pub struct DbContext {
    pub event_summaries: Vec<EventSummary>,
    pub patterns_summaries: Vec<PatternSummary>,
    pub recent_episodes: Vec<EpisodeSummary>,
    pub profile: Option<String>,
    pub primary_app: String,
    pub primary_title: String,
}

/// Result from pattern matching (sync)
pub enum PatternMatchResult {
    Silent,
    ReAsk(InferenceResult),
    NeedLlm(DbContext),
}

impl InferenceEngine {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Synchronous: check patterns and gather DB context
    pub fn check_patterns_and_gather_context(
        &self,
        events: &[AwEvent],
        db: &Database,
    ) -> Option<PatternMatchResult> {
        if events.is_empty() {
            return None;
        }

        let filtered: Vec<&AwEvent> = events
            .iter()
            .filter(|e| {
                e.app()
                    .map(|app| should_include_app(app, &self.config.privacy))
                    .unwrap_or(false)
            })
            .collect();

        debug!(
            "Privacy filter: {} events -> {} after filtering",
            events.len(),
            filtered.len()
        );

        if filtered.is_empty() {
            info!("All events excluded by privacy filter");
            return None;
        }

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        // Try pattern matching
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

                info!(
                    "Pattern match: app={}, effective_confidence={:.3}, action={:?}",
                    app, eff_conf, action
                );

                match action {
                    ConfidenceAction::Silent => {
                        let new_conf = (pattern.confidence + 0.01).min(1.0);
                        db.update_pattern_confidence(pattern.id, new_conf, &now).ok();
                        return Some(PatternMatchResult::Silent);
                    }
                    ConfidenceAction::SoftDelete => {
                        db.soft_delete_pattern(pattern.id, &now).ok();
                    }
                    ConfidenceAction::ReAsk => {
                        let question = format!(
                            "以前「{}」と教えてもらった行動パターンですが、今も同じですか？（{}を使用中）",
                            pattern.meaning, app
                        );
                        return Some(PatternMatchResult::ReAsk(InferenceResult {
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

        info!("No pattern match, proceeding to LLM inference");

        // Gather context for LLM
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

        let patterns_summaries = db
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

        let primary_app = filtered[0].app().unwrap_or("unknown").to_string();
        let primary_title = filtered[0].title().unwrap_or("").to_string();

        Some(PatternMatchResult::NeedLlm(DbContext {
            event_summaries,
            patterns_summaries,
            recent_episodes,
            profile,
            primary_app,
            primary_title,
        }))
    }

    /// Async: call LLM for inference
    pub async fn call_llm(
        &self,
        ctx: &DbContext,
        provider: &dyn LlmProvider,
    ) -> Result<InferenceOutput, String> {
        let input = InferenceInput {
            events: ctx.event_summaries.clone(),
            patterns: ctx.patterns_summaries.clone(),
            recent_episodes: ctx.recent_episodes.clone(),
            user_profile: ctx.profile.clone(),
        };

        provider.infer(&input).await
    }

    /// Synchronous: save speculation to DB
    pub fn save_speculation(
        &self,
        output: &InferenceOutput,
        primary_app: &str,
        primary_title: &str,
        db: &Database,
    ) -> InferenceResult {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let action = determine_action(output.confidence, &self.config.confidence, false);

        info!(
            "LLM result: confidence={:.3}, should_ask={}, action={:?}",
            output.confidence, output.should_ask, action
        );

        let expires = (Utc::now() + chrono::Duration::days(3))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        db.create_speculation(&NewSpeculation {
            timestamp: now,
            observed_app: primary_app.to_string(),
            observed_title: primary_title.to_string(),
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

        InferenceResult {
            inference: output.inference.clone(),
            confidence: output.confidence,
            action,
            question,
        }
    }
}
