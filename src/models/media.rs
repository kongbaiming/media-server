use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// 媒体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
}

/// 媒体文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub media_type: MediaType,
    pub size: u64,
    pub duration: Option<f64>,
    pub format: String,
    pub created_at: String,
    pub modified_at: String,
    pub thumbnail: Option<String>,
    pub metadata: MediaMetadata,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub last_played: Option<String>,
    pub play_progress: Option<f64>,
}

/// 媒体元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub bitrate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub fps: Option<f64>,
}

impl Default for MediaMetadata {
    fn default() -> Self {
        Self {
            bitrate: None,
            width: None,
            height: None,
            video_codec: None,
            audio_codec: None,
            sample_rate: None,
            channels: None,
            fps: None,
        }
    }
}

/// 播放历史来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum HistorySource {
    #[default]
    Local,
    Douyin,
}

/// 播放历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayHistory {
    pub media_id: String,
    pub timestamp: String,
    pub progress: f64,
    pub duration: f64,
    #[serde(default)]
    pub source: HistorySource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_url: Option<String>,
}

impl PlayHistory {
    pub fn is_douyin(&self) -> bool {
        self.source == HistorySource::Douyin || self.media_id.starts_with("douyin:")
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub library_paths: Vec<PathBuf>,
    pub auto_scan: bool,
    pub scan_interval: u64,
    pub transcode_quality: TranscodeQuality,
    pub hardware_acceleration: bool,
    pub default_subtitle_language: String,
    pub server_port: u16,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

/// 转码质量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscodeQuality {
    High,   // 1080p
    Medium, // 720p
    Low,    // 480p
    Auto,   // 根据源自动选择
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            library_paths: Vec::new(),
            auto_scan: true,
            scan_interval: 300,
            transcode_quality: TranscodeQuality::Auto,
            hardware_acceleration: false,
            default_subtitle_language: "chi".to_string(),
            server_port: 8080,
            thumbnail_width: 320,
            thumbnail_height: 180,
        }
    }
}

/// 扫描进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: String,
    pub status: ScanStatus,
}

/// 扫描状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanStatus {
    Idle,
    Scanning,
    Completed,
    Error(String),
}

/// 转码任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeTask {
    pub id: String,
    pub media_id: String,
    pub status: TranscodeStatus,
    pub progress: f64,
    pub output_path: PathBuf,
    pub quality: TranscodeQuality,
}

/// 转码状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscodeStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

impl MediaFile {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let media_type = match extension.as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "ts" | "m2ts" => {
                MediaType::Video
            }
            "mp3" | "flac" | "aac" | "ogg" | "wav" | "wma" | "m4a" => MediaType::Audio,
            _ => MediaType::Video,
        };

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            path,
            media_type,
            size: 0,
            duration: None,
            format: extension,
            created_at: String::new(),
            modified_at: String::new(),
            thumbnail: None,
            metadata: MediaMetadata::default(),
            tags: Vec::new(),
            favorite: false,
            last_played: None,
            play_progress: None,
        }
    }

    pub fn is_video(&self) -> bool {
        self.media_type == MediaType::Video
    }

    pub fn is_audio(&self) -> bool {
        self.media_type == MediaType::Audio
    }

    pub fn resolution_string(&self) -> Option<String> {
        match (self.metadata.width, self.metadata.height) {
            (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
            _ => None,
        }
    }

    pub fn duration_string(&self) -> String {
        match self.duration {
            Some(dur) => {
                let total_seconds = dur as u64;
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                if hours > 0 {
                    format!("{}:{:02}:{:02}", hours, minutes, seconds)
                } else {
                    format!("{}:{:02}", minutes, seconds)
                }
            }
            None => "Unknown".to_string(),
        }
    }

    pub fn file_size_string(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.2} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.2} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}
