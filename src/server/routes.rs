use super::handlers::*;
use super::online_handlers;
use super::torrent_handlers;
use super::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

/// API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // 媒体库
        .route("/api/library", get(get_library))
        .route("/api/library/:id", get(get_media_detail))
        .route("/api/library/scan", post(scan_library))
        .route("/api/library/scan/progress", get(get_scan_progress))

        // 搜索
        .route("/api/search", get(search_media))

        // 收藏
        .route("/api/favorites", get(get_favorites))
        .route("/api/favorites/:id", post(toggle_favorite))

        // 播放历史
        .route("/api/history", get(get_history))
        .route("/api/history/douyin", post(add_douyin_history))
        .route("/api/history/:id/progress", post(update_progress))
        .route("/api/history/:id/progress", get(get_progress))

        // 转码
        .route("/api/transcode", post(start_transcode))
        .route("/api/transcode/:id", get(get_transcode_status))
        .route("/api/transcode/:id", delete(delete_transcode))

        // 抖音
        .route("/api/douyin/parse", post(parse_douyin_url))
        .route("/api/douyin/play", post(get_douyin_play_url))
        .route("/api/douyin/proxy", get(proxy_douyin_video))

        // 在线 / 直播流
        .route("/api/online/probe", get(online_handlers::probe))
        .route("/api/online/recent", get(online_handlers::recent))

        // 种子 / 磁力链接
        .route("/api/torrent/add", post(torrent_handlers::add))
        .route("/api/torrent/list", get(torrent_handlers::list))
        .route("/api/torrent/:id", get(torrent_handlers::status))
// 配置
        .route("/api/config", get(get_config))
        .route("/api/config", put(update_config))

        // 统计
        .route("/api/stats", get(get_statistics))

        // 系统信息
        .route("/api/system/info", get(get_system_info))
}

/// 流媒体 routes
pub fn stream_routes() -> Router<AppState> {
    Router::new()
        .route("/api/stream/:id/master.m3u8", get(get_hls_playlist))
        .route("/api/stream/:id/thumbnail", get(get_thumbnail))
        .route("/api/stream/:id/direct", get(direct_stream))
        .route("/api/stream/:id/segments/:file", get(get_hls_segment))
        // Generic online / live stream proxy.
        .route("/api/stream/online", get(online_handlers::proxy))
        // Torrent stream: serves the partially-downloaded file with
        // HTTP Range support. The frontend points a <video> at this.
        .route("/api/stream/torrent/:id", get(torrent_handlers::stream))
}

/// 静态文件 routes
pub fn static_routes() -> Router<AppState> {
    use tower_http::services::ServeDir;

    Router::new()
        .nest_service("/", ServeDir::new("static"))
}







