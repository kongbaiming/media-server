mod models;
mod scanner;
mod metadata;
mod transcoder;
mod server;
mod storage;
mod douyin;

use scanner::MediaScanner;
use storage::StorageManager;
use transcoder::Transcoder;
use douyin::DouyinParser;
use server::{AppState, Server};
use std::sync::Arc;
use tracing::{info, error, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting MediaVault Server...");

    // 检查FFmpeg（可选）
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
        warn!("Visit https://ffmpeg.org/download.html for installation instructions.");
    }

    // 初始化存储
    let storage = Arc::new(StorageManager::new()?);
    info!("Storage initialized");

    // 加载配置
    let config = storage.load_config()?;
    info!("Configuration loaded");

    // 初始化扫描器
    let scanner = Arc::new(MediaScanner::new());
    info!("Media scanner initialized");

    // 初始化转码器
    let storage_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".mediavault")
        .join("transcode");
    let transcoder = Arc::new(Transcoder::new(storage_dir));
    info!("Transcoder initialized");

    // 初始化抖音解析器
    let douyin = Arc::new(DouyinParser::new());
    info!("Douyin parser initialized");

    // 创建应用状态
    let state = AppState {
        storage: storage.clone(),
        scanner: scanner.clone(),
        transcoder: transcoder.clone(),
        douyin: douyin.clone(),
    };

    // 如果配置了自动扫描，启动后台扫描
    if config.auto_scan && !config.library_paths.is_empty() {
        let scanner = scanner.clone();
        let storage = storage.clone();
        let paths = config.library_paths.clone();

        tokio::spawn(async move {
            info!("Starting automatic library scan...");
            match scanner.scan_multiple_directories(&paths).await {
                Ok(files) => {
                    if let Err(e) = storage.save_media_library(&files) {
                        error!("Failed to save library: {}", e);
                    } else {
                        info!("Automatic scan completed: {} files found", files.len());
                    }
                }
                Err(e) => {
                    error!("Automatic scan failed: {}", e);
                }
            }
        });
    }

    // 启动HTTP服务器
    let server = Server::new(config.server_port, state);

    info!("Starting HTTP server on port {}...", config.server_port);

    if let Err(e) = server.start().await {
        error!("Server error: {}", e);
        return Err(e);
    }

    Ok(())
}
