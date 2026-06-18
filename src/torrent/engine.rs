//! One torrent session: parses metadata, downloads pieces from HTTP web
//! seeds (BEP 19) and serves the resulting bytes to the WebView with
//! HTTP Range support.
//!
//! Scope: single-file torrents with at least one web seed. Multi-file
//! torrents are flattened to a single concatenated buffer; peer wire /
//! DHT / UDP trackers are out of scope.

use crate::torrent::magnet::Magnet;
use lava_torrent::torrent::v1::Torrent;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Resolving,
    Downloading,
    Ready,
    Failed,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileInfo {
    pub path: String,
    pub length: u64,
    pub downloaded: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub status: SessionStatus,
    pub progress: f64,
    pub downloaded: u64,
    pub total: u64,
    pub download_speed_bps: u64,
    pub info_hash: String,
    pub files: Vec<FileInfo>,
    pub error: Option<String>,
    pub stream_url: String,
}

#[derive(Default)]
struct Metadata {
    name: String,
    total_size: u64,
    piece_length: u64,
    piece_hashes: Vec<[u8; 20]>,
    web_seeds: Vec<String>,
    files: Vec<FileInfo>,
}

pub struct TorrentSession {
    pub id: String,
    pub info_hash: String,
    pub status: parking_lot::Mutex<SessionStatus>,
    pub error: parking_lot::Mutex<Option<String>>,
    metadata: RwLock<Metadata>,
    pub downloaded: AtomicU64,
    last_speed_sample: parking_lot::Mutex<SpeedSample>,
    pub pieces: parking_lot::Mutex<Vec<bool>>,
    pub data_path: PathBuf,
    pub write_lock: AsyncMutex<()>,
}

#[derive(Clone, Copy)]
struct SpeedSample {
    bytes: u64,
    at: Instant,
}

impl TorrentSession {
    pub async fn start(
        id: String,
        magnet: &Magnet,
        uploaded_torrent: Option<Vec<u8>>,
        storage_dir: PathBuf,
    ) -> Result<Arc<Self>, String> {
        let data_path = storage_dir.join(&id).join("data.bin");
        if let Some(parent) = data_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {}", e))?;
        }

        let session = Arc::new(Self {
            id: id.clone(),
            info_hash: magnet.normalized_info_hash().unwrap_or_default(),
            status: parking_lot::Mutex::new(SessionStatus::Resolving),
            error: parking_lot::Mutex::new(None),
            metadata: RwLock::new(Metadata {
                name: magnet
                    .display_name
                    .clone()
                    .unwrap_or_else(|| "torrent".to_string()),
                ..Default::default()
            }),
            downloaded: AtomicU64::new(0),
            last_speed_sample: parking_lot::Mutex::new(SpeedSample {
                bytes: 0,
                at: Instant::now(),
            }),
            pieces: parking_lot::Mutex::new(Vec::new()),
            data_path,
            write_lock: AsyncMutex::new(()),
        });

        let session_for_task = session.clone();
        let magnet_owned = magnet.clone();
        let id_for_task = id.clone();
        tokio::spawn(async move {
            let result = resolve_and_populate(&session_for_task, &magnet_owned, uploaded_torrent).await;
            if let Err(e) = result {
                warn!("[torrent:{}] resolve failed: {}", id_for_task, e);
                *session_for_task.error.lock() = Some(e);
                *session_for_task.status.lock() = SessionStatus::Failed;
                return;
            }
            *session_for_task.pieces.lock() =
                vec![false; session_for_task.metadata.read().piece_hashes.len()];
            *session_for_task.status.lock() = SessionStatus::Downloading;
            start_piece_download_loop(session_for_task.clone());
        });

