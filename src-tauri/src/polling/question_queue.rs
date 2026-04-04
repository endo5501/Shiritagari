/// Manages question queueing during AFK and emission on user return.
///
/// When the user is AFK, questions are stored (latest only).
/// When the user returns from AFK, the pending question is emitted.
pub struct QuestionQueue {
    pending_question: Option<String>,
    was_afk: bool,
}

impl QuestionQueue {
    pub fn new() -> Self {
        Self {
            pending_question: None,
            was_afk: false,
        }
    }

    /// Process a question based on current AFK state.
    /// Returns Some(question) if it should be emitted immediately,
    /// or None if it was queued for later.
    pub fn process_question(&mut self, question: String, is_afk: bool) -> Option<String> {
        if is_afk {
            self.pending_question = Some(question);
            None
        } else {
            Some(question)
        }
    }

    /// Check if user returned from AFK and return any pending question.
    /// Should be called at the start of each cycle with current AFK state.
    pub fn check_afk_return(&mut self, is_afk: bool) -> Option<String> {
        if self.was_afk && !is_afk {
            self.pending_question.take()
        } else {
            None
        }
    }

    /// Update AFK state at the end of a cycle.
    pub fn update_afk_state(&mut self, is_afk: bool) {
        self.was_afk = is_afk;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_emitted_immediately_when_not_afk() {
        let mut queue = QuestionQueue::new();
        let result = queue.process_question("何をしていますか？".to_string(), false);
        assert_eq!(result, Some("何をしていますか？".to_string()));
    }

    #[test]
    fn test_question_queued_when_afk() {
        let mut queue = QuestionQueue::new();
        let result = queue.process_question("何をしていますか？".to_string(), true);
        assert_eq!(result, None);
        assert_eq!(queue.pending_question, Some("何をしていますか？".to_string()));
    }

    #[test]
    fn test_queued_question_overwritten_by_newer() {
        let mut queue = QuestionQueue::new();
        queue.process_question("古い質問".to_string(), true);
        queue.process_question("新しい質問".to_string(), true);
        assert_eq!(queue.pending_question, Some("新しい質問".to_string()));
    }

    #[test]
    fn test_pending_question_emitted_on_afk_return() {
        let mut queue = QuestionQueue::new();
        queue.was_afk = true;
        queue.pending_question = Some("AFK中の質問".to_string());

        let result = queue.check_afk_return(false); // not AFK now = returned
        assert_eq!(result, Some("AFK中の質問".to_string()));
        assert_eq!(queue.pending_question, None); // cleared
    }

    #[test]
    fn test_no_emission_on_afk_return_when_queue_empty() {
        let mut queue = QuestionQueue::new();
        queue.was_afk = true;

        let result = queue.check_afk_return(false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_no_emission_when_still_afk() {
        let mut queue = QuestionQueue::new();
        queue.was_afk = true;
        queue.pending_question = Some("AFK中の質問".to_string());

        let result = queue.check_afk_return(true); // still AFK
        assert_eq!(result, None);
        assert_eq!(queue.pending_question, Some("AFK中の質問".to_string())); // still queued
    }

    #[test]
    fn test_no_emission_when_not_previously_afk() {
        let mut queue = QuestionQueue::new();
        queue.was_afk = false;
        queue.pending_question = None;

        let result = queue.check_afk_return(false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_update_afk_state() {
        let mut queue = QuestionQueue::new();
        assert!(!queue.was_afk);

        queue.update_afk_state(true);
        assert!(queue.was_afk);

        queue.update_afk_state(false);
        assert!(!queue.was_afk);
    }

    #[test]
    fn test_full_afk_cycle() {
        let mut queue = QuestionQueue::new();

        // Cycle 1: User is active, question emitted immediately
        let q1 = queue.process_question("質問1".to_string(), false);
        assert_eq!(q1, Some("質問1".to_string()));
        queue.update_afk_state(false);

        // Cycle 2: User goes AFK, question queued
        let returned = queue.check_afk_return(true);
        assert_eq!(returned, None); // was not afk before
        let q2 = queue.process_question("質問2".to_string(), true);
        assert_eq!(q2, None);
        queue.update_afk_state(true);

        // Cycle 3: Still AFK, no new events (no question generated)
        let returned = queue.check_afk_return(true);
        assert_eq!(returned, None);
        queue.update_afk_state(true);

        // Cycle 4: User returns
        let returned = queue.check_afk_return(false);
        assert_eq!(returned, Some("質問2".to_string()));
        queue.update_afk_state(false);

        // Queue is now empty
        assert_eq!(queue.pending_question, None);
    }
}
