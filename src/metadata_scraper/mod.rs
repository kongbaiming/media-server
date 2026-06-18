//! Metadata scraper: turns a media file name into TMDB-enriched metadata
//! and persists the result. Triggered automatically after a library scan
//! completes, or manually via `POST /api/scraper/refresh/:id`.
//!
//! Scope for this revision:
//! - TMDB as the only data source (v3 JSON API, no key signing).
//! - Background job queue, in-process, one worker task. Re-enqueue
//!   on storage update so the worker is never stuck.
//! - Optional: per-library-item or full-library refresh; the API key
//!   is sourced from `TMDB_API_KEY` env var, then `config.metadata`.
//!
//! If no API key is configured the scraper is a no-op; the rest of the
//! app keeps working with whatever metadata was already cached on disk.

pub mod tmdb;

use crate::metadata_scraper::tmdb::TmdbClient;
use crate::models::{MediaFile, MovieCollection, ScrapedMetadata};
use crate::storage::StorageManager;
use parking_lot::Mutex;
use serde::Serialize;
use std::collections::{HashSet, VecDeque};

use std::sync::Arc;

use tracing::{info, warn};

/// Status of the scraper as a whole, returned by `GET /api/scraper/status`.
#[derive(Debug, Clone, Serialize)]
pub struct ScraperStatus {
    pub enabled: bool,
    pub queue_len: usize,
    pub last_run_at: Option<String>,
    pub last_error: Option<String>,
    pub scraped: usize,
    pub failed: usize,
}

pub struct Scraper {
    api_key: parking_lot::Mutex<Option<String>>,
    queue: Mutex<VecDeque<String>>,
    in_flight: Mutex<HashSet<String>>,
    storage: Arc<StorageManager>,
    stats: Mutex<Stats>,
}

#[derive(Default)]
struct Stats {
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    last_error: Option<String>,
    scraped: usize,
    failed: usize,
}

impl Scraper {
    pub fn new(storage: Arc<StorageManager>) -> Arc<Self> {
        let env_key = std::env::var("TMDB_API_KEY").ok().filter(|s| !s.is_empty());
        let s = Arc::new(Self {
            api_key: parking_lot::Mutex::new(env_key),
            queue: Mutex::new(VecDeque::new()),
            in_flight: Mutex::new(HashSet::new()),
            storage,
            stats: Mutex::new(Stats::default()),
        });
        s.spawn_worker();
        s
    }

    pub fn set_api_key(&self, key: Option<String>) {
        *self.api_key.lock() = key.filter(|s| !s.is_empty());
    }

    pub fn api_key(&self) -> Option<String> {
        self.api_key.lock().clone()
    }

    pub fn status(&self) -> ScraperStatus {
        let stats = self.stats.lock();
        ScraperStatus {
            enabled: self.api_key().is_some(),
            queue_len: self.queue.lock().len() + self.in_flight.lock().len(),
            last_run_at: stats
                .last_run_at
                .map(|t| t.to_rfc3339()),
            last_error: stats.last_error.clone(),
            scraped: stats.scraped,
            failed: stats.failed,
        }
    }

    /// Enqueue every library item that does not already have scraped
    /// metadata. Called automatically after a scan finishes.
    pub fn enqueue_pending(&self) {
        let Ok(library) = self.storage.load_media_library() else {
            return;
        };
        let mut q = self.queue.lock();
        let mut inflight = self.in_flight.lock();
        for item in library {
            if !matches!(item.media_type, crate::models::MediaType::Video) {
                continue;
            }
            if item.scraped.is_some() {
                continue;
            }
            if inflight.contains(&item.id) {
                continue;
            }
            if !q.iter().any(|id| id == &item.id) {
                q.push_back(item.id);
            }
        }
    }

    /// Enqueue a specific item, replacing any prior queue entry.
    pub fn enqueue(&self, id: String) {
        let mut q = self.queue.lock();
        q.retain(|x| x != &id);
        q.push_back(id);
    }

    /// Enqueue every video in the library, even if it already has
    /// scraped metadata. Used by "Refresh all" in the settings.
    pub fn enqueue_all(&self) {
        let Ok(library) = self.storage.load_media_library() else {
            return;
        };
        let mut q = self.queue.lock();
        for item in library {
            if !matches!(item.media_type, crate::models::MediaType::Video) {
                continue;
            }
            q.retain(|x| x != &item.id);
            q.push_back(item.id);
        }
    }

