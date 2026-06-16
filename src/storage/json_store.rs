use crate::models::{AppConfig, MediaFile, PlayHistory};
use dirs::home_dir;
use std::fs;
use std::path::PathBuf;
use tracing::info;

/// JSON文件存储管理器
pub struct JsonStore {
    base_dir: PathBuf,
}

impl JsonStore {
    pub fn new() -> anyhow::Result<Self> {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let base_dir = home.join(".mediavault");

        // 创建存储目录
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)?;
            info!("Created storage directory: {:?}", base_dir);
        }

        // 创建子目录
        let thumbnails_dir = base_dir.join("thumbnails");
        let transcode_dir = base_dir.join("transcode");

        if !thumbnails_dir.exists() {
            fs::create_dir_all(&thumbnails_dir)?;
        }
        if !transcode_dir.exists() {
            fs::create_dir_all(&transcode_dir)?;
        }

        Ok(Self { base_dir })
    }

    fn library_path(&self) -> PathBuf {
        self.base_dir.join("library.json")
    }

    fn history_path(&self) -> PathBuf {
        self.base_dir.join("history.json")
    }

    fn config_path(&self) -> PathBuf {
        self.base_dir.join("config.json")
    }

    pub fn thumbnails_dir(&self) -> PathBuf {
        self.base_dir.join("thumbnails")
    }

    pub fn transcode_dir(&self) -> PathBuf {
        self.base_dir.join("transcode")
    }

    // 媒体库操作
    pub fn save_media_library(&self, files: &[MediaFile]) -> anyhow::Result<()> {
        let path = self.library_path();
        let json = serde_json::to_string_pretty(files)?;
        fs::write(&path, json)?;
        info!("Saved {} media files to library", files.len());
        Ok(())
    }

    pub fn load_media_library(&self) -> anyhow::Result<Vec<MediaFile>> {
        let path = self.library_path();

        if !path.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(&path)?;
        let files: Vec<MediaFile> = serde_json::from_str(&json)?;
        Ok(files)
    }

    // 播放历史操作
    pub fn save_play_history(&self, history: &[PlayHistory]) -> anyhow::Result<()> {
        let path = self.history_path();
        let json = serde_json::to_string_pretty(history)?;
        fs::write(&path, json)?;
        Ok(())
    }

    pub fn load_play_history(&self) -> anyhow::Result<Vec<PlayHistory>> {
        let path = self.history_path();

        if !path.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(&path)?;
        let history: Vec<PlayHistory> = serde_json::from_str(&json)?;
        Ok(history)
    }

    // 配置操作
    pub fn save_config(&self, config: &AppConfig) -> anyhow::Result<()> {
        let path = self.config_path();
        let json = serde_json::to_string_pretty(config)?;
        fs::write(&path, json)?;
        info!("Saved configuration");
        Ok(())
    }

    pub fn load_config(&self) -> anyhow::Result<AppConfig> {
        let path = self.config_path();

        if !path.exists() {
            info!("No config file found, using default configuration");
            return Ok(AppConfig::default());
        }

        let json = fs::read_to_string(&path)?;
        let config: AppConfig = serde_json::from_str(&json)?;
        Ok(config)
    }

    // 清理操作
    pub fn clear_thumbnails(&self) -> anyhow::Result<()> {
        let dir = self.thumbnails_dir();
        if dir.exists() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                if entry.path().is_file() {
                    fs::remove_file(entry.path())?;
                }
            }
            info!("Cleared thumbnails directory");
        }
        Ok(())
    }

    pub fn clear_transcode_cache(&self) -> anyhow::Result<()> {
        let dir = self.transcode_dir();
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
            fs::create_dir_all(&dir)?;
            info!("Cleared transcode cache");
        }
        Ok(())
    }

    pub fn get_thumbnail_path(&self, media_id: &str) -> PathBuf {
        self.thumbnails_dir().join(format!("{}.ppm", media_id))
    }

    pub fn get_transcode_path(&self, media_id: &str) -> PathBuf {
        self.transcode_dir().join(media_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_store_paths() {
        let store = JsonStore::new().unwrap();

        assert!(store.library_path().to_string_lossy().contains("library.json"));
        assert!(store.history_path().to_string_lossy().contains("history.json"));
        assert!(store.config_path().to_string_lossy().contains("config.json"));
    }
}