        Ok(session)
    }

    pub fn info(&self, base_stream_url: String) -> SessionInfo {
        let meta = self.metadata.read();
        let downloaded = self.downloaded.load(Ordering::Relaxed);
        let total = meta.total_size.max(downloaded);
        let progress = if total == 0 {
            0.0
        } else {
            (downloaded as f64 / total as f64) * 100.0
        };
        let speed = {
            let mut s = self.last_speed_sample.lock();
            let now = Instant::now();
            let elapsed = now.duration_since(s.at).as_secs_f64().max(0.001);
            let delta = downloaded.saturating_sub(s.bytes);
            s.bytes = downloaded;
            s.at = now;
            (delta as f64 / elapsed) as u64
        };
        SessionInfo {
            id: self.id.clone(),
            name: meta.name.clone(),
            status: *self.status.lock(),
            progress,
            downloaded,
            total,
            download_speed_bps: speed,
            info_hash: self.info_hash.clone(),
            files: meta.files.clone(),
            error: self.error.lock().clone(),
            stream_url: format!("{}/{}", base_stream_url, self.id),
        }
    }

    pub async fn read_range(
        &self,
        start: u64,
        end: Option<u64>,
    ) -> Result<(u16, u64, Vec<u8>), String> {
        let total = self.metadata.read().total_size;
        if total == 0 {
            return Err("Torrent metadata not yet loaded".to_string());
        }
        let end = end.unwrap_or(total - 1).min(total - 1);
        if start > end || start >= total {
            return Err(format!("Range out of bounds: {}..={}", start, end));
        }
        let want = end - start + 1;

        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        let mut file = match tokio::fs::File::open(&self.data_path).await {
            Ok(f) => f,
            Err(_) => return Ok((206, want, vec![0u8; want as usize])),
        };
        if file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
            return Ok((206, want, vec![0u8; want as usize]));
        }
        let mut buf = vec![0u8; want as usize];
        let mut got = 0usize;
        while got < want as usize {
            match file.read(&mut buf[got..]).await {
                Ok(0) => break,
                Ok(n) => got += n,
                Err(e) => return Err(format!("read failed: {}", e)),
            }
        }
        if got < want as usize {
            for b in &mut buf[got..] {
                *b = 0;
            }
        }
        Ok((206, want, buf))
    }

    pub fn metadata_total_size(&self) -> u64 {
        self.metadata.read().total_size
    }

    pub fn metadata_name(&self) -> String {
        self.metadata.read().name.clone()
    }
}

