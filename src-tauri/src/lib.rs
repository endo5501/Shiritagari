pub mod commands;
pub mod config;
pub mod events;
pub mod inference;
pub mod memory;
pub mod polling;
pub mod providers;

use commands::router::CommandRouter;
use commands::types::CommandContext;
use config::AppConfig;
use inference::InferenceEngine;
use log::{debug, info, warn};
use memory::Database;
use polling::{AwClient, Poller, QuestionQueue};
use providers::factory::{create_chat_provider, create_inference_provider};
use providers::types::{ChatMessage, MessageRole};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{
    tray::TrayIconBuilder, Manager, State,
    menu::{MenuBuilder, MenuItemBuilder},
};

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub config: AppConfig,
    pub command_router: Arc<CommandRouter>,
}

fn bring_window_to_front(app_handle: &tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().ok();
        window.set_always_on_top(true).ok();
        window.set_focus().ok();
    }
}

#[tauri::command]
fn get_mascot_config(state: State<'_, AppState>) -> config::MascotConfig {
    state.config.mascot.clone()
}

#[tauri::command]
async fn send_message(
    message: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Check for slash command
    if message.trim_start().starts_with('/') {
        let ctx = CommandContext {
            app_handle,
            db: state.db.clone(),
            plugin_list: state.command_router.plugin_list(),
        };
        let result = state.command_router.dispatch(&message, &ctx).await?;
        return Ok(result.response);
    }

    // Collect context from DB in a non-async block to avoid Send issues
    let context = {
        let db = state.db.lock().unwrap();
        let recent_specs = db.get_recent_speculations(5).unwrap_or_default();
        let recent_episodes = db.get_recent_episodes(5).unwrap_or_default();
        let active_patterns = db.get_all_active_patterns().unwrap_or_default();
        let profile = db.get_user_profile().ok().flatten();

        let mut ctx = String::new();
        if let Some(p) = &profile {
            ctx.push_str(&format!(
                "ユーザプロファイル: {}\n",
                p.occupation.as_deref().unwrap_or("不明")
            ));
        }
        if !active_patterns.is_empty() {
            ctx.push_str("学習済みパターン:\n");
            for p in &active_patterns {
                ctx.push_str(&format!("- {} ({}): {} (confidence: {:.2})\n", p.trigger_app, p.trigger_title_contains, p.meaning, p.confidence));
            }
        }
        if !recent_episodes.is_empty() {
            ctx.push_str("最近のエピソード記憶:\n");
            for e in &recent_episodes {
                ctx.push_str(&format!("- {} ({}): Q:{} A:{}\n", e.context_app, e.timestamp, e.question, e.answer));
            }
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
async fn answer_question(
    answer: String,
    question_context: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
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

    // Restore normal window state after answering
    if let Some(window) = app_handle.get_webview_window("main") {
        window.set_always_on_top(false).ok();
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

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

    let mut command_router = CommandRouter::new();
    command_router.register(Box::new(commands::help::HelpPlugin));
    command_router.register(Box::new(commands::timer::TimerPlugin));
    let command_router = Arc::new(command_router);

    let app_state = AppState {
        db: db.clone(),
        config: config.clone(),
        command_router,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![get_mascot_config, send_message, answer_question])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide instead of closing so the app stays in system tray
                window.hide().ok();
                api.prevent_close();
            }
        })
        .setup(move |app| {
            // System tray
            let show = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .icon_as_template(true)
                .tooltip("Shiritagari")
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
                .build(app)?;

            // Start polling in background
            let app_handle = app.handle().clone();
            let config_clone = config.clone();
            let db_clone = db.clone();

            tauri::async_runtime::spawn(async move {
                let aw_client = AwClient::default();
                let poller = Poller::new(aw_client, db_clone.clone(), config_clone.polling.interval_minutes);
                let engine = InferenceEngine::new(config_clone.clone());

                // Check if external API needs first-use confirmation
                let inference_provider_name = config_clone.llm.inference_provider
                    .as_deref()
                    .unwrap_or(&config_clone.llm.provider);
                if inference_provider_name == "claude" || inference_provider_name == "openai" {
                    let confirmation_key = format!("external_api_{}", inference_provider_name);
                    let needs_confirmation = {
                        let db = db_clone.lock().unwrap();
                        !db.is_confirmed(&confirmation_key).unwrap_or(true)
                    };
                    if needs_confirmation {
                        events::emit_question(
                            &app_handle,
                            &format!(
                                "外部LLM API ({}) を推論に使用します。ウィンドウタイトル等の操作データが外部サーバーに送信されます。よろしいですか？（このメッセージは初回のみ表示されます）",
                                inference_provider_name
                            ),
                        );
                        let db = db_clone.lock().unwrap();
                        db.set_confirmed(&confirmation_key).ok();
                    }
                }

                let mut question_queue = QuestionQueue::new();

                loop {
                    debug!("Polling cycle started");

                    if let Some(result) = poller.poll_once().await {
                        // Check if user returned from AFK and emit any pending question
                        if let Some(pending) = question_queue.check_afk_return(result.is_afk) {
                            info!("User returned from AFK, emitting pending question");
                            events::emit_question(&app_handle, &pending);
                            bring_window_to_front(&app_handle);
                        }

                        if result.window_events.is_empty() {
                            debug!("Cycle skipped: no new events");
                            question_queue.update_afk_state(result.is_afk);
                            continue;
                        }

                        let inference_provider = match create_inference_provider(&config_clone.llm) {
                            Ok(p) => p,
                            Err(e) => {
                                warn!("Failed to create inference provider: {}", e);
                                question_queue.update_afk_state(result.is_afk);
                                continue;
                            }
                        };

                        // Step 1: Sync - check patterns and gather context (holds lock briefly)
                        let match_result = {
                            let db = db_clone.lock().unwrap();
                            engine.check_patterns_and_gather_context(&result.window_events, &db)
                        };

                        let mut processed_ok = false;

                        match match_result {
                            Some(inference::engine::PatternMatchResult::Silent) => {
                                debug!("Result: Silent (known pattern, high confidence)");
                                processed_ok = true;
                            },
                            Some(inference::engine::PatternMatchResult::ReAsk(ir)) => {
                                if let Some(ref question) = ir.question {
                                    match question_queue.process_question(question.clone(), result.is_afk) {
                                        Some(q) => {
                                            info!("Emitting re-ask question to frontend");
                                            events::emit_question(&app_handle, &q);
                                            bring_window_to_front(&app_handle);
                                        }
                                        None => {
                                            debug!("Re-ask question queued (user is AFK)");
                                        }
                                    }
                                }
                                processed_ok = true;
                            }
                            Some(inference::engine::PatternMatchResult::NeedLlm(ctx)) => {
                                info!("LLM inference starting (app: {}, events: {})", ctx.primary_app, ctx.event_summaries.len());
                                let llm_start = std::time::Instant::now();
                                // Step 2: Async - call LLM (no lock held)
                                match engine.call_llm(&ctx, inference_provider.as_ref()).await {
                                    Ok(output) => {
                                        let elapsed = llm_start.elapsed();
                                        info!("LLM inference completed in {:.1}s", elapsed.as_secs_f64());

                                        events::emit_thought(&app_handle, &output.inference, output.confidence);

                                        // Step 3: Sync - save results (holds lock briefly)
                                        let ir = {
                                            let db = db_clone.lock().unwrap();
                                            engine.save_speculation(&output, &ctx.primary_app, &ctx.primary_title, &db)
                                        };
                                        if let Some(ref question) = ir.question {
                                            match question_queue.process_question(question.clone(), result.is_afk) {
                                                Some(q) => {
                                                    info!("Emitting question to frontend");
                                                    events::emit_question(&app_handle, &q);
                                                    bring_window_to_front(&app_handle);
                                                }
                                                None => {
                                                    debug!("Question queued (user is AFK)");
                                                }
                                            }
                                        }
                                        processed_ok = true;
                                    }
                                    Err(e) => {
                                        let elapsed = llm_start.elapsed();
                                        warn!("LLM inference failed after {:.1}s: {}", elapsed.as_secs_f64(), e);
                                    }
                                }
                            }
                            None => {
                                debug!("No actionable events after filtering");
                                processed_ok = true;
                            }
                        }

                        // Only acknowledge events after successful processing
                        if processed_ok {
                            poller.acknowledge_events(&result.window_events, &result.window_bucket);
                            debug!("Events acknowledged, cycle complete");
                        }

                        // Update AFK state at end of cycle
                        question_queue.update_afk_state(result.is_afk);

                        // Run periodic cleanup
                        {
                            let db = db_clone.lock().unwrap();
                            db.run_cleanup().ok();
                        }
                    } else {
                        debug!("Cycle skipped: poll_once returned None");
                    }

                    tokio::time::sleep(poller.interval_duration()).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
