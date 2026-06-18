use super::AppState;
use crate::models::TranscodeQuality;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use futures_util::StreamExt;
use tokio::io::AsyncReadExt;
use serde::{Deserialize, Serialize};
use std::path::{Path as StdPath, PathBuf};
use tracing::{info, warn};

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct LibraryQuery {
    pub media_type: Option<String>,
    pub favorite: Option<bool>,
    pub sort_by: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ProgressQuery {
    pub progress: f64,
    pub duration: f64,
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TranscodeRequest {
    pub media_id: String,
    pub quality: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub library_paths: Option<Vec<String>>,
    pub auto_scan: Option<bool>,
    pub server_port: Option<u16>,
    /// Triple Option so the front-end can clear the key by sending null.
    #[serde(default)]
    pub tmdb_api_key: Option<Option<String>>,
}

/// API响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// 获取媒体库
pub async fn get_library(
    State(state): State<AppState>,
    Query(query): Query<LibraryQuery>,
) -> impl IntoResponse {
    match state.storage.load_media_library() {
        Ok(mut library) => {
            // 过滤类型
            if let Some(media_type) = &query.media_type {
                library.retain(|f| match media_type.as_str() {
                    "video" => f.is_video(),
                    "audio" => f.is_audio(),
                    _ => true,
                });
            }

            // 过滤收藏
            if let Some(favorite) = query.favorite {
                library.retain(|f| f.favorite == favorite);
            }

            // 排序
            if let Some(sort_by) = &query.sort_by {
                match sort_by.as_str() {
                    "name" => library.sort_by(|a, b| a.name.cmp(&b.name)),
                    "date" => library.sort_by(|a, b| b.modified_at.cmp(&a.modified_at)),
                    "size" => library.sort_by(|a, b| b.size.cmp(&a.size)),
                    "duration" => {
                        library.sort_by(|a, b| b.duration.unwrap_or(0.0).partial_cmp(&a.duration.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal))
                    }
                    _ => {}
                }
            }

            // 分页
            let page = query.page.unwrap_or(1);
            let per_page = query.per_page.unwrap_or(20);
            let total = library.len();
            let start = (page - 1) * per_page;
            let end = start + per_page;

            let paged = if start < library.len() {
                library[start..end.min(library.len())].to_vec()
            } else {
                Vec::new()
            };

            Json(ApiResponse::success(serde_json::json!({
                "items": paged,
                "total": total,
                "page": page,
                "per_page": per_page
            })))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取媒体详情
pub async fn get_media_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.storage.get_media_file(&id) {
        Ok(Some(file)) => Json(ApiResponse::success(file)),
        Ok(None) => Json(ApiResponse::error("Media not found".to_string())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 删除媒体
pub async fn delete_media(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.storage.delete_media_file(&id) {
        Ok(_) => Json(ApiResponse::success("Deleted")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 扫描媒体库
pub async fn scan_library(
    State(state): State<AppState>,
    Json(request): Json<ScanRequest>,
) -> impl IntoResponse {
    let paths: Vec<PathBuf> = request.paths.iter().map(PathBuf::from).collect();

    let scanner = state.scanner.clone();
    let storage = state.storage.clone();

    tokio::spawn(async move {
        match scanner.scan_multiple_directories(&paths).await {
            Ok(files) => {
                if let Err(e) = storage.save_media_library(&files) {
                    warn!("Failed to save library: {}", e);
                } else {
                    info!("Library scan completed: {} files", files.len());
                }
            }
            Err(e) => {
                warn!("Scan failed: {}", e);
            }
        }
    });

    Json(ApiResponse::success("Scan started"))
}

/// 获取扫描进度
pub async fn get_scan_progress(State(state): State<AppState>) -> impl IntoResponse {
    let progress = state.scanner.get_progress().await;
    Json(ApiResponse::success(progress))
}

/// 搜索媒体
pub async fn search_media(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    match state.storage.search_media(&query.q) {
        Ok(results) => Json(ApiResponse::success(results)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取收藏列表
pub async fn get_favorites(State(state): State<AppState>) -> impl IntoResponse {
    match state.storage.get_favorites() {
        Ok(favorites) => Json(ApiResponse::success(favorites)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 切换收藏状态
pub async fn toggle_favorite(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.storage.toggle_favorite(&id) {
        Ok(is_favorite) => Json(ApiResponse::success(is_favorite)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取播放历史
pub async fn get_history(State(state): State<AppState>) -> impl IntoResponse {
    match state.storage.get_recent_history() {
        Ok(history) => Json(ApiResponse::success(history)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 记录抖音播放历史
pub async fn add_douyin_history(
    State(state): State<AppState>,
    Json(video): Json<crate::douyin::DouyinVideo>,
) -> impl IntoResponse {
    match state.storage.add_douyin_history(&video, 0.0) {
        Ok(_) => Json(ApiResponse::success("History updated")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 更新播放进度
pub async fn update_progress(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ProgressQuery>,
) -> impl IntoResponse {
    match state.storage.update_play_progress(&id, query.progress, query.duration) {
        Ok(_) => Json(ApiResponse::success("Progress updated")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取播放进度
pub async fn get_progress(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.storage.get_media_file(&id) {
        Ok(Some(file)) => Json(ApiResponse::success(serde_json::json!({
            "progress": file.play_progress,
            "last_played": file.last_played
        }))),
        Ok(None) => Json(ApiResponse::error("Media not found".to_string())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 开始转码
pub async fn start_transcode(
    State(state): State<AppState>,
    Json(request): Json<TranscodeRequest>,
) -> impl IntoResponse {
    let quality = match request.quality.as_deref() {
        Some("high") => TranscodeQuality::High,
        Some("medium") => TranscodeQuality::Medium,
        Some("low") => TranscodeQuality::Low,
        _ => TranscodeQuality::Auto,
    };

    // 获取媒体文件路径
    let media = match state.storage.get_media_file(&request.media_id) {
        Ok(Some(file)) => file,
        Ok(None) => {
            return Json(ApiResponse::error("Media not found".to_string()));
        }
        Err(e) => {
            return Json(ApiResponse::error(e.to_string()));
        }
    };

    // 创建转码任务
    match state
        .transcoder
        .create_task(&request.media_id, &media.path, quality)
        .await
    {
        Ok(task_id) => {
            // 开始转码
            if let Err(e) = state
                .transcoder
                .start_transcode(&task_id, &media.path)
                .await
            {
                return Json(ApiResponse::error(e.to_string()));
            }

            Json(ApiResponse::success(task_id))
        }
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取转码状态
pub async fn get_transcode_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let status = state.transcoder.get_status(&id).await;
    let progress = state.transcoder.get_progress(&id).await;

    Json(ApiResponse::success(serde_json::json!({
        "status": status,
        "progress": progress
    })))
}

/// 删除转码任务
pub async fn delete_transcode(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.transcoder.delete_task(&id).await {
        Ok(_) => Json(ApiResponse::success("Deleted")),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取配置
pub async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    match state.storage.load_config() {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 更新配置
pub async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    match state.storage.update_config(|config| {
        if let Some(paths) = request.library_paths {
            config.library_paths = paths.iter().map(PathBuf::from).collect();
        }
        if let Some(auto_scan) = request.auto_scan {
            config.auto_scan = auto_scan;
        }
        if let Some(port) = request.server_port {
            config.server_port = port;
        }
        if let Some(key) = request.tmdb_api_key {
            config.tmdb_api_key = key.filter(|s| !s.is_empty());
            state.scraper.set_api_key(config.tmdb_api_key.clone());
            if config.tmdb_api_key.is_some() {
                state.scraper.enqueue_pending();
            }
        }
    }) {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取统计信息
pub async fn get_statistics(State(state): State<AppState>) -> impl IntoResponse {
    match state.storage.get_statistics() {
        Ok(stats) => Json(ApiResponse::success(stats)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// 获取系统信息
pub async fn get_system_info(State(_state): State<AppState>) -> impl IntoResponse {
    let ffmpeg_installed = crate::transcoder::check_ffmpeg_installed().await;

    Json(ApiResponse::success(serde_json::json!({
        "ffmpeg_installed": ffmpeg_installed,
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
    })))
}

/// HLS播放列表
pub async fn get_hls_playlist(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.transcoder.get_playlist_path(&id).await {
        Some(path) => {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => (
                    StatusCode::OK,
                    [("Content-Type", "application/vnd.apple.mpegurl")],
                    content,
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read playlist: {}", e),
                )
                    .into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, "Playlist not found").into_response(),
    }
}

/// 获取缩略图
pub async fn get_thumbnail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let storage = &state.storage;
    let thumbnail_path = storage.store.get_thumbnail_path(&id);

    if thumbnail_path.exists() {
        match tokio::fs::read(&thumbnail_path).await {
            Ok(data) => (
                StatusCode::OK,
                [("Content-Type", "image/x-portable-pixmap")],
                data,
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read thumbnail: {}", e),
            )
                .into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Thumbnail not found").into_response()
    }
}

/// Stream a local file directly to the WebView with full HTTP Range
/// support. This is the fallback path the player uses when HLS
/// transcoding is unavailable or the source format is already
/// browser-native (mp4 / webm). The lookup path is forced into the
/// `\\?\` verbatim form on Windows so UNC and other long paths
/// actually open (the regular path can hit a 10048/259 error
/// when the blocking-task wrapper converts through ANSI).
pub async fn direct_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let file = match state.storage.get_media_file(&id) {
        Ok(Some(f)) => f,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND, Json(serde_json::json!({"success": false, "data": null, "error": "Media not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "data": null, "error": e.to_string()})),
            )
                .into_response();
        }
    };

    let real_path = to_long_path(&file.path);
    let meta = match tokio::fs::metadata(&real_path).await {
        Ok(m) => m,
        Err(_) => match tokio::fs::metadata(&file.path).await {
            Ok(m) => m,
            Err(e) => {
                warn!("direct_stream: cannot stat {:?}: {}", real_path, e);
                return (
                    StatusCode::NOT_FOUND,
                    format!("Cannot access file: {}", e),
                )
                    .into_response();
            }
        },
    };
    let total = meta.len();

    let (start, end) = match headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
    {
        Some(range) => match parse_range(range, total) {
            Some(r) => r,
            None => {
                return Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_RANGE, format!("bytes */{}", total))
                    .body(Body::empty())
                    .unwrap();
            }
        },
        None => (0, total - 1),
    };
    let want = end - start + 1;

    let mut f = match tokio::fs::File::open(&real_path).await {
        Ok(f) => f,
        Err(_) => match tokio::fs::File::open(&file.path).await {
            Ok(f) => f,
            Err(e) => {
                warn!("direct_stream: cannot open {:?}: {}", real_path, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Cannot open file: {}", e),
                )
                    .into_response();
            }
        },
    };
    use tokio::io::AsyncSeekExt;
    if let Err(e) = f.seek(std::io::SeekFrom::Start(start)).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("seek failed: {}", e),
        )
            .into_response();
    }
    let limited = f.take(want);
    let stream = tokio_util::io::ReaderStream::new(limited).map(|res| {
        res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    });
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, content_type_for(&file.format))
        .header(header::CONTENT_LENGTH, want)
        .header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end, total),
        )
        .header(header::ACCEPT_RANGES, "bytes")
        .header("Access-Control-Allow-Origin", "*")
        .body(body)
        .unwrap()
}
pub async fn get_hls_segment(
    State(state): State<AppState>,
    Path((id, file)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.transcoder.get_playlist_path(&id).await {
        Some(playlist_path) => {
            let segment_path = playlist_path.parent().unwrap().join(&file);

            if segment_path.exists() {
                match tokio::fs::read(&segment_path).await {
                    Ok(data) => (
                        StatusCode::OK,
                        [("Content-Type", "video/MP2T")],
                        data,
                    )
                        .into_response(),
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read segment: {}", e),
                    )
                        .into_response(),
                }
            } else {
                (StatusCode::NOT_FOUND, "Segment not found").into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Transcode not found").into_response(),
    }
}

/// 抖音链接解析请求
#[derive(Debug, Deserialize)]
pub struct DouyinParseRequest {
    pub url: String,
}

/// 抖音播放地址请求
#[derive(Debug, Deserialize)]
pub struct DouyinPlayRequest {
    pub url: String,
}

/// 解析抖音链接
pub async fn parse_douyin_url(
    State(state): State<AppState>,
    Json(request): Json<DouyinParseRequest>,
) -> impl IntoResponse {
    info!("Received Douyin parse request, URL length: {}", request.url.len());

    // 首先尝试从文本中提取URL（无论是直接URL还是分享文本）
    let url = match crate::douyin::extract_url_from_text(&request.url) {
        Some(extracted) => {
            info!("Extracted URL from input: {}", extracted);
            extracted
        },
        None => {
            // 如果没有提取到URL，检查输入是否已经是完整的抖音URL
            if crate::douyin::is_douyin_url(&request.url) && request.url.starts_with("http") {
                info!("Input is a direct Douyin URL");
                request.url.clone()
            } else {
                warn!("Could not extract Douyin URL from input");
                return Json(ApiResponse::error(
                    "Not a valid Douyin URL. Please paste a Douyin share link or text.".to_string()
                ));
            }
        }
    };

    info!("Parsing Douyin URL: {}", url);

    match state.douyin.parse_share_url(&url).await {
        Ok(video) => {
            info!("Successfully parsed Douyin video: {}", video.title);
            Json(ApiResponse::success(video))
        },
        Err(e) => {
            warn!("Failed to parse Douyin URL: {}", e);
            Json(ApiResponse::error(format!("Failed to parse Douyin URL: {}", e)))
        }
    }
}

/// 获取抖音视频播放地址
pub async fn get_douyin_play_url(
    State(state): State<AppState>,
    Json(request): Json<DouyinPlayRequest>,
) -> impl IntoResponse {
    // 检查是否是抖音链接
    let url = if crate::douyin::is_douyin_url(&request.url) {
        request.url.clone()
    } else {
        crate::douyin::extract_url_from_text(&request.url)
            .unwrap_or_else(|| request.url.clone())
    };

    if !crate::douyin::is_douyin_url(&url) {
        return Json(ApiResponse::error("Not a valid Douyin URL".to_string()));
    }

    match state.douyin.get_play_url(&url).await {
        Ok(play_url) => Json(ApiResponse::success(serde_json::json!({
            "play_url": play_url
        }))),
        Err(e) => {
            warn!("Failed to get Douyin play URL: {}", e);
            Json(ApiResponse::error(format!("Failed to get play URL: {}", e)))
        }
    }
}

/// 抖音视频代理请求
#[derive(Debug, Deserialize)]
pub struct DouyinProxyQuery {
    pub url: String,
}

/// 代理抖音视频流（绕过 CDN Referer 限制）
pub async fn proxy_douyin_video(
    headers: HeaderMap,
    Query(query): Query<DouyinProxyQuery>,
) -> impl IntoResponse {
    if !crate::douyin::is_allowed_play_url(&query.url) {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid or unsupported video URL",
        )
            .into_response();
    }

    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create HTTP client: {}", e),
            )
                .into_response();
        }
    };

    let mut request = client
        .get(&query.url)
        .header("Referer", "https://www.douyin.com/")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );

    if let Some(range) = headers.get(header::RANGE) {
        if let Ok(range_val) = range.to_str() {
            request = request.header("Range", range_val);
        }
    }

    match request.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::BAD_GATEWAY);

            let mut builder = Response::builder().status(status);

            for key in ["content-type", "content-length", "content-range", "accept-ranges"] {
                if let Some(value) = resp.headers().get(key) {
                    if let Ok(value_str) = value.to_str() {
                        builder = builder.header(key, value_str);
                    }
                }
            }

            let stream = resp.bytes_stream().map(|result| {
                result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            });

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
            warn!("Douyin proxy request failed: {}", e);
            (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)).into_response()
        }
    }
}


// -- Helpers for streaming local files (including long / UNC paths) --------

/// On Windows, paths longer than MAX_PATH (260 chars) or UNC paths benefit
/// from the `\\?\` or `\\?\UNC\` prefix so the underlying Win32 calls
/// actually open the file. We add it transparently, but only once.
pub fn to_long_path(path: &StdPath) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if s.starts_with(r"\\?\") {
            return path.to_path_buf();
        }
        // If the user already used a verbatim path style, don't double-prefix.
        if let Some(rest) = s.strip_prefix(r"\\") {
            // UNC path: \\server\share\... -> \\?\UNC\server\share\...
            return PathBuf::from(format!(r"\\?\UNC\{}", rest));
        }
        return PathBuf::from(format!(r"\\?\{}", s));
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

/// Map a media-format extension to a Content-Type the browser understands.
fn content_type_for(format: &str) -> &'static str {
    match format.to_ascii_lowercase().as_str() {
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "wmv" => "video/x-ms-wmv",
        "flv" => "video/x-flv",
        "ts" | "m2ts" | "mts" => "video/mp2t",
        "mpg" | "mpeg" | "vob" => "video/mpeg",
        "3gp" | "3gpp" => "video/3gpp",
        "ogv" => "video/ogg",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "ogg" | "oga" => "audio/ogg",
        "opus" => "audio/opus",
        "wma" => "audio/x-ms-wma",
        "ape" => "audio/x-ape",
        "alac" => "audio/x-alac",
        _ => "application/octet-stream",
    }
}

fn parse_range(s: &str, total: u64) -> Option<(u64, u64)> {
    let s = s.strip_prefix("bytes=")?;
    let mut parts = s.splitn(2, '-');
    let start = parts.next()?.trim();
    let end = parts.next()?.trim();
    let start: u64 = if start.is_empty() {
        // Suffix form: -N -> last N bytes.
        let n: u64 = end.parse().ok()?;
        total.saturating_sub(n)
    } else {
        start.parse().ok()?
    };
    let end: u64 = if end.is_empty() {
        total - 1
    } else {
        u64::from_str_radix(end, 10).ok()?.min(total - 1)
    };
    if start > end || start >= total {
        return None;
    }
    Some((start, end))
}










