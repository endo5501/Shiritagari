pub mod config;
pub mod inference;
pub mod memory;
pub mod polling;
pub mod providers;

use config::AppConfig;
use inference::InferenceEngine;
use memory::Database;
use polling::{AwClient, Poller};
use providers::factory::{create_chat_provider, create_inference_provider};
use providers::types::{ChatMessage, MessageRole};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{
    tray::TrayIconBuilder, Emitter, Manager, State,
    menu::{MenuBuilder, MenuItemBuilder},
};

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub config: AppConfig,
}

#[tauri::command]
async fn send_message(message: String, state: State<'_, AppState>) -> Result<String, String> {
    // Collect context from DB in a non-async block to avoid Send issues
    let context = {
        let db = state.db.lock().unwrap();
        let recent_specs = db.get_recent_speculations(5).unwrap_or_default();
        let profile = db.get_user_profile().ok().flatten();

        let mut ctx = String::new();
        if let Some(p) = &profile {
            ctx.push_str(&format!(
                "ユーザプロファイル: {}\n",
                p.occupation.as_deref().unwrap_or("不明")
            ));
        }
        if !recent_specs.is_empty() {
            ctx.push_str("最近の推測:\n");
            for s in &recent_specs {
                ctx.push_str(&format!("- {} ({}): {}\n", s.observed_app, s.timestamp, s.inference));
            }
        }
        ctx
    };

    let provider = create_chat_provider(&state.config.llm)?;

    let messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: format!(
                "あなたはShiritagariというAIアシスタントです。ユーザのPC操作を観察し学習しています。\n以下はあなたが知っているコンテキストです:\n{}",
                context
            ),
        },
        ChatMessage {
            role: MessageRole::User,
            content: message,
        },
    ];

    let response = provider.chat(&messages).await?;
    Ok(response.content)
}

#[tauri::command]
async fn answer_question(answer: String, question_context: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    db.create_episode(&memory::NewEpisode {
        timestamp: now,
        context_app: "Shiritagari".to_string(),
        context_title: question_context,
        context_duration_minutes: None,
        question: "AI質問".to_string(),
        answer,
        tags: vec![],
    })
    .map_err(|e| format!("Failed to save episode: {}", e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shiritagari")
        .join("config.toml");

    let config = AppConfig::load(&config_path).unwrap_or_default();

    let db_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shiritagari")
        .join("shiritagari.db");

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db = Database::open(&db_path).expect("Failed to open database");
    let db = Arc::new(Mutex::new(db));

    let app_state = AppState {
        db: db.clone(),
        config: config.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![send_message, answer_question])
        .setup(move |app| {
            // System tray
            let show = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            // Start polling in background
            let app_handle = app.handle().clone();
            let config_clone = config.clone();
            let db_clone = db.clone();

            tauri::async_runtime::spawn(async move {
                let aw_client = AwClient::default();
                let poller = Poller::new(aw_client, db_clone.clone(), config_clone.polling.interval_minutes);
                let engine = InferenceEngine::new(config_clone.clone());

                loop {
                    tokio::time::sleep(poller.interval_duration()).await;

                    if let Some(result) = poller.poll_once().await {
                        if result.skipped_afk || result.window_events.is_empty() {
                            continue;
                        }

                        let inference_provider = match create_inference_provider(&config_clone.llm) {
                            Ok(p) => p,
                            Err(_) => continue,
                        };

                        // Step 1: Sync - check patterns and gather context (holds lock briefly)
                        let match_result = {
                            let db = db_clone.lock().unwrap();
                            engine.check_patterns_and_gather_context(&result.window_events, &db)
                        };

                        let mut processed_ok = false;

                        match match_result {
                            Some(inference::engine::PatternMatchResult::Silent) => {
                                processed_ok = true;
                            },
                            Some(inference::engine::PatternMatchResult::ReAsk(ir)) => {
                                if let Some(question) = ir.question {
                                    app_handle.emit("shiritagari-question", &question).ok();
                                }
                                processed_ok = true;
                            }
                            Some(inference::engine::PatternMatchResult::NeedLlm(ctx)) => {
                                // Step 2: Async - call LLM (no lock held)
                                if let Ok(output) = engine.call_llm(&ctx, inference_provider.as_ref()).await {
                                    // Step 3: Sync - save results (holds lock briefly)
                                    let ir = {
                                        let db = db_clone.lock().unwrap();
                                        engine.save_speculation(&output, &ctx.primary_app, &ctx.primary_title, &db)
                                    };
                                    if let Some(question) = ir.question {
                                        app_handle.emit("shiritagari-question", &question).ok();
                                    }
                                    processed_ok = true;
                                }
                                // If LLM failed, processed_ok stays false — events will be retried next cycle
                            }
                            None => {
                                processed_ok = true;
                            }
                        }

                        // Only acknowledge events after successful processing
                        if processed_ok {
                            poller.acknowledge_events(&result.window_events, &result.window_bucket);
                        }

                        // Run periodic cleanup
                        {
                            let db = db_clone.lock().unwrap();
                            db.run_cleanup().ok();
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
