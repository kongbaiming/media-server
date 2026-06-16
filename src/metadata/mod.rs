mod extractor;

use crate::models::MediaFile;
use std::path::Path;
use tokio::process::Command;
use tracing::{info, warn};

/// 元数据提取器
pub struct MetadataExtractor {
    ffmpeg_available: bool,
}

impl MetadataExtractor {
    pub fn new() -> Self {
        Self {
            ffmpeg_available: false,
        }
    }

    /// 检查FFmpeg是否可用
    pub async fn check_ffmpeg(&mut self) -> bool {
        match Command::new("ffmpeg").arg("-version").output().await {
            Ok(output) => {
                self.ffmpeg_available = output.status.success();
                self.ffmpeg_available
            }
            Err(_) => {
                self.ffmpeg_available = false;
                false
            }
        }
    }

    /// 提取媒体文件的元数据
    pub async fn extract_metadata(&self, media_file: &mut MediaFile) -> anyhow::Result<()> {
        if !self.ffmpeg_available {
            warn!("FFmpeg not available, skipping metadata extraction");
            return Ok(());
        }

        let path = &media_file.path;

        // 使用ffprobe获取媒体信息
        let output = Command::new("ffprobe")
            .args(&[
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                path.to_str().unwrap_or(""),
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("ffprobe failed for {:?}: {}", path, stderr);
            return Err(anyhow::anyhow!("ffprobe failed: {}", stderr));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let info: serde_json::Value = serde_json::from_str(&json_str)?;

        // 解析时长
        if let Some(format) = info.get("format") {
            if let Some(duration) = format.get("duration") {
                if let Some(dur_str) = duration.as_str() {
                    if let Ok(dur) = dur_str.parse::<f64>() {
                        media_file.duration = Some(dur);
                    }
                }
            }

            if let Some(bitrate) = format.get("bit_rate") {
                if let Some(br_str) = bitrate.as_str() {
                    if let Ok(br) = br_str.parse::<u64>() {
                        media_file.metadata.bitrate = Some(br);
                    }
                }
            }
        }

        // 解析流信息
        if let Some(streams) = info.get("streams").and_then(|s| s.as_array()) {
            for stream in streams {
                let codec_type = stream
                    .get("codec_type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                match codec_type {
                    "video" => {
                        if let Some(width) = stream.get("width").and_then(|w| w.as_i64()) {
                            media_file.metadata.width = Some(width as u32);
                        }
                        if let Some(height) = stream.get("height").and_then(|h| h.as_i64()) {
                            media_file.metadata.height = Some(height as u32);
                        }
                        if let Some(codec) = stream.get("codec_name").and_then(|c| c.as_str()) {
                            media_file.metadata.video_codec = Some(codec.to_string());
                        }
                        // 解析帧率
                        if let Some(fps_str) = stream
                            .get("r_frame_rate")
                            .and_then(|f| f.as_str())
                        {
                            let parts: Vec<&str> = fps_str.split('/').collect();
                            if parts.len() == 2 {
                                if let (Ok(num), Ok(den)) =
                                    (parts[0].parse::<f64>(), parts[1].parse::<f64>())
                                {
                                    if den > 0.0 {
                                        media_file.metadata.fps = Some(num / den);
                                    }
                                }
                            }
                        }
                    }
                    "audio" => {
                        if let Some(codec) = stream.get("codec_name").and_then(|c| c.as_str()) {
                            media_file.metadata.audio_codec = Some(codec.to_string());
                        }
                        if let Some(rate) = stream
                            .get("sample_rate")
                            .and_then(|r| r.as_str())
                        {
                            if let Ok(sr) = rate.parse::<u32>() {
                                media_file.metadata.sample_rate = Some(sr);
                            }
                        }
                        if let Some(channels) = stream.get("channels").and_then(|c| c.as_i64()) {
                            media_file.metadata.channels = Some(channels as u32);
                        }
                    }
                    _ => {}
                }
            }
        }

        info!("Extracted metadata for: {}", media_file.name);
        Ok(())
    }

    /// 生成缩略图
    pub async fn generate_thumbnail(
        &self,
        media_file: &MediaFile,
        output_path: &Path,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        if !self.ffmpeg_available {
            return Err(anyhow::anyhow!("FFmpeg not available"));
        }

        let path = &media_file.path;

        // 计算视频10%位置的时间戳
        let timestamp = media_file.duration.map(|d| d * 0.1).unwrap_or(10.0);
        let timestamp_str = format!("{:.1}", timestamp);

        let output = Command::new("ffmpeg")
            .args(&[
                "-i",
                path.to_str().unwrap_or(""),
                "-ss",
                &timestamp_str,
                "-vframes",
                "1",
                "-vf",
                &format!("scale={}:{}", width, height),
                "-y",
                output_path.to_str().unwrap_or(""),
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Thumbnail extraction failed: {}", stderr));
        }

        info!("Generated thumbnail for: {}", media_file.name);
        Ok(())
    }
}
