//! Manages all in-flight torrent sessions.

use crate::torrent::engine::{AddRequest, SessionInfo, TorrentSession};
use crate::torrent::magnet;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct TorrentManager {
    sessions: Mutex<HashMap<String, Arc<TorrentSession>>>,
    storage_dir: PathBuf,
}

impl TorrentManager {
    pub fn new(storage_dir: PathBuf) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            storage_dir,
        }
    }

    pub fn storage_dir(&self) -> &PathBuf {
        &self.storage_dir
    }

    pub async fn add(&self, req: AddRequest) -> Result<SessionInfo, String> {
        let (parsed_magnet, uploaded) = if let Some(b64) = req.torrent_b64.as_deref() {
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(b64.trim())
                .map_err(|e| format!("Invalid base64: {}", e))?;
            // Derive a synthetic magnet for the manager key.
            let m = magnet::Magnet {
                info_hash_hex: Some(crate::torrent::infohash_from_bytes(&bytes)),
                ..Default::default()
            };
            (m, Some(bytes))
        } else if let Some(uri) = req.magnet.as_deref() {
            let m = magnet::parse_magnet(uri)?;
            (m, None)
        } else {
            return Err("Provide either 'magnet' or 'torrent_b64'".to_string());
        };

        let id = uuid::Uuid::new_v4().to_string();
        let session = TorrentSession::start(
            id.clone(),
            &parsed_magnet,
            uploaded,
            self.storage_dir.clone(),
        )
        .await?;

        // Wait briefly for metadata to populate so the returned SessionInfo
        // is useful to the UI.
        for _ in 0..40 {
            let info = session.info(format!("/api/stream/torrent"));
            if info.status != crate::torrent::engine::SessionStatus::Resolving {
                self.sessions.lock().insert(id.clone(), session.clone());
                return Ok(info);
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
        self.sessions.lock().insert(id.clone(), session.clone());
        Ok(session.info("/api/stream/torrent".to_string()))
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        let base = "/api/stream/torrent";
        self.sessions
            .lock()
            .values()
            .map(|s| s.info(base.to_string()))
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<Arc<TorrentSession>> {
        self.sessions.lock().get(id).cloned()
    }

    pub fn remove(&self, id: &str) -> bool {
        let removed = self.sessions.lock().remove(id).is_some();
        if removed {
            let _ = std::fs::remove_dir_all(self.storage_dir.join(id));
        }
        removed
    }
}