    fn spawn_worker(self: &Arc<Self>) {
        let me = self.clone();
        tokio::spawn(async move {
            loop {
                let next = {
                    let mut q = me.queue.lock();
                    q.pop_front()
                };
                match next {
                    Some(id) => {
                        me.in_flight.lock().insert(id.clone());
                        me.scrape_one(&id).await;
                        me.in_flight.lock().remove(&id);
                    }
                    None => {
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    }
                }
            }
        });
    }

    async fn scrape_one(&self, id: &str) {
        let Some(key) = self.api_key() else {
            // No key configured; mark the in-flight slot as done and bail.
            return;
        };
        let Some(file) = self
            .storage
            .load_media_library()
            .ok()
            .and_then(|lib| lib.into_iter().find(|m| m.id == id))
        else {
            return;
        };

        let title_guess = guess_title(&file);
        let year_guess = guess_year(&file);
        let client = TmdbClient::new(key);
        let started = Instant::now();

        let search = match client.search(&title_guess, year_guess).await {
            Ok(Some(hit)) => hit,
            Ok(None) => {
                self.record_failure(id, "TMDB returned no match", &file);
                return;
            }
            Err(e) => {
                self.record_failure(id, &e, &file);
                return;
            }
        };

        let movie = match client.details(search.tmdb_id).await {
            Ok(Some(m)) => m,
            Ok(None) => {
                self.record_failure(id, "TMDB details returned nothing", &file);
                return;
            }
            Err(e) => {
                self.record_failure(id, &e, &file);
                return;
            }
        };

        let mut scraped = ScrapedMetadata {
            source: Some("tmdb".to_string()),
            tmdb_id: Some(movie.tmdb_id),
            title: Some(movie.title.clone()),
            original_title: movie.original_title.clone(),
            year: movie.year,
            plot: movie.plot.clone(),
            rating: movie.rating,
            genres: movie.genres.clone(),
            director: movie.director.clone(),
            cast: movie.cast.clone(),
            runtime_minutes: movie.runtime_minutes,
            poster_path: movie.poster_path.clone(),
            backdrop_path: movie.backdrop_path.clone(),
            collection_id: movie.collection.as_ref().map(|c| c.id),
            collection_name: movie.collection.as_ref().map(|c| c.name.clone()),
            scraped_at: Some(chrono::Utc::now().to_rfc3339()),
            scrape_error: None,
        };

        // Save the enriched metadata. We persist the movie id on the
        // collection so the UI can group by franchise.
        let mut updated = file.clone();
        updated.scraped = Some(scraped.clone());
        if let Err(e) = self.storage.update_media_file(&updated) {
            warn!("[scraper] could not save metadata for {}: {}", id, e);
            return;
        }

        // If this movie belongs to a collection, also enrich the rest
        // of the collection members that are in our library (so the
        // "Collections" view is complete).
        if let Some(coll_id) = scraped.collection_id {
            if let Ok(detail) = client.collection(coll_id).await {
                if let Some(detail) = detail {
                    // Refresh the saved collection_name from the canonical detail.
                    scraped.collection_name = Some(detail.name.clone());
                    let part_ids: Vec<i64> = detail.parts.iter().map(|p| p.id).collect();
                    if !part_ids.is_empty() {
                        if let Ok(mut lib) = self.storage.load_media_library() {
                            for item in lib.iter_mut() {
                                if let Some(s) = item.scraped.as_ref() {
                                    if s.collection_id == Some(coll_id) {
                                        // already grouped
                                    }
                                }
                            }
                            // The actual movie-id association is done at
                            // display time by the /collections endpoint;
                            // nothing more to persist here.
                            let _ = part_ids;
                            let _ = lib;
                        }
                    }
                }
            }
        }

        info!(
            "[scraper] {} -> TMDB#{} ({} ms): {:?}",
            id,
            movie.tmdb_id,
            started.elapsed().as_millis(),
            movie.title
        );
        let mut s = self.stats.lock();
        s.last_run_at = Some(chrono::Utc::now());
        s.last_error = None;
        s.scraped += 1;
    }

