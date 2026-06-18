use crate::douyin::DouyinParser;
use crate::scanner::MediaScanner;
use crate::server::{AppState, Server};
use crate::storage::StorageManager;
use crate::torrent::TorrentManager;
use crate::transcoder::{self, Transcoder};
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}

pub async fn run_server() -> anyhow::Result<()> {
    init_tracing();

    info!("Starting MediaVault Server...");

    let ffmpeg_available = transcoder::check_ffmpeg_installed().await;
    if ffmpeg_available {
        match transcoder::get_ffmpeg_version().await {
            Ok(version) => info!("FFmpeg version: {}", version),
            Err(e) => warn!("Failed to get FFmpeg version: {}", e),
        }
        let hw_accel = transcoder::check_hardware_acceleration().await;
        info!("Hardware acceleration: {:?}", hw_accel);
    } else {
        warn!("FFmpeg is not installed. Transcoding features will be disabled.");
    }

    let storage = Arc::new(StorageManager::new()?);
    info!("Storage initialized");

    let config = storage.load_config()?;
    info!("Configuration loaded");

    let scanner = Arc::new(MediaScanner::new());
    info!("Media scanner initialized");

    let mediavault_home = dirs::home_dir()
        .unwrap_or_default()
        .join(".mediavault");
    let transcoder_dir = mediavault_home.join("transcode");
    let torrent_dir = mediavault_home.join("torrents");
    std::fs::create_dir_all(&torrent_dir).ok();

    let transcoder = Arc::new(Transcoder::new(transcoder_dir));
    info!("Transcoder initialized");

    let douyin = Arc::new(DouyinParser::new());
    info!("Douyin parser initialized");

    let torrents = Arc::new(TorrentManager::new(torrent_dir));
    info!("Torrent manager initialized");

    let scraper = crate::metadata_scraper::Scraper::new(storage.clone());
    info!("Metadata scraper initialized");
    // Seed the scraper's in-memory API key with whatever is on disk or
    // in the env. Settings -> Metadata can update it at runtime.
    scraper.set_api_key(
        config
            .tmdb_api_key
            .clone()
            .or_else(|| std::env::var("TMDB_API_KEY").ok().filter(|s| !s.is_empty())),
    );

    let state = AppState {
        storage: storage.clone(),
        scanner: scanner.clone(),
        transcoder: transcoder.clone(),
        douyin: douyin.clone(),
        torrents: torrents.clone(),
        scraper: scraper.clone(),
    };

    if config.auto_scan && !config.library_paths.is_empty() {
        let scanner = scanner.clone();
        let storage = storage.clone();
        let scraper = scraper.clone();
        let paths = config.library_paths.clone();

        tokio::spawn(async move {
            info!("Starting automatic library scan...");
            match scanner.scan_multiple_directories(&paths).await {
                Ok(files) => {
                    if let Err(e) = storage.save_media_library(&files) {
                        error!("Failed to save library: {}", e);
                    } else {
                        info!("Automatic scan completed: {} files found", files.len());
                        // Enqueue any items that need metadata for the background
                        // scraper. This is best-effort; failures are logged and
                        // skipped.
                        scraper.enqueue_pending();
                    }
                }
                Err(e) => {
                    error!("Automatic library scan failed: {}", e);
                }
            }
        });
    }

    let port = config.server_port;
    let server = Server::new(port, state);

    info!("Starting HTTP server on port {}...", port);
    server.start().await
}

pub fn spawn_server() {
    std::thread::Builder::new()
        .name("mediavault-server".into())
        .spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            if let Err(e) = rt.block_on(run_server()) {
                error!("Server error: {}", e);
            }
        })
        .expect("Failed to spawn server thread");
}
