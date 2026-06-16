mod json_store;

pub use json_store::*;

use crate::models::{AppConfig, MediaFile, PlayHistory, HistorySource};
use crate::douyin::DouyinVideo;
use std::collections::HashMap;

/// 存储管理器
pub struct StorageManager {
    pub store: JsonStore,
}

impl StorageManager {
    pub fn new() -> anyhow::Result<Self> {
        let store = JsonStore::new()?;
        Ok(Self { store })
    }

    // 媒体库操作
    pub fn save_media_library(&self, files: &[MediaFile]) -> anyhow::Result<()> {
        self.store.save_media_library(files)
    }

    pub fn load_media_library(&self) -> anyhow::Result<Vec<MediaFile>> {
        self.store.load_media_library()
    }

    pub fn add_media_file(&self, file: &MediaFile) -> anyhow::Result<()> {
        let mut library = self.load_media_library()?;
        library.push(file.clone());
        self.save_media_library(&library)
    }

    pub fn update_media_file(&self, file: &MediaFile) -> anyhow::Result<()> {
        let mut library = self.load_media_library()?;
        if let Some(pos) = library.iter().position(|f| f.id == file.id) {
            library[pos] = file.clone();
            self.save_media_library(&library)
        } else {
            Err(anyhow::anyhow!("Media file not found"))
        }
    }

    pub fn delete_media_file(&self, id: &str) -> anyhow::Result<()> {
        let mut library = self.load_media_library()?;
        library.retain(|f| f.id != id);
        self.save_media_library(&library)
    }

    pub fn get_media_file(&self, id: &str) -> anyhow::Result<Option<MediaFile>> {
        let library = self.load_media_library()?;
        Ok(library.into_iter().find(|f| f.id == id))
    }

    pub fn search_media(&self, query: &str) -> anyhow::Result<Vec<MediaFile>> {
        let library = self.load_media_library()?;
        let query_lower = query.to_lowercase();

        let results: Vec<MediaFile> = library
            .into_iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&query_lower)
                    || f.path.to_string_lossy().to_lowercase().contains(&query_lower)
                    || f.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    // 播放历史操作
    pub fn save_play_history(&self, history: &[PlayHistory]) -> anyhow::Result<()> {
        self.store.save_play_history(history)
    }

    pub fn load_play_history(&self) -> anyhow::Result<Vec<PlayHistory>> {
        self.store.load_play_history()
    }

    pub fn add_play_history(&self, history: &PlayHistory) -> anyhow::Result<()> {
        let mut histories = self.load_play_history()?;
        histories.retain(|h| h.media_id != history.media_id);
        histories.push(history.clone());

        // 只保留最近1000条记录
        if histories.len() > 1000 {
            histories = histories[histories.len() - 1000..].to_vec();
        }

        self.save_play_history(&histories)
    }

    pub fn get_recent_history(&self) -> anyhow::Result<Vec<PlayHistory>> {
        let histories = self.load_play_history()?;
        let mut latest: HashMap<String, PlayHistory> = HashMap::new();

        for entry in histories {
            latest.insert(entry.media_id.clone(), entry);
        }

        let mut recent: Vec<PlayHistory> = latest.into_values().collect();
        recent.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(recent)
    }

    pub fn add_douyin_history(&self, video: &DouyinVideo, progress: f64) -> anyhow::Result<()> {
        let history = PlayHistory {
            media_id: format!("douyin:{}", video.id),
            timestamp: chrono::Utc::now().to_rfc3339(),
            progress,
            duration: video.duration,
            source: HistorySource::Douyin,
            title: Some(video.title.clone()),
            author: Some(video.author.clone()),
            cover: video.cover.clone(),
            share_url: Some(video.share_url.clone()),
        };
        self.add_play_history(&history)
    }

