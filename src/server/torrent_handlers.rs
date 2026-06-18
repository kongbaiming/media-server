//! HTTP handlers for the torrent subsystem: add, list, status, stream.

use crate::server::AppState;
use crate::torrent;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RangeQuery {
    /// Optional 0-based file index; defaults to the largest video file.
    pub file: Option<usize>,
}

/// POST /api/torrent/add  body: { magnet?, torrent_b64? }
pub async fn add(
    State(state): State<AppState>,
    Json(body): Json<torrent::engine::AddRequest>,
) -> impl IntoResponse {
    match state.torrents.add(body).await {
        Ok(info) => axum::Json(serde_json::json!({
            "success": true,
            "data": info,
            "error": null,
        }))
        .into_response(),
        Err(e) => axum::Json(serde_json::json!({
            "success": false,
            "data": null,
            "error": e,
        }))
        .into_response(),
    }
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let items = state.torrents.list();
    axum::Json(serde_json::json!({
        "success": true,
        "data": items,
        "error": null,
    }))
}

pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.torrents.get(&id) {
        Some(s) => axum::Json(serde_json::json!({
            "success": true,
            "data": s.info("/api/stream/torrent".to_string()),
            "error": null,
        }))
        .into_response(),
        None => axum::Json(serde_json::json!({
            "success": false,
            "data": null,
            "error": format!("Torrent {} not found", id),
        }))
        .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    /// If true, also wipe the on-disk pieces.
    #[serde(default)]
    pub purge: bool,
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(_q): Query<DeleteQuery>,
) -> impl IntoResponse {
    let removed = state.torrents.remove(&id);
    axum::Json(serde_json::json!({
        "success": removed,
        "data": null,
        "error": if removed { None } else { Some(format!("Torrent {} not found", id)) },
    }))
}

/// GET /api/stream/torrent/{id}  - stream the partially-downloaded file
/// with HTTP Range support. The frontend points a <video> at this URL.
pub async fn stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    let session = match state.torrents.get(&id) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("Torrent {} not found", id)))
                .unwrap();
        }
    };

    let total = session.metadata_total_size();
    if total == 0 {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("Torrent metadata not yet resolved"))
            .unwrap();
    }

    // Parse the Range header. If absent, serve the full file from the
    // beginning.
    let (start, end) = match headers.get(header::RANGE).and_then(|v| v.to_str().ok()) {
        Some(range) => match parse_range(range, total) {
            Ok(r) => r,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_RANGE, format!("bytes */{}", total))
                    .body(Body::empty())
                    .unwrap();
            }
        },
        None => (0, total - 1),
    };

    match session.read_range(start, Some(end)).await {
        Ok((status, len, body)) => Response::builder()
            .status(StatusCode::from_u16(status).unwrap_or(StatusCode::PARTIAL_CONTENT))
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, len)
            .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, start + len - 1, total))
            .header(header::ACCEPT_RANGES, "bytes")
            .header("Access-Control-Allow-Origin", "*")
            .body(Body::from(body))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(e))
            .unwrap(),
    }
}

fn parse_range(s: &str, total: u64) -> Result<(u64, u64), String> {
    // We only accept the simple "bytes=START-END" or "bytes=START-" form.
    let s = s
        .strip_prefix("bytes=")
        .ok_or_else(|| "range must start with bytes=".to_string())?;
    let mut parts = s.splitn(2, '-');
    let start = parts
        .next()
        .ok_or_else(|| "missing start".to_string())?
        .trim();
    let end = parts
        .next()
        .ok_or_else(|| "missing end".to_string())?
        .trim();
    let start: u64 = if start.is_empty() {
        // Suffix form: "-N" means the last N bytes.
        let n: u64 = end.parse().map_err(|_| "bad suffix length".to_string())?;
        total.saturating_sub(n)
    } else {
        start.parse().map_err(|_| "bad start".to_string())?
    };
    let end: u64 = if end.is_empty() {
        total - 1
    } else {
        end.parse().map_err(|_| "bad end".to_string())?
    };
    if start > end || start >= total {
        return Err("out of bounds".to_string());
    }
    Ok((start, end.min(total - 1)))
}
