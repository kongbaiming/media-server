mod watcher;

use crate::models::{MediaFile, ScanProgress, ScanStatus};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use walkdir::WalkDir;

/// 支持的媒体文件扩展名
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "ts", "m2ts", "mpg", "mpeg", "3gp",
    "ogv", "vob",
];

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "aac", "ogg", "wav", "wma", "m4a", "opus", "ape", "alac",
];

/// 媒体扫描器
pub struct MediaScanner {
    progress: Arc<Mutex<ScanProgress>>,
}

impl MediaScanner {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(ScanProgress {
                total_files: 0,
                processed_files: 0,
                current_file: String::new(),
                status: ScanStatus::Idle,
            })),
        }
    }

    /// 获取扫描进度
    pub async fn get_progress(&self) -> ScanProgress {
        self.progress.lock().await.clone()
    }

    /// 扫描目录中的媒体文件
    pub async fn scan_directory(&self, path: &Path) -> anyhow::Result<Vec<MediaFile>> {
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {:?}", path));
        }

        info!("Starting scan of directory: {:?}", path);

        // 更新进度状态
        {
            let mut progress = self.progress.lock().await;
            progress.status = ScanStatus::Scanning;
            progress.current_file = path.to_string_lossy().to_string();
        }

        let mut media_files = Vec::new();

        // 首先统计总文件数
        let total_files = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| is_media_file(e.path()))
            .count();

        {
            let mut progress = self.progress.lock().await;
            progress.total_files = total_files;
            progress.processed_files = 0;
        }

        // 扫描文件
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();

            if !is_media_file(file_path) {
                continue;
            }

            // 更新当前扫描文件
            {
                let mut progress = self.progress.lock().await;
                progress.current_file = file_path.to_string_lossy().to_string();
            }

            match create_media_file(file_path).await {
                Ok(media_file) => {
                    info!("Found media file: {}", media_file.name);
                    media_files.push(media_file);
                }
                Err(e) => {
                    warn!("Failed to process file {:?}: {}", file_path, e);
                }
            }

            // 更新进度
            {
                let mut progress = self.progress.lock().await;
                progress.processed_files += 1;
            }
        }

        // 完成扫描
        {
            let mut progress = self.progress.lock().await;
            progress.status = ScanStatus::Completed;
            progress.current_file = String::new();
        }

        info!("Scan completed. Found {} media files", media_files.len());

        Ok(media_files)
    }

    /// 扫描多个目录
    pub async fn scan_multiple_directories(
        &self,
        paths: &[PathBuf],
    ) -> anyhow::Result<Vec<MediaFile>> {
        let mut all_media = Vec::new();

        for path in paths {
            match self.scan_directory(path).await {
                Ok(media) => all_media.extend(media),
                Err(e) => {
                    warn!("Failed to scan directory {:?}: {}", path, e);
                }
            }
        }

        Ok(all_media)
    }

    /// 增量扫描（只扫描新增或修改的文件）
    pub async fn incremental_scan(
        &self,
        path: &Path,
        existing_files: &[MediaFile],
    ) -> anyhow::Result<Vec<MediaFile>> {
        let existing_paths: std::collections::HashSet<PathBuf> =
            existing_files.iter().map(|f| f.path.clone()).collect();

        let mut new_files = Vec::new();

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();

            if !is_media_file(file_path) {
                continue;
            }

            // 检查是否是新文件
            if !existing_paths.contains(file_path) {
                match create_media_file(file_path).await {
                    Ok(media_file) => {
                        new_files.push(media_file);
                    }
                    Err(e) => {
                        warn!("Failed to process new file {:?}: {}", file_path, e);
                    }
                }
            }
        }

        info!("Incremental scan found {} new files", new_files.len());

        Ok(new_files)
    }
}

/// 检查文件是否是媒体文件
fn is_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            VIDEO_EXTENSIONS.contains(&ext_lower.as_str())
                || AUDIO_EXTENSIONS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

/// 创建媒体文件对象
async fn create_media_file(path: &Path) -> anyhow::Result<MediaFile> {
    let metadata = tokio::fs::metadata(path).await?;

    let mut media_file = MediaFile::new(path.to_path_buf());
    media_file.size = metadata.len();

    // 获取文件时间
    if let Ok(created) = metadata.created() {
        media_file.created_at = format!("{:?}", created);
    }
    if let Ok(modified) = metadata.modified() {
        media_file.modified_at = format!("{:?}", modified);
    }

    Ok(media_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_media_file() {
        assert!(is_media_file(&PathBuf::from("test.mp4")));
        assert!(is_media_file(&PathBuf::from("test.MKV")));
        assert!(is_media_file(&PathBuf::from("test.mp3")));
        assert!(is_media_file(&PathBuf::from("test.flac")));
        assert!(!is_media_file(&PathBuf::from("test.txt")));
        assert!(!is_media_file(&PathBuf::from("test.jpg")));
    }

    #[test]
    fn test_get_media_type() {
        use crate::models::MediaType;

        assert_eq!(
            MediaFile::new(PathBuf::from("test.mp4")).media_type,
            MediaType::Video
        );
        assert_eq!(
            MediaFile::new(PathBuf::from("test.mp3")).media_type,
            MediaType::Audio
        );
        assert_eq!(
            MediaFile::new(PathBuf::from("test.flac")).media_type,
            MediaType::Audio
        );
    }
}
