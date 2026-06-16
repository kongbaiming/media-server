use std::path::Path;
use tokio::process::Command;
use tracing::info;

/// 检查FFmpeg是否可用
pub async fn check_ffmpeg_installed() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 获取FFmpeg版本
pub async fn get_ffmpeg_version() -> anyhow::Result<String> {
    let output = Command::new("ffmpeg").arg("-version").output().await?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        // 提取版本号
        let version_line = version.lines().next().unwrap_or("Unknown");
        Ok(version_line.to_string())
    } else {
        Err(anyhow::anyhow!("FFmpeg not found"))
    }
}

/// 获取媒体文件信息（使用ffprobe）
pub async fn get_media_info(input_path: &Path) -> anyhow::Result<MediaInfo> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            input_path.to_str().unwrap_or(""),
        ])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("ffprobe failed: {}", stderr));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let info: MediaInfo = serde_json::from_str(&json_str)?;

    Ok(info)
}

/// 媒体文件信息（ffprobe输出）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MediaInfo {
    pub format: Option<FormatInfo>,
    pub streams: Option<Vec<StreamInfo>>,
}

/// 格式信息
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FormatInfo {
    pub filename: Option<String>,
    pub format_name: Option<String>,
    pub format_long_name: Option<String>,
    pub duration: Option<String>,
    pub size: Option<String>,
    pub bit_rate: Option<String>,
    pub tags: Option<serde_json::Value>,
}

/// 流信息
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StreamInfo {
    pub index: Option<i32>,
    pub codec_name: Option<String>,
    pub codec_long_name: Option<String>,
    pub codec_type: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub r_frame_rate: Option<String>,
    pub avg_frame_rate: Option<String>,
    pub bit_rate: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<i32>,
    pub channels_layout: Option<String>,
}

impl MediaInfo {
    pub fn duration_seconds(&self) -> Option<f64> {
        self.format
            .as_ref()
            .and_then(|f| f.duration.as_ref())
            .and_then(|d| d.parse::<f64>().ok())
    }

    pub fn file_size(&self) -> Option<u64> {
        self.format
            .as_ref()
            .and_then(|f| f.size.as_ref())
            .and_then(|s| s.parse::<u64>().ok())
    }

    pub fn video_stream(&self) -> Option<&StreamInfo> {
        self.streams
            .as_ref()
            .and_then(|s| s.iter().find(|s| s.codec_type.as_deref() == Some("video")))
    }

    pub fn audio_stream(&self) -> Option<&StreamInfo> {
        self.streams
            .as_ref()
            .and_then(|s| s.iter().find(|s| s.codec_type.as_deref() == Some("audio")))
    }

    pub fn video_codec(&self) -> Option<&str> {
        self.video_stream()
            .and_then(|s| s.codec_name.as_deref())
    }

    pub fn audio_codec(&self) -> Option<&str> {
        self.audio_stream()
            .and_then(|s| s.codec_name.as_deref())
    }

    pub fn resolution(&self) -> Option<(i32, i32)> {
        self.video_stream()
            .and_then(|s| match (s.width, s.height) {
                (Some(w), Some(h)) => Some((w, h)),
                _ => None,
            })
    }

    pub fn fps(&self) -> Option<f64> {
        self.video_stream()
            .and_then(|s| {
                s.r_frame_rate
                    .as_ref()
                    .or(s.avg_frame_rate.as_ref())
                    .and_then(|fps| {
                        let parts: Vec<&str> = fps.split('/').collect();
                        if parts.len() == 2 {
                            let num: f64 = parts[0].parse().ok()?;
                            let den: f64 = parts[1].parse().ok()?;
                            if den > 0.0 {
                                Some(num / den)
                            } else {
                                None
                            }
                        } else {
                            fps.parse().ok()
                        }
                    })
            })
    }

    pub fn audio_sample_rate(&self) -> Option<u32> {
        self.audio_stream()
            .and_then(|s| s.sample_rate.as_ref().and_then(|r| r.parse().ok()))
    }

