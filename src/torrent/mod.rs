//! Torrent support: parse magnets, fetch .torrent metadata from public
//! caches, download pieces from HTTP web seeds (BEP 17/19), and stream the
//! result to the WebView with HTTP Range support.
//!
//! Scope: single-file or concatenated multi-file torrents that have at
//! least one HTTP web seed. Peer wire / DHT / UDP trackers are out of
//! scope; magnet links without metadata in any public cache are also
//! out of scope (the user can upload a .torrent file as a fallback).

pub mod engine;
pub mod magnet;
pub mod manager;

pub use engine::{FileInfo, SessionInfo, SessionStatus, TorrentSession};
pub use magnet::{parse_magnet, Magnet};
pub use manager::TorrentManager;

use lava_torrent::torrent::v1::Torrent;
use sha1::{Digest, Sha1};

/// Compute the SHA1 of the bencoded info dict of a .torrent file.
/// Used as a fallback info hash when the user uploads a .torrent file
/// without a magnet. Returns a 40-char lowercase hex string.
pub fn infohash_from_bytes(bytes: &[u8]) -> String {
    if let Ok(t) = Torrent::read_from_bytes(bytes) {
        return t.info_hash().to_string();
    }
    // Fall back to hashing the whole thing (won't match BitTorrent's
    // info hash but is at least a stable identifier).
    let mut h = Sha1::new();
    h.update(bytes);
    h.finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