async fn resolve_and_populate(
    session: &Arc<TorrentSession>,
    magnet: &Magnet,
    uploaded: Option<Vec<u8>>,
) -> Result<(), String> {
    let torrent_bytes: Vec<u8> = if let Some(b) = uploaded {
        b
    } else if !magnet.xs.is_empty() {
        fetch_url(&magnet.xs[0]).await?
    } else if let Some(hash) = magnet.normalized_info_hash() {
        let mut last_err = String::new();
        let mut got: Option<Vec<u8>> = None;
        for cache in [
            format!("https://itorrents.org/torrent/{}.torrent", hash),
            format!("https://btcache.me/torrent/{}", hash),
        ] {
            match fetch_url(&cache).await {
                Ok(b) => {
                    got = Some(b);
                    break;
                }
                Err(e) => {
                    last_err = e;
                    continue;
                }
            }
        }
        match got {
            Some(b) => {
                // Public metadata caches often return an HTML landing page
                // instead of a 404; treat that as a cache miss.
                let head: &[u8] = b.as_slice();
                if head.starts_with(b"<") || head.starts_with(b"<!") {
                    return Err("Torrent metadata not in public cache. Try uploading the .torrent file directly.".to_string());
                }
                b
            }
            None => {
                return Err(format!(
                    "Could not fetch torrent metadata from public caches ({}). \
                     Provide a .torrent file or a magnet with xs= pointing at one.",
                    last_err
                ));
            }
        }
    } else {
        return Err("No metadata source available".to_string());
    };

    let torrent = Torrent::read_from_bytes(&torrent_bytes)
        .map_err(|e| format!("Invalid .torrent file: {}", e))?;

    let piece_length = torrent.piece_length as u64;
    if torrent.pieces.is_empty() {
        return Err("Torrent has no pieces".to_string());
    }
    let piece_hashes: Vec<[u8; 20]> = torrent
        .pieces
        .iter()
        .filter_map(|p| {
            if p.len() == 20 {
                let mut h = [0u8; 20];
                h.copy_from_slice(p);
                Some(h)
            } else {
                None
            }
        })
        .collect();
    if piece_hashes.len() != torrent.pieces.len() {
        return Err("Malformed piece hashes".to_string());
    }

    let mut web_seeds = magnet.web_seeds.clone();
    if let Some(dict) = torrent.extra_fields.as_ref() {
        if let Some(extra) = dict.get("url-list") {
            extract_string_list(extra, &mut web_seeds);
        }
        if let Some(extra) = dict.get("httpseeds") {
            extract_string_list(extra, &mut web_seeds);
        }
    }
    if web_seeds.is_empty() {
        return Err(
            "Torrent has no HTTP web seeds (BEP 17/19). Peer-only torrents are not supported."
                .to_string(),
        );
    }

    let total = if let Some(files) = &torrent.files {
        files.iter().map(|f| f.length as u64).sum()
    } else {
        torrent.length as u64
    };
    let files = if let Some(list) = &torrent.files {
        list.iter()
            .map(|f| FileInfo {
                path: f.path.to_string_lossy().to_string(),
                length: f.length as u64,
                downloaded: 0,
            })
            .collect()
    } else {
        vec![FileInfo {
            path: torrent.name.clone(),
            length: torrent.length as u64,
            downloaded: 0,
        }]
    };

    let name = if torrent.name.is_empty() {
        magnet
            .display_name
            .clone()
            .unwrap_or_else(|| "torrent".to_string())
    } else {
        torrent.name.clone()
    };

    {
        let mut meta = session.metadata.write();
        meta.name = name;
        meta.total_size = total;
        meta.piece_length = piece_length;
        meta.piece_hashes = piece_hashes;
        meta.web_seeds = web_seeds;
        meta.files = files;
    }

    info!(
        "[torrent:{}] resolved: {} ({} bytes, {} pieces, {} web seeds)",
        session.id,
        session.metadata.read().name,
        session.metadata.read().total_size,
        session.metadata.read().piece_hashes.len(),
        session.metadata.read().web_seeds.len()
    );

    Ok(())
}

fn extract_string_list(elem: &lava_torrent::bencode::BencodeElem, out: &mut Vec<String>) {
    use lava_torrent::bencode::BencodeElem;
    match elem {
        BencodeElem::String(s) => {
            // `s` is `&String`; convert to bytes before UTF-8 check.
            if !s.is_empty() {
                out.push(s.clone());
            }
        }
        BencodeElem::List(list) => {
            for e in list {
                extract_string_list(e, out);
            }
        }
        _ => {}
    }
}

fn start_piece_download_loop(session: Arc<TorrentSession>) {
    tokio::spawn(async move {
        let piece_count = session.metadata.read().piece_hashes.len();
        let concurrency: usize = 3;
        let mut next_piece: usize = 0;
        let mut handles = Vec::new();
        loop {
            let status = *session.status.lock();
            if status == SessionStatus::Ready || status == SessionStatus::Failed {
                break;
            }
            if next_piece >= piece_count {
                for h in handles.drain(..) {
                    let _ = h.await;
                }
                if next_piece >= piece_count {
                    *session.status.lock() = SessionStatus::Ready;
                    info!("[torrent:{}] all pieces downloaded", session.id);
                    break;
                }
            }
            while handles.len() < concurrency && next_piece < piece_count {
                let s = session.clone();
                let idx = next_piece;
                next_piece += 1;
                handles.push(tokio::spawn(async move {
                    download_piece(s, idx).await;
                }));
            }
            if !handles.is_empty() {
                let (result, _idx, remaining) = futures_util::future::select_all(handles).await;
                if let Err(e) = result {
                    warn!("[torrent:{}] piece task panicked: {}", session.id, e);
                }
                handles = remaining;
            }
        }
    });
}

