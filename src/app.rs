use crate::douyin::DouyinParser;
use crate::scanner::MediaScanner;
use crate::server::{AppState, Server};
use crate::storage::StorageManager;
use crate::torrent::TorrentManager;
use crate::transcoder::{self, Transcoder};
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

/// Path of the file we tee panics into. Created on first panic. Lives
/// in `~/.mediavault/server.log` so the user can grep it after the fact,
/// especially when running under Tauri where stderr is discarded.
fn panic_log_path() -> Option<std::path::PathBuf> {
    let mut p = dirs::home_dir()?.join(".mediavault");
    let _ = std::fs::create_dir_all(&p);
    p.push("server.log");
    Some(p)
}

/// Append a line to the panic log, best-effort. We deliberately swallow
/// every error here — the last thing we want from the panic handler is
/// another panic.
fn append_panic_log(line: &str) {
    if let Some(path) = panic_log_path() {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{}", line);
        }
    }
}

/// Install a process-wide panic hook that always tees the panic message
/// (and location) to `~/.mediavault/server.log` before delegating to the
/// default hook. This is the only reliable way to diagnose a backend
/// crash under Tauri, where stderr is invisible to the user.
fn install_panic_hook() {
    use std::panic;
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let prev = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let payload = if let Some(s) = info.payload().downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "non-string panic payload".to_string()
            };
            let loc = info
                .location()
                .map(|l| format!("{}:{}", l.file(), l.line()))
                .unwrap_or_else(|| "<unknown>".to_string());
            let now = chrono::Utc::now().to_rfc3339();
            append_panic_log(&format!("[{}] PANIC at {}: {}", now, loc, payload));
            // Also let the default hook run so the message lands in
            // stderr when there is one (e.g. `cargo run -p media-server`).
            prev(info);
        }));
    });
}

pub fn init_tracing() {
    install_panic_hook();
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

/// Spawn the backend server on its own OS thread, supervised so a
/// panic or a transient bind error does not leave the Tauri shell
/// alive with a dead HTTP port. The Tauri WebView keeps trying to
/// fetch from `127.0.0.1:<port>`; this supervisor makes sure there is
/// always a server listening (modulo the back-off).
///
/// All panics land in `~/.mediavault/server.log` thanks to the hook
/// installed by `init_tracing`, so a crash is diagnosable after the
/// fact even if stderr is hidden (which it is under Tauri).
pub fn spawn_server() {
    std::thread::Builder::new()
        .name("mediavault-server".into())
        .spawn(|| {
            let mut backoff_ms: u64 = 200;
            loop {
                // Wrap the whole runtime in catch_unwind so a panic
                // in any task — including the auto-scan, the metadata
                // scraper worker, the torrent manager, or a request
                // handler — does not kill the supervisor.
                let outcome = std::panic::catch_unwind(|| {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(r) => r,
                        Err(e) => {
                            error!("[server] failed to build Tokio runtime: {}", e);
                            return Err(e.to_string());
                        }
                    };
                    rt.block_on(run_server()).map_err(|e| e.to_string())
                });

                match outcome {
                    Ok(Ok(())) => {
                        // Clean exit. This should not happen in
                        // practice (the server runs forever) but
                        // handle it gracefully by restarting.
                        warn!("[server] exited cleanly; restarting in 1s");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        backoff_ms = 200;
                    }
                    Ok(Err(e)) => {
                        // Logical error (e.g. port already in use).
                        // Retry with exponential backoff up to 30s.
                        error!(
                            "[server] returned error: {}; retrying in {}ms",
                            e, backoff_ms
                        );
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                        backoff_ms = (backoff_ms * 2).min(30_000);
                    }
                    Err(panic_payload) => {
                        // Panic. Try to extract the message; the
                        // panic_log_path hook has already written a
                        // proper line to ~/.mediavault/server.log.
                        let msg = if let Some(s) =
                            panic_payload.downcast_ref::<&'static str>()
                        {
                            (*s).to_string()
                        } else if let Some(s) =
                            panic_payload.downcast_ref::<String>()
                        {
                            s.clone()
                        } else {
                            "non-string panic payload".to_string()
                        };
                        error!(
                            "[server] PANIC: {}; restarting in {}ms (see ~/.mediavault/server.log for full backtrace)",
                            msg, backoff_ms
                        );
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                        backoff_ms = (backoff_ms * 2).min(30_000);
                    }
                }
            }
        })
        .expect("Failed to spawn server thread");
}