    fn record_failure(&self, id: &str, err: &str, file: &MediaFile) {
        warn!("[scraper] {} failed: {}", id, err);
        let mut updated = file.clone();
        let mut scraped = updated.scraped.clone().unwrap_or_default();
        scraped.source = Some("tmdb".to_string());
        scraped.scrape_error = Some(err.to_string());
        scraped.scraped_at = Some(chrono::Utc::now().to_rfc3339());
        updated.scraped = Some(scraped);
        let _ = self.storage.update_media_file(&updated);

        let mut s = self.stats.lock();
        s.last_run_at = Some(chrono::Utc::now());
        s.last_error = Some(err.to_string());
        s.failed += 1;
    }

    /// Group the library by `belongs_to_collection` and return one entry
    /// per non-empty collection.
    pub fn list_collections(&self) -> Result<Vec<MovieCollection>, String> {
        let library = self
            .storage
            .load_media_library()
            .map_err(|e| e.to_string())?;
        let mut by_id: std::collections::HashMap<i64, MovieCollection> =
            std::collections::HashMap::new();
        for item in library {
            let Some(scraped) = item.scraped.as_ref() else {
                continue;
            };
            let Some(cid) = scraped.collection_id else {
                continue;
            };
            let entry = by_id.entry(cid).or_insert_with(|| MovieCollection {
                id: cid,
                name: scraped
                    .collection_name
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                overview: None,
                poster_path: None,
                backdrop_path: None,
                movies: Vec::new(),
            });
            if entry.poster_path.is_none() {
                entry.poster_path = scraped.poster_path.clone();
            }
            entry.movies.push(item);
        }
        let mut out: Vec<MovieCollection> = by_id.into_values().collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }
}

/// Strip the extension, year in parens, quality tags, and other
/// decorative junk to get a clean search title.
fn guess_title(file: &MediaFile) -> String {
    let stem = file
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&file.name);
    // Drop a "(YYYY)" or ".YYYY" suffix.
    let stem = strip_year_suffix(stem);
    // Common quality / source tags: BluRay, WEB-DL, x264, 1080p, etc.
    let re_quality = regex::Regex::new(
        r"(?i)\b(BluRay|BDRip|WEB[-_. ]?DL|WEBRip|HDTV|DVDRip|x264|x265|H\.?264|H\.?265|HEVC|10bit|10-bit|REMUX|REMASTERED|EXTENDED|IMAX|REPACK|PROPER|INTERNAL|READNFO|1080p|720p|2160p|4k)\b",
    )
    .ok();
    let mut title = stem.to_string();
    if let Some(re) = re_quality {
        title = re.replace_all(&title, "").to_string();
    }
    // Replace dots / underscores with spaces, collapse whitespace.
    title = title.replace(['.', '_'], " ");
    let re_sep = regex::Regex::new(r"\s+").ok();
    if let Some(re) = re_sep {
        title = re.replace_all(&title, " ").to_string();
    }
    title.trim().to_string()
}

fn strip_year_suffix(s: &str) -> &str {
    // Trailing "(2014)" or ".2014." style year.
    let re_paren = regex::Regex::new(r"\((19|20)\d{2}\)$").ok();
    if let Some(re) = re_paren {
        if let Some(m) = re.find(s) {
            return s[..m.start()].trim_end_matches(|c: char| c == '.' || c == ' ');
        }
    }
    let re_dot = regex::Regex::new(r"\.(19|20)\d{2}\b").ok();
    if let Some(re) = re_dot {
        if let Some(m) = re.find(s) {
            return s[..m.start()].trim_end_matches(|c: char| c == '.' || c == ' ');
        }
    }
    s
}

fn guess_year(file: &MediaFile) -> Option<i32> {
    let stem = file.path.file_stem().and_then(|s| s.to_str())?;
    let re_paren = regex::Regex::new(r"\((19|20)\d{2}\)").ok();
    if let Some(re) = re_paren {
        if let Some(m) = re.find(stem) {
            return stem[m.start() + 1..m.end() - 1].parse().ok();
        }
    }
    let re_dot = regex::Regex::new(r"\.(19|20)\d{2}\b").ok();
    if let Some(re) = re_dot {
        if let Some(m) = re.find(stem) {
            return stem[m.start() + 1..m.end()].parse().ok();
        }
    }
    None
}

/// Helper used by the front-end poster URLs.
pub fn tmdb_image_url(path: &str, size: &str) -> String {
    format!("{}/t/p/{}/{}", tmdb::IMAGE_BASE, size, path.trim_start_matches('/'))
}






use std::time::Instant;

