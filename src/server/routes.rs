use super::handlers::*;
use super::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

/// API路由
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // 媒体库
        .route("/api/library", get(get_library))
        .route("/api/library/{id}", get(get_media_detail))
        .route("/api/library/{id}", delete(delete_media))
        .route("/api/library/scan", post(scan_library))
        .route("/api/library/scan/progress", get(get_scan_progress))

        // 搜索
        .route("/api/search", get(search_media))

        // 收藏
        .route("/api/favorites", get(get_favorites))
        .route("/api/favorites/{id}", post(toggle_favorite))

        // 播放历史
        .route("/api/history", get(get_history))
        .route("/api/history/douyin", post(add_douyin_history))
        .route("/api/history/{id}/progress", post(update_progress))
        .route("/api/history/{id}/progress", get(get_progress))

        // 转码
        .route("/api/transcode", post(start_transcode))
        .route("/api/transcode/{id}", get(get_transcode_status))
        .route("/api/transcode/{id}", delete(delete_transcode))

        // 抖音
        .route("/api/douyin/parse", post(parse_douyin_url))
        .route("/api/douyin/play", post(get_douyin_play_url))
        .route("/api/douyin/proxy", get(proxy_douyin_video))

        // 配置
        .route("/api/config", get(get_config))
        .route("/api/config", put(update_config))

        // 统计
        .route("/api/stats", get(get_statistics))

        // 系统信息
        .route("/api/system/info", get(get_system_info))
}

/// 流媒体路由
pub fn stream_routes() -> Router<AppState> {
    Router::new()
        .route("/api/stream/{id}/master.m3u8", get(get_hls_playlist))
        .route("/api/stream/{id}/thumbnail", get(get_thumbnail))
        .route("/api/stream/{id}/direct", get(direct_stream))
        .route("/api/stream/{id}/segments/{file}", get(get_hls_segment))
}

/// 静态文件路由
pub fn static_routes() -> Router<AppState> {
    use tower_http::services::ServeDir;

    Router::new()
        .nest_service("/", ServeDir::new("static"))
}