async fn download_piece(session: Arc<TorrentSession>, piece_idx: usize) {
    // Snapshot the metadata we need first; parking_lot guards are not
    // Send and would otherwise be held across awaits below.
    let (piece_length, total, this_piece_len, expected_hash, web_seeds) = {
        let meta = session.metadata.read();
        if piece_idx >= meta.piece_hashes.len() {
            return;
        }
        {
            let pieces = session.pieces.lock();
            if *pieces.get(piece_idx).unwrap_or(&true) {
                return;
            }
        }
        let piece_offset = piece_idx as u64 * meta.piece_length;
        let this_piece_len = if piece_idx == meta.piece_hashes.len() - 1 {
            meta.total_size - piece_offset
        } else {
            meta.piece_length
        };
        (
            meta.piece_length,
            meta.total_size,
            this_piece_len,
            meta.piece_hashes[piece_idx],
            meta.web_seeds.clone(),
        )
    };
    let _ = total; // silence unused warning
    let piece_offset = piece_idx as u64 * piece_length;

    let mut last_err = String::new();
    'outer: for attempt in 0..3 {
        for seed in web_seeds.iter() {
            match fetch_piece(seed, piece_offset, this_piece_len).await {
                Ok(bytes) => {
                    if bytes.len() as u64 != this_piece_len {
                        last_err = format!(
                            "short read from {} ({} < {})",
                            seed,
                            bytes.len(),
                            this_piece_len
                        );
                        continue;
                    }
                    let mut hasher = Sha1::new();
                    hasher.update(&bytes);
                    let actual = hasher.finalize();
                    if actual.as_slice() != &expected_hash {
                        last_err = format!("SHA1 mismatch from {}", seed);
                        continue;
                    }
                    if let Err(e) = write_piece(&session, piece_offset, &bytes).await {
                        last_err = format!("write failed: {}", e);
                        continue;
                    }
                    {
                        let mut pieces = session.pieces.lock();
                        if let Some(p) = pieces.get_mut(piece_idx) {
                            *p = true;
                        }
                    }
                    session
                        .downloaded
                        .fetch_add(bytes.len() as u64, Ordering::Relaxed);
                    return;
                }
                Err(e) => {
                    last_err = e;
                    continue;
                }
            }
        }
        warn!(
            "[torrent:{}] piece {} attempt {} failed: {}",
            session.id, piece_idx, attempt, last_err
        );
        // If we ran through every seed, no point continuing to the next
        // attempt — same seeds will fail again. Bail.
        let _ = attempt;
        break 'outer;
    }
    warn!(
        "[torrent:{}] giving up on piece {}: {}",
        session.id, piece_idx, last_err
    );
}

async fn fetch_piece(seed: &str, offset: u64, length: u64) -> Result<Vec<u8>, String> {
    let end = offset + length - 1;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(3))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("client: {}", e))?;
    let resp = client
        .get(seed)
        .header("Range", format!("bytes={}-{}", offset, end))
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("send: {}", e))?;
    let status = resp.status();
    if !(status.is_success() || status.as_u16() == 206) {
        return Err(format!("HTTP {}", status));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("read body: {}", e))?;
    Ok(bytes.to_vec())
}

async fn write_piece(
    session: &Arc<TorrentSession>,
    offset: u64,
    bytes: &[u8],
) -> Result<(), String> {
    let _guard = session.write_lock.lock().await;
    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&session.data_path)
        .await
        .map_err(|e| format!("open: {}", e))?;
    use tokio::io::AsyncSeekExt;
    file.seek(std::io::SeekFrom::Start(offset))
        .await
        .map_err(|e| format!("seek: {}", e))?;
    file.write_all(bytes).await.map_err(|e| format!("write: {}", e))?;
    file.flush().await.map_err(|e| format!("flush: {}", e))?;
    Ok(())
}

async fn fetch_url(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("client: {}", e))?;
    let resp = client
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("send: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("read: {}", e))
}

#[derive(Debug, Deserialize)]
pub struct AddRequest {
    pub magnet: Option<String>,
    pub torrent_b64: Option<String>,
}


