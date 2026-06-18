mod routes;
mod online_handlers;
mod torrent_handlers;

pub mod handlers;
pub use handlers::to_long_path;

use crate::scanner::MediaScanner;
use crate::storage::StorageManager;
use crate::torrent::TorrentManager;
use crate::transcoder::Transcoder;
use crate::douyin::DouyinParser;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageManager>,
    pub scanner: Arc<MediaScanner>,
    pub transcoder: Arc<Transcoder>,
    pub douyin: Arc<DouyinParser>,
    pub torrents: Arc<TorrentManager>,
}

pub struct Server {
    port: u16,
    state: AppState,
}

impl Server {
    pub fn new(port: u16, state: AppState) -> Self {
        Self { port, state }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let app = create_router(self.state.clone());

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("Server listening on {}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }
}

fn create_router(state: AppState) -> Router {
    // The Tauri shell serves the React bundle itself, so this fallback is
    // only relevant for the standalone web build. Using a non-conflicting
    // path prefix keeps it out of the way of the API routes.
    Router::new()
        .merge(routes::api_routes())
        .merge(routes::stream_routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let storage = Arc::new(StorageManager::new().unwrap());
        let scanner = Arc::new(MediaScanner::new());
        let transcoder = Arc::new(Transcoder::new(std::path::PathBuf::from("/tmp/transcode")));
        let douyin = Arc::new(DouyinParser::new());

        let state = AppState {
            storage,
            scanner,
            transcoder,
            douyin,
            torrents: Arc::new(TorrentManager::new(std::path::PathBuf::from("/tmp/torrents"))),
        };

        assert!(state.storage.load_media_library().is_ok());
    }
}