    pub fn audio_channels(&self) -> Option<i32> {
        self.audio_stream().and_then(|s| s.channels)
    }
}

/// 从媒体文件提取缩略图
pub async fn extract_thumbnail(
    input_path: &Path,
    output_path: &Path,
    timestamp: &str,
    width: i32,
    height: i32,
) -> anyhow::Result<()> {
    let output = Command::new("ffmpeg")
        .args(&[
            "-i",
            input_path.to_str().unwrap_or(""),
            "-ss",
            timestamp,
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

    info!("Extracted thumbnail: {:?}", output_path);

    Ok(())
}

/// 检查硬件加速支持
pub async fn check_hardware_acceleration() -> HardwareAccelInfo {
    let mut info = HardwareAccelInfo {
        nvenc: false,
        qsv: false,
        vaapi: false,
        videotoolbox: false,
    };

    // 检查NVENC
    if let Ok(output) = Command::new("ffmpeg")
        .args(&["-encoders"])
        .output()
        .await
    {
        let encoders = String::from_utf8_lossy(&output.stdout);
        info.nvenc = encoders.contains("h264_nvenc") || encoders.contains("hevc_nvenc");
        info.qsv = encoders.contains("h264_qsv") || encoders.contains("hevc_qsv");
        info.vaapi = encoders.contains("h264_vaapi") || encoders.contains("hevc_vaapi");
        info.videotoolbox = encoders.contains("h264_videotoolbox");
    }

    info
}

/// 硬件加速信息
#[derive(Debug, Clone)]
pub struct HardwareAccelInfo {
    pub nvenc: bool,
    pub qsv: bool,
    pub vaapi: bool,
    pub videotoolbox: bool,
}

impl HardwareAccelInfo {
    pub fn has_support(&self) -> bool {
        self.nvenc || self.qsv || self.vaapi || self.videotoolbox
    }

    pub fn best_encoder(&self, codec: &str) -> &str {
        if self.nvenc {
            match codec {
                "h264" => "h264_nvenc",
                "h265" | "hevc" => "hevc_nvenc",
                _ => "h264_nvenc",
            }
        } else if self.qsv {
            match codec {
                "h264" => "h264_qsv",
                "h265" | "hevc" => "hevc_qsv",
                _ => "h264_qsv",
            }
        } else if self.vaapi {
            match codec {
                "h264" => "h264_vaapi",
                "h265" | "hevc" => "hevc_vaapi",
                _ => "h264_vaapi",
            }
        } else {
            match codec {
                "h264" => "libx264",
                "h265" | "hevc" => "libx265",
                _ => "libx264",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ffmpeg_check() {
        let installed = check_ffmpeg_installed().await;
        println!("FFmpeg installed: {}", installed);
    }

    #[test]
    fn test_media_info_parsing() {
        let json = r#"{
            "format": {
                "filename": "test.mp4",
                "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
                "duration": "120.5",
                "size": "1024000",
                "bit_rate": "500000"
            },
            "streams": [
                {
                    "index": 0,
                    "codec_name": "h264",
                    "codec_type": "video",
                    "width": 1920,
                    "height": 1080,
                    "r_frame_rate": "30/1"
                },
                {
                    "index": 1,
                    "codec_name": "aac",
                    "codec_type": "audio",
                    "sample_rate": "44100",
                    "channels": 2
                }
            ]
        }"#;

        let info: MediaInfo = serde_json::from_str(json).unwrap();

        assert_eq!(info.duration_seconds(), Some(120.5));
        assert_eq!(info.resolution(), Some((1920, 1080)));
        assert_eq!(info.video_codec(), Some("h264"));
        assert_eq!(info.audio_codec(), Some("aac"));
        assert_eq!(info.audio_sample_rate(), Some(44100));
        assert_eq!(info.audio_channels(), Some(2));
    }
}
