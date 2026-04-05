use tauri::Emitter;

pub const EVENT_THOUGHT: &str = "shiritagari-thought";
pub const EVENT_QUESTION: &str = "shiritagari-question";

pub fn emit_thought(app_handle: &tauri::AppHandle, inference: &str, confidence: f64) {
    app_handle
        .emit(
            EVENT_THOUGHT,
            &serde_json::json!({
                "inference": inference,
                "confidence": confidence,
            }),
        )
        .ok();
}

pub fn emit_question(app_handle: &tauri::AppHandle, question: &str) {
    app_handle.emit(EVENT_QUESTION, question).ok();
}
