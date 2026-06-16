mod ffmpeg;

pub use ffmpeg::*;

use crate::models::{TranscodeQuality, TranscodeStatus, TranscodeTask};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

/// HLS转码器
pub struct Transcoder {
    tasks: Arc<Mutex<HashMap<String, TranscodeTask>>>,
    output_dir: PathBuf,
}

impl Transcoder {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            output_dir,
        }
    }

    /// 创建转码任务
    pub async fn create_task(
        &self,
        media_id: &str,
        _input_path: &Path,
        quality: TranscodeQuality,
    ) -> anyhow::Result<String> {
        let task_id = Uuid::new_v4().to_string();
        let output_path = self.output_dir.join(&task_id);

        // 创建输出目录
        tokio::fs::create_dir_all(&output_path).await?;

        let task = TranscodeTask {
            id: task_id.clone(),
            media_id: media_id.to_string(),
            status: TranscodeStatus::Pending,
            progress: 0.0,
            output_path: output_path.clone(),
            quality,
        };

        let mut tasks = self.tasks.lock().await;
        tasks.insert(task_id.clone(), task);

        info!("Created transcode task: {}", task_id);

        Ok(task_id)
    }

    /// 开始转码
    pub async fn start_transcode(
        &self,
        task_id: &str,
        input_path: &Path,
    ) -> anyhow::Result<()> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.status = TranscodeStatus::InProgress;

        let task_id = task_id.to_string();
        let output_path = task.output_path.clone();
        let quality = task.quality.clone();

        drop(tasks);

        // 在后台执行转码
        let tasks = self.tasks.clone();
        let input_path = input_path.to_path_buf();

        tokio::spawn(async move {
            match Self::run_transcode(&input_path, &output_path, quality).await {
                Ok(_) => {
                    let mut tasks = tasks.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.status = TranscodeStatus::Completed;
                        task.progress = 100.0;
                    }
                    info!("Transcode completed: {}", task_id);
                }
                Err(e) => {
                    let mut tasks = tasks.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.status = TranscodeStatus::Failed(e.to_string());
                    }
                    warn!("Transcode failed: {} - {}", task_id, e);
                }
            }
        });

        Ok(())
    }

    /// 运行FFmpeg转码
    async fn run_transcode(
        input_path: &Path,
        output_path: &Path,
        quality: TranscodeQuality,
    ) -> anyhow::Result<()> {
        let (width, height, bitrate) = match quality {
            TranscodeQuality::High => (1920, 1080, 5000),
            TranscodeQuality::Medium => (1280, 720, 2500),
            TranscodeQuality::Low => (854, 480, 1000),
            TranscodeQuality::Auto => {
                // TODO: 根据源文件自动选择质量
                (1280, 720, 2500)
            }
        };

        // 创建HLS切片
        let master_playlist = output_path.join("master.m3u8");
        let segment_pattern = output_path.join("segment%03d.ts");

        let args = vec![
            "-i".to_string(),
            input_path.to_string_lossy().to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "medium".to_string(),
            "-b:v".to_string(),
            format!("{}k", bitrate),
            "-maxrate".to_string(),
            format!("{}k", bitrate * 2),
            "-bufsize".to_string(),
            format!("{}k", bitrate * 2),
            "-vf".to_string(),
            format!("scale={}:{}", width, height),
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "128k".to_string(),
            "-hls_time".to_string(),
            "10".to_string(),
            "-hls_list_size".to_string(),
            "0".to_string(),
            "-hls_segment_filename".to_string(),
            segment_pattern.to_string_lossy().to_string(),
            master_playlist.to_string_lossy().to_string(),
        ];

        run_ffmpeg(&args).await?;

        Ok(())
    }

    /// 获取转码进度
    pub async fn get_progress(&self, task_id: &str) -> Option<f64> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id).map(|t| t.progress)
    }

    /// 获取转码状态
    pub async fn get_status(&self, task_id: &str) -> Option<TranscodeStatus> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id).map(|t| t.status.clone())
    }

    /// 获取HLS播放列表路径
    pub async fn get_playlist_path(&self, task_id: &str) -> Option<PathBuf> {
        let tasks = self.tasks.lock().await;
        tasks
            .get(task_id)
            .filter(|t| matches!(t.status, TranscodeStatus::Completed))
            .map(|t| t.output_path.join("master.m3u8"))
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<TranscodeTask> {
        let tasks = self.tasks.lock().await;
        tasks.values().cloned().collect()
    }

    /// 删除转码任务
    pub async fn delete_task(&self, task_id: &str) -> anyhow::Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.remove(task_id) {
            // 删除输出目录
            if task.output_path.exists() {
                tokio::fs::remove_dir_all(&task.output_path).await?;
            }
            info!("Deleted transcode task: {}", task_id);
        }
        Ok(())
    }

    /// 清理所有已完成的任务
    pub async fn cleanup_completed(&self) -> anyhow::Result<()> {
        let mut tasks = self.tasks.lock().await;
        let completed_ids: Vec<String> = tasks
            .iter()
            .filter(|(_, t)| matches!(t.status, TranscodeStatus::Completed))
            .map(|(id, _)| id.clone())
            .collect();

        for id in completed_ids {
            if let Some(task) = tasks.remove(&id) {
                if task.output_path.exists() {
                    tokio::fs::remove_dir_all(&task.output_path).await?;
                }
            }
        }

        Ok(())
    }
}

/// FFmpeg包装器
async fn run_ffmpeg(args: &[String]) -> anyhow::Result<()> {
    use tokio::process::Command;

    let output = Command::new("ffmpeg")
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("FFmpeg failed: {}", stderr));
    }

    Ok(())
}

/// 创建简单的HLS切片（不转码，直接切片）
pub async fn create_hls_segments(
    input_path: &Path,
    output_dir: &Path,
    segment_duration: u32,
) -> anyhow::Result<PathBuf> {
    tokio::fs::create_dir_all(output_dir).await?;

    let master_playlist = output_dir.join("master.m3u8");
    let segment_pattern = output_dir.join("segment%03d.ts");

    let args = vec![
        "-i".to_string(),
        input_path.to_string_lossy().to_string(),
        "-c".to_string(),
        "copy".to_string(),
        "-hls_time".to_string(),
        segment_duration.to_string(),
        "-hls_list_size".to_string(),
        "0".to_string(),
        "-hls_segment_filename".to_string(),
        segment_pattern.to_string_lossy().to_string(),
        master_playlist.to_string_lossy().to_string(),
    ];

    run_ffmpeg(&args).await?;

    Ok(master_playlist)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_transcoder_creation() {
        let dir = tempdir().unwrap();
        let transcoder = Transcoder::new(dir.path().to_path_buf());

        let tasks = transcoder.get_all_tasks().await;
        assert_eq!(tasks.len(), 0);
    }
}
