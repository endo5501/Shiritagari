use chrono::NaiveDateTime;

use crate::config::ConfidenceConfig;

pub fn calculate_effective_confidence(
    base_confidence: f64,
    last_confirmed: &str,
    now: &str,
    config: &ConfidenceConfig,
) -> f64 {
    let last = NaiveDateTime::parse_from_str(last_confirmed, "%Y-%m-%dT%H:%M:%S");
    let current = NaiveDateTime::parse_from_str(now, "%Y-%m-%dT%H:%M:%S");

    match (last, current) {
        (Ok(last_dt), Ok(now_dt)) => {
            let days = (now_dt - last_dt).num_days().max(0) as f64;
            base_confidence * config.decay_rate.powf(days)
        }
        _ => base_confidence,
    }
}

#[derive(Debug, PartialEq)]
pub enum ConfidenceAction {
    Silent,
    Ask,
    ReAsk,
    SoftDelete,
}

pub fn determine_action(effective_confidence: f64, config: &ConfidenceConfig, is_existing_pattern: bool) -> ConfidenceAction {
    if effective_confidence >= config.threshold_silent {
        ConfidenceAction::Silent
    } else if effective_confidence <= config.threshold_soft_delete {
        ConfidenceAction::SoftDelete
    } else if is_existing_pattern && effective_confidence <= config.threshold_re_ask {
        ConfidenceAction::ReAsk
    } else {
        ConfidenceAction::Ask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ConfidenceConfig {
        ConfidenceConfig {
            decay_rate: 0.99,
            threshold_silent: 0.8,
            threshold_re_ask: 0.5,
            threshold_soft_delete: 0.3,
        }
    }

    #[test]
    fn test_no_decay_same_day() {
        let config = default_config();
        let eff = calculate_effective_confidence(
            0.9,
            "2026-04-01T10:00:00",
            "2026-04-01T15:00:00",
            &config,
        );
        assert!((eff - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_decay_after_30_days() {
        let config = default_config();
        let eff = calculate_effective_confidence(
            0.95,
            "2026-03-01T10:00:00",
            "2026-03-31T10:00:00",
            &config,
        );
        // 0.95 * 0.99^30 ≈ 0.700
        assert!(eff < 0.75);
        assert!(eff > 0.65);
    }

    #[test]
    fn test_decay_to_soft_delete_threshold() {
        let config = default_config();
        // 0.9 * 0.99^110 ≈ 0.299
        let eff = calculate_effective_confidence(
            0.9,
            "2026-01-01T10:00:00",
            "2026-04-21T10:00:00",
            &config,
        );
        assert!(eff < config.threshold_soft_delete + 0.05);
    }

    #[test]
    fn test_action_silent() {
        let config = default_config();
        assert_eq!(determine_action(0.85, &config, false), ConfidenceAction::Silent);
        assert_eq!(determine_action(0.85, &config, true), ConfidenceAction::Silent);
    }

    #[test]
    fn test_action_ask_new() {
        let config = default_config();
        assert_eq!(determine_action(0.6, &config, false), ConfidenceAction::Ask);
    }

    #[test]
    fn test_action_re_ask_existing() {
        let config = default_config();
        assert_eq!(determine_action(0.45, &config, true), ConfidenceAction::ReAsk);
    }

    #[test]
    fn test_action_soft_delete() {
        let config = default_config();
        assert_eq!(determine_action(0.25, &config, false), ConfidenceAction::SoftDelete);
        assert_eq!(determine_action(0.25, &config, true), ConfidenceAction::SoftDelete);
    }
}