    pub fn update_play_progress(
        &self,
        media_id: &str,
        progress: f64,
        duration: f64,
    ) -> anyhow::Result<()> {
        if media_id.starts_with("douyin:") {
            let mut histories = self.load_play_history()?;
            if let Some(entry) = histories.iter_mut().find(|h| h.media_id == media_id) {
                entry.progress = progress;
                entry.duration = duration;
                entry.timestamp = chrono::Utc::now().to_rfc3339();
                self.save_play_history(&histories)?;
            }
            return Ok(());
        }

        // 更新媒体文件的播放进度
        let mut library = self.load_media_library()?;
        if let Some(file) = library.iter_mut().find(|f| f.id == media_id) {
            file.play_progress = Some(progress);
            file.last_played = Some(chrono::Utc::now().to_rfc3339());
            self.save_media_library(&library)?;
        }

        // 添加播放历史
        let history = PlayHistory {
            media_id: media_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            progress,
            duration,
            source: HistorySource::Local,
            title: None,
            author: None,
            cover: None,
            share_url: None,
        };
        self.add_play_history(&history)?;

        Ok(())
    }

    // 收藏操作
    pub fn toggle_favorite(&self, media_id: &str) -> anyhow::Result<bool> {
        let mut library = self.load_media_library()?;
        if let Some(file) = library.iter_mut().find(|f| f.id == media_id) {
            file.favorite = !file.favorite;
            let is_favorite = file.favorite;
            self.save_media_library(&library)?;
            Ok(is_favorite)
        } else {
            Err(anyhow::anyhow!("Media file not found"))
        }
    }

    pub fn get_favorites(&self) -> anyhow::Result<Vec<MediaFile>> {
        let library = self.load_media_library()?;
        Ok(library.into_iter().filter(|f| f.favorite).collect())
    }

    // 配置操作
    pub fn save_config(&self, config: &AppConfig) -> anyhow::Result<()> {
        self.store.save_config(config)
    }

    pub fn load_config(&self) -> anyhow::Result<AppConfig> {
        self.store.load_config()
    }

    pub fn update_config<F>(&self, updater: F) -> anyhow::Result<AppConfig>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.load_config()?;
        updater(&mut config);
        self.save_config(&config)?;
        Ok(config)
    }

    // 统计信息
    pub fn get_statistics(&self) -> anyhow::Result<LibraryStatistics> {
        let library = self.load_media_library()?;
        let history = self.load_play_history()?;

        let total_files = library.len();
        let video_count = library.iter().filter(|f| f.is_video()).count();
        let audio_count = library.iter().filter(|f| f.is_audio()).count();
        let total_size: u64 = library.iter().map(|f| f.size).sum();
        let total_duration: f64 = library.iter().filter_map(|f| f.duration).sum();
        let favorite_count = library.iter().filter(|f| f.favorite).count();

        Ok(LibraryStatistics {
            total_files,
            video_count,
            audio_count,
            total_size,
            total_duration,
            favorite_count,
            play_count: history.len(),
        })
    }
}

/// 库统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct LibraryStatistics {
    pub total_files: usize,
    pub video_count: usize,
    pub audio_count: usize,
    pub total_size: u64,
    pub total_duration: f64,
    pub favorite_count: usize,
    pub play_count: usize,
}

impl LibraryStatistics {
    pub fn total_size_string(&self) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;

        if self.total_size >= GB {
            format!("{:.2} GB", self.total_size as f64 / GB as f64)
        } else {
            format!("{:.2} MB", self.total_size as f64 / MB as f64)
        }
    }

    pub fn total_duration_string(&self) -> String {
        let total_seconds = self.total_duration as u64;
        let hours = total_seconds / 3600;
        let days = hours / 24;

        if days > 0 {
            format!("{} days {} hours", days, hours % 24)
        } else {
            format!("{} hours", hours)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_statistics() {
        let stats = LibraryStatistics {
            total_files: 100,
            video_count: 80,
            audio_count: 20,
            total_size: 1024 * 1024 * 1024 * 10, // 10 GB
            total_duration: 3600.0 * 24,          // 24 hours
            favorite_count: 10,
            play_count: 50,
        };

        assert_eq!(stats.total_size_string(), "10.00 GB");
        assert_eq!(stats.total_duration_string(), "1 days 0 hours");
    }
}
