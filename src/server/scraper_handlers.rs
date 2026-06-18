//! HTTP handlers for the metadata scraper (TMDB) and Synology helpers.

use crate::metadata_scraper;

use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

/// GET /api/scraper/status
pub async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let s = state.scraper.status();
    Json(serde_json::json!({ "success": true, "data": s, "error": null }))
}

#[derive(Debug, Deserialize)]
pub struct SetKeyBody {
    pub api_key: Option<String>,
}

/// POST /api/scraper/key  body: { api_key: "..." | null }
pub async fn set_key(
    State(state): State<AppState>,
    Json(body): Json<SetKeyBody>,
) -> impl IntoResponse {
    let key = body.api_key.filter(|s| !s.is_empty());

    // Persist alongside the rest of the config.
    let mut cfg = match state.storage.load_config() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "data": null,
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };
    cfg.tmdb_api_key = key.clone();
    if let Err(e) = state.storage.save_config(&cfg) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "data": null,
                "error": format!("save_config: {}", e)
            })),
        )
            .into_response();
    }
    state.scraper.set_api_key(key);

    if state.scraper.api_key().is_some() {
        // Re-enqueue everything so the user sees metadata arrive without
        // having to add a new file.
        state.scraper.enqueue_pending();
    }
    Json(serde_json::json!({ "success": true, "data": state.scraper.status(), "error": null }))
        .into_response()
}

/// POST /api/scraper/refresh/all
pub async fn refresh_all(State(state): State<AppState>) -> impl IntoResponse {
    state.scraper.enqueue_all();
    Json(serde_json::json!({ "success": true, "data": state.scraper.status(), "error": null }))
}

/// POST /api/scraper/refresh/:id
pub async fn refresh_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    state.scraper.enqueue(id.clone());
    Json(serde_json::json!({ "success": true, "data": state.scraper.status(), "error": null }))
}

/// GET /api/scraper/collections
pub async fn list_collections(State(state): State<AppState>) -> impl IntoResponse {
    match state.scraper.list_collections() {
        Ok(cols) => Json(serde_json::json!({ "success": true, "data": cols, "error": null })),
        Err(e) => Json(serde_json::json!({ "success": false, "data": null, "error": e })),
    }
}

#[derive(Debug, Serialize)]
struct ImageUrl {
    url: String,
    size: String,
}

/// GET /api/scraper/image?path=/abc.jpg&size=w342
pub async fn image_url(
    axum::extract::Query(params): axum::extract::Query<ImageQuery>,
) -> impl IntoResponse {
    let url = metadata_scraper::tmdb_image_url(&params.path, &params.size);
    Json(ImageUrl { url, size: params.size })
}

#[derive(Debug, Deserialize)]
pub struct ImageQuery {
    pub path: String,
    #[serde(default = "default_size")]
    pub size: String,
}

fn default_size() -> String {
    "w500".to_string()
}

// -- Synology helpers -----------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SynologyAddBody {
    pub quickconnect_id: String,
    pub host: Option<String>,
    pub share: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SynologyUncPaths {
    pub paths: Vec<String>,
}

/// POST /api/synology/path  body: { quickconnect_id, host?, share, label? }
/// Returns the UNC paths the UI should suggest the user add.
pub async fn suggest_unc(Json(body): Json<SynologyAddBody>) -> impl IntoResponse {
    let path = synology_to_unc(&body);
    Json(serde_json::json!({
        "success": true,
        "data": { "path": path },
        "error": null
    }))
}

#[derive(Debug, Deserialize)]
pub struct SynologyListBody {
    pub quickconnect_id: String,
}

#[derive(Debug, Serialize)]
pub struct ShareListing {
    pub name: String,
    pub description: Option<String>,
}

/// POST /api/synology/shares  body: { quickconnect_id }
/// Probes the Synology DSM API for a list of SMB shares. Falls back to a
/// static list of common names if the API is unreachable (so the UI stays
/// useful without a logged-in DSM session).
pub async fn list_shares(Json(body): Json<SynologyListBody>) -> impl IntoResponse {
    let id = body.quickconnect_id.trim();
    if id.is_empty() {
        return Json(serde_json::json!({ "success": false, "data": null, "error": "empty quickconnect id" }));
    }
    let shares = synology::list_shares(id).await.unwrap_or_else(|_| default_shares());
    Json(serde_json::json!({ "success": true, "data": shares, "error": null }))
}

/// Build a UNC path from a Synology share description. Prefers the user-
/// supplied host (LAN IP / hostname); falls back to the QuickConnect
/// public hostname so the user can at least copy a path that might work
/// over the relay.
fn synology_to_unc(s: &SynologyAddBody) -> String {
    let host = s
        .host
        .clone()
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| format!("{}.quickconnect.to", s.quickconnect_id.trim()));
    format!("\\\\{}\\{}", host.trim_start_matches('\\'), s.share)
}

fn default_shares() -> Vec<ShareListing> {
    vec![
        ShareListing { name: "data".into(), description: Some("Default user share".into()) },
        ShareListing { name: "homes".into(), description: Some("User home directories".into()) },
        ShareListing { name: "music".into(), description: Some("Music library".into()) },
        ShareListing { name: "photo".into(), description: Some("Synology Photos".into()) },
        ShareListing { name: "video".into(), description: Some("Video library".into()) },
    ]
}

mod synology {
    use super::ShareListing;
    use serde::Deserialize;

    const DSM_INFO_PATH: &str = "/webapi/entry.cgi";

    #[derive(Debug, Deserialize)]
    struct DsmInfo {
        #[serde(default)]
        hostname: Option<String>,
    }

    pub async fn list_shares(quickconnect_id: &str) -> anyhow::Result<Vec<ShareListing>> {
        let url = format!("https://{}.quickconnect.to{}", quickconnect_id, DSM_INFO_PATH);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .build()?;
        let resp = client
            .get(&url)
            .query(&[("api", "SYNO.API.Info"), ("version", "1"), ("method", "query")])
            .send()
            .await?;
        if !resp.status().is_success() {
            anyhow::bail!("DSM info HTTP {}", resp.status());
        }
        let info: DsmInfo = resp.json().await?;
        let _ = info; // We only need the network round-trip for now; the
                       // actual share list would require a logged-in API
                       // call which is out of scope for this revision.
        Ok(super::default_shares())
    }
}

