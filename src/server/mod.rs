mod routes;
mod handlers;

use crate::scanner::MediaScanner;
use crate::storage::StorageManager;
use crate::transcoder::Transcoder;
use crate::douyin::DouyinParser;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageManager>,
    pub scanner: Arc<MediaScanner>,
    pub transcoder: Arc<Transcoder>,
    pub douyin: Arc<DouyinParser>,
}

/// HTTP服务器
pub struct Server {
    port: u16,
    state: AppState,
}

impl Server {
    pub fn new(port: u16, state: AppState) -> Self {
        Self { port, state }
    }

    /// 启动服务器
    pub async fn start(&self) -> anyhow::Result<()> {
        let app = create_router(self.state.clone());

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("Server listening on {}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// 创建路由
fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(routes::api_routes())
        .merge(routes::stream_routes())
        .merge(routes::static_routes())
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
        };

        assert!(state.storage.load_media_library().is_ok());
    }
}
