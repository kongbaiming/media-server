//! Generic online / live stream proxy.
//!
//! Provides a backend-side HTTP proxy that fetches an arbitrary m3u8 / mp4 /
//! webm / ts URL on behalf of the WebView, bypassing CORS and Referer
//! restrictions. Supports HTTP Range requests so the player can seek.

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Deserialize)]
pub struct ProxyQuery {
    pub url: String,
    /// Optional Referer header to send upstream. Some CDNs reject requests
    /// without one. Defaults to none.
    pub referer: Option<String>,
}

/// Result of probing a URL via HEAD (with GET fallback).
#[derive(Debug, Serialize)]
pub struct ProbeResult {
    pub url: String,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub accepts_ranges: bool,
    pub kind: StreamKind,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StreamKind {
    /// m3u8 HLS playlist (live or VOD)
    Hls,
    /// Direct video (mp4, webm, mov, mkv, ts, etc.)
    Direct,
    /// Audio-only (mp3, aac, m4a, ogg, ...)
    Audio,
    /// Anything else we don't recognize
    Other,
}

impl StreamKind {
    pub fn from_content_type(ct: Option<&str>, url: &str) -> Self {
        let url_lc = url.to_lowercase();
        if let Some(ct) = ct {
            let ct = ct.split(';').next().unwrap_or("").trim();
            if ct == "application/vnd.apple.mpegurl" || ct == "application/x-mpegurl" {
                return StreamKind::Hls;
            }
            if ct.starts_with("video/") {
                return StreamKind::Direct;
            }
            if ct.starts_with("audio/") {
                return StreamKind::Audio;
            }
        }
        if url_lc.contains(".m3u8") {
            StreamKind::Hls
        } else if url_lc.contains(".mp3")
            || url_lc.contains(".aac")
            || url_lc.contains(".m4a")
            || url_lc.contains(".ogg")
            || url_lc.contains(".flac")
        {
            StreamKind::Audio
        } else {
            StreamKind::Other
        }
    }
}

/// Reject obvious SSRF / non-HTTP(S) targets. The check is intentionally
/// lenient because users paste a wide variety of CDNs; we only block the
/// unsafe schemes.
pub fn is_proxyable_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_default()
}

/// Probe a URL: returns content-type, content-length, accepts-ranges, and a
/// coarse stream kind classification.
pub async fn probe(url: &str, referer: Option<&str>) -> Result<ProbeResult, String> {
    if !is_proxyable_url(url) {
        return Err("Only http:// and https:// URLs are supported".to_string());
    }
    let client = build_client();
    let mut req = client.head(url).header(
        "User-Agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    );
    if let Some(r) = referer {
        req = req.header("Referer", r);
    }
    let resp = req.send().await.map_err(|e| format!("HEAD failed: {}", e))?;
    let headers = resp.headers();
    let ct = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let cl = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    let accepts_ranges = headers
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_ascii_lowercase().contains("bytes"))
        .unwrap_or(false);
    let kind = StreamKind::from_content_type(ct.as_deref(), url);
    Ok(ProbeResult {
        url: url.to_string(),
        content_type: ct,
        content_length: cl,
        accepts_ranges,
        kind,
    })
}

/// Proxy a URL: streams the body to the client, copying the relevant response
/// headers (content-type, content-length, content-range, accept-ranges) and
/// forwarding any Range request the WebView sent.
pub async fn proxy(headers: HeaderMap, query: ProxyQuery) -> Response {
    if !is_proxyable_url(&query.url) {
        return (
            StatusCode::BAD_REQUEST,
            "Only http:// and https:// URLs are supported",
        )
            .into_response();
    }

    let client = build_client();
    let mut request = client.get(&query.url).header(
        "User-Agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    );

    if let Some(r) = query.referer.as_deref() {
        request = request.header("Referer", r);
    }

    if let Some(range) = headers.get("range") {
        if let Ok(range_val) = range.to_str() {
            request = request.header("Range", range_val);
        }
    }

    match request.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::BAD_GATEWAY);

            let mut builder = Response::builder().status(status);
            for key in [
                "content-type",
                "content-length",
                "content-range",
                "accept-ranges",
                "last-modified",
                "etag",
            ] {
                if let Some(value) = resp.headers().get(key) {
                    if let Ok(value_str) = value.to_str() {
                        builder = builder.header(key, value_str);
                    }
                }
            }
            // Wide-open CORS for proxied resources so the WebView can play
            // them when accessed directly (e.g. <video src=...>).
            builder = builder.header("Access-Control-Allow-Origin", "*");

            let stream = resp
                .bytes_stream()
                .map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

            match builder.body(Body::from_stream(stream)) {
                Ok(response) => response.into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
                    .into_response(),
            }
        }
        Err(e) => {
            warn!("Online proxy request failed for {}: {}", query.url, e);
            (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)).into_response()
        }
    }
}
