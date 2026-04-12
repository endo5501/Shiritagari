use std::sync::atomic::Ordering;
use tauri::{Emitter, Manager};

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

pub fn bring_window_to_front(app_handle: &tauri::AppHandle) {
    let state = app_handle.state::<crate::AppState>();
    state.question_topmost.store(true, Ordering::Release);
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().ok();
        window.set_always_on_top(true).ok();
        window.set_focus().ok();
    }
}
