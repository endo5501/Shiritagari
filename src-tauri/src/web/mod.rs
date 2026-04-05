pub mod handlers;

use std::sync::{Arc, Mutex};

use axum::Router;
use log::{info, warn};
use tokio::net::TcpListener;

use crate::memory::Database;

const DEFAULT_PORT: u16 = 14789;
const PORT_SEARCH_RANGE: u16 = 100;

#[derive(Clone)]
pub struct WebState {
    pub db: Arc<Mutex<Database>>,
}

pub fn build_router(db: Arc<Mutex<Database>>) -> Router {
    let state = WebState { db };

    Router::new()
        .route("/", axum::routing::get(handlers::patterns_page))
        .route("/episodes", axum::routing::get(handlers::episodes_page))
        .route("/profile", axum::routing::get(handlers::profile_page))
        .with_state(state)
}

async fn bind_available_port(start: u16) -> Option<TcpListener> {
    for port in start..start + PORT_SEARCH_RANGE {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)).await {
            return Some(listener);
        }
    }
    None
}

/// Start the web server and return the port it's listening on.
pub async fn start_server(db: Arc<Mutex<Database>>) -> Option<u16> {
    let listener = match bind_available_port(DEFAULT_PORT).await {
        Some(l) => l,
        None => {
            warn!("Could not find available port in range {}-{}", DEFAULT_PORT, DEFAULT_PORT + PORT_SEARCH_RANGE);
            return None;
        }
    };

    let port = listener.local_addr().unwrap().port();
    let router = build_router(db);

    info!("Knowledge Base web server started at http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            warn!("Web server error: {}", e);
        }
    });

    Some(port)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::memory::{NewEpisode, NewPattern};

    fn test_db() -> Arc<Mutex<Database>> {
        Arc::new(Mutex::new(Database::open_in_memory().unwrap()))
    }

    #[tokio::test]
    async fn test_patterns_page_empty() {
        let app = build_router(test_db());
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("まだパターンが学習されていません"));
    }

    #[tokio::test]
    async fn test_patterns_page_with_data() {
        let db = test_db();
        {
            let d = db.lock().unwrap();
            d.create_pattern(&NewPattern {
                trigger_app: "VS Code".to_string(),
                trigger_title_contains: "main.rs".to_string(),
                trigger_time_range: None,
                trigger_day_of_week: None,
                meaning: "Rust開発中".to_string(),
                confidence: 0.85,
                last_confirmed: "2026-04-01".to_string(),
            }).unwrap();
        }

        let app = build_router(db);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("VS Code"));
        assert!(html.contains("Rust開発中"));
        assert!(html.contains("Patterns (1件)"));
    }

    #[tokio::test]
    async fn test_episodes_page_empty() {
        let app = build_router(test_db());
        let response = app
            .oneshot(Request::builder().uri("/episodes").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("まだエピソードが記録されていません"));
    }

    #[tokio::test]
    async fn test_episodes_page_with_data() {
        let db = test_db();
        {
            let d = db.lock().unwrap();
            d.create_episode(&NewEpisode {
                timestamp: "2026-04-05T14:00:00".to_string(),
                context_app: "Chrome".to_string(),
                context_title: "Google Docs".to_string(),
                context_duration_minutes: Some(30.0),
                question: "何を書いていますか？".to_string(),
                answer: "企画書を作成中".to_string(),
                tags: vec!["docs".to_string()],
            }).unwrap();
        }

        let app = build_router(db);
        let response = app
            .oneshot(Request::builder().uri("/episodes").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Chrome"));
        assert!(html.contains("企画書を作成中"));
        assert!(html.contains("Episodes (1件)"));
    }

    #[tokio::test]
    async fn test_profile_page_empty() {
        let app = build_router(test_db());
        let response = app
            .oneshot(Request::builder().uri("/profile").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("プロフィールはまだ設定されていません"));
    }

    #[tokio::test]
    async fn test_profile_page_with_data() {
        let db = test_db();
        {
            let d = db.lock().unwrap();
            d.upsert_user_profile(
                Some("エンジニア"),
                &["Rust".to_string()],
                &["AI".to_string()],
                "テストメモ",
            ).unwrap();
        }

        let app = build_router(db);
        let response = app
            .oneshot(Request::builder().uri("/profile").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("エンジニア"));
        assert!(html.contains("Rust"));
    }

    #[tokio::test]
    async fn test_patterns_pagination() {
        let db = test_db();
        {
            let d = db.lock().unwrap();
            // Create 25 patterns to test pagination (page size is 20)
            for i in 0..25 {
                d.create_pattern(&NewPattern {
                    trigger_app: format!("App{}", i),
                    trigger_title_contains: "".to_string(),
                    trigger_time_range: None,
                    trigger_day_of_week: None,
                    meaning: format!("Meaning{}", i),
                    confidence: 0.9,
                    last_confirmed: "2026-04-01".to_string(),
                }).unwrap();
            }
        }

        let app = build_router(db.clone());
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Patterns (25件)"));
        assert!(html.contains("1 / 2")); // pagination shows page info
        assert!(html.contains("次へ"));

        // Page 2
        let app = build_router(db);
        let response = app
            .oneshot(Request::builder().uri("/?page=2").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("2 / 2"));
        assert!(html.contains("前へ"));
    }
}
