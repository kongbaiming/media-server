//! HTTP handlers for the online / live stream proxy.

use crate::online;
use crate::server::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ProbeResponse {
    success: bool,
    data: Option<online::ProbeResult>,
    error: Option<String>,
}

pub async fn probe(Query(q): Query<online::ProxyQuery>) -> impl IntoResponse {
    match online::probe(&q.url, q.referer.as_deref()).await {
        Ok(result) => Json(ProbeResponse {
            success: true,
            data: Some(result),
            error: None,
        })
        .into_response(),
        Err(e) => Json(ProbeResponse {
            success: false,
            data: None,
            error: Some(e),
        })
        .into_response(),
    }
}

pub async fn proxy(headers: HeaderMap, Query(q): Query<online::ProxyQuery>) -> impl IntoResponse {
    online::proxy(headers, q).await.into_response()
}

#[derive(Debug, Serialize)]
struct HistoryItem {
    url: String,
    title: Option<String>,
    kind: Option<String>,
    last_played: String,
}

/// Return the recently-played online URLs. We surface existing
/// `HistorySource::Douyin` entries (which already store the share URL
/// and title) and any `Local` entries that look like a streamed source.
pub async fn recent(State(state): State<AppState>) -> impl IntoResponse {
    let history = state.storage.load_play_history().unwrap_or_default();
    let items: Vec<HistoryItem> = history
        .into_iter()
        .filter(|h| (matches!(h.source, crate::models::HistorySource::Douyin)))
        .map(|h| HistoryItem {
            url: h.share_url.unwrap_or_default(),
            title: h.title,
            kind: Some("douyin".to_string()),
            last_played: h.timestamp,
        })
        .collect();
    axum::Json(serde_json::json!({
        "success": true,
        "data": items,
        "error": null,
    }))
}


