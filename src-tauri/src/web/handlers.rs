use askama::Template;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::memory::episodes::Episode;
use crate::memory::patterns::Pattern;
use crate::memory::profile::UserProfile;

use super::WebState;

const PAGE_SIZE: i64 = 20;

fn render_template(tmpl: &impl Template) -> Response {
    match tmpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Template error: {}", e)).into_response(),
    }
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

#[derive(Template)]
#[template(path = "patterns.html")]
pub struct PatternsTemplate {
    pub patterns: Vec<Pattern>,
    pub page: i64,
    pub total_pages: i64,
    pub total_count: i64,
}

#[derive(Template)]
#[template(path = "episodes.html")]
pub struct EpisodesTemplate {
    pub episodes: Vec<Episode>,
    pub page: i64,
    pub total_pages: i64,
    pub total_count: i64,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub profile: Option<UserProfile>,
}

pub async fn patterns_page(
    State(state): State<WebState>,
    Query(query): Query<PageQuery>,
) -> Response {
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * PAGE_SIZE;

    let (patterns, total_count) = {
        let db = state.db.lock().unwrap();
        let patterns = db.get_active_patterns_paginated(PAGE_SIZE, offset).unwrap_or_default();
        let total_count = db.count_active_patterns().unwrap_or(0);
        (patterns, total_count)
    };

    let total_pages = (total_count + PAGE_SIZE - 1) / PAGE_SIZE;

    render_template(&PatternsTemplate {
        patterns,
        page,
        total_pages: total_pages.max(1),
        total_count,
    })
}

pub async fn episodes_page(
    State(state): State<WebState>,
    Query(query): Query<PageQuery>,
) -> Response {
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * PAGE_SIZE;

    let (episodes, total_count) = {
        let db = state.db.lock().unwrap();
        let episodes = db.get_episodes_paginated(PAGE_SIZE, offset).unwrap_or_default();
        let total_count = db.count_episodes().unwrap_or(0);
        (episodes, total_count)
    };

    let total_pages = (total_count + PAGE_SIZE - 1) / PAGE_SIZE;

    render_template(&EpisodesTemplate {
        episodes,
        page,
        total_pages: total_pages.max(1),
        total_count,
    })
}

pub async fn profile_page(
    State(state): State<WebState>,
) -> Response {
    let profile = {
        let db = state.db.lock().unwrap();
        db.get_user_profile().unwrap_or(None)
    };

    render_template(&ProfileTemplate { profile })
}
