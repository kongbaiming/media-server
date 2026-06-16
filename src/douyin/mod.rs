use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// 抖音视频信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DouyinVideo {
    pub id: String,
    pub title: String,
    pub author: String,
    pub author_avatar: Option<String>,
    pub cover: Option<String>,
    pub duration: f64,
    pub play_url: String,
    pub play_url_no_watermark: String,
    pub share_url: String,
    pub description: Option<String>,
    pub likes: Option<u64>,
    pub comments: Option<u64>,
    pub shares: Option<u64>,
}

/// 抖音解析器
pub struct DouyinParser {
    client: reqwest::Client,
}

fn default_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
    );
    headers
}

impl DouyinParser {
    fn default_headers(&self) -> reqwest::header::HeaderMap {
        default_headers()
    }

    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .default_headers(default_headers())
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// 解析抖音分享链接
    pub async fn parse_share_url(&self, url: &str) -> anyhow::Result<DouyinVideo> {
        info!("Parsing Douyin URL: {}", url);

        // 提取视频ID
        let video_id = self.extract_video_id(url).await?;
        info!("Extracted video ID: {}", video_id);

        // 获取视频信息
        let video_info = self.get_video_info(&video_id).await?;

        Ok(video_info)
    }

    /// 提取视频ID
    async fn extract_video_id(&self, url: &str) -> anyhow::Result<String> {
        // 清理URL，移除可能的干扰字符
        let cleaned_url = self.clean_url(url);

        // 处理短链接 https://v.douyin.com/xxxxx
        if cleaned_url.contains("v.douyin.com") || cleaned_url.contains("vm.douyin.com") {
            let redirect_url = self.resolve_short_url(&cleaned_url).await?;
            info!("Resolved to: {}", redirect_url);
            return self.extract_id_from_url(&redirect_url);
        }

        // 处理完整链接 https://www.douyin.com/video/xxxxx
        self.extract_id_from_url(&cleaned_url)
    }

    /// 清理URL
    fn clean_url(&self, url: &str) -> String {
        // 提取URL部分（从文本中）
        let url_patterns = [
            r"https?://v\.douyin\.com/[a-zA-Z0-9_\-]+/?",
            r"https?://vm\.douyin\.com/[a-zA-Z0-9_\-]+/?",
            r"https?://www\.douyin\.com/video/\d+",
            r"https?://www\.iesdouyin\.com/share/video/\d+",
        ];

        for pattern in &url_patterns {
            if let Some(captures) = regex_capture(url, pattern) {
                let cleaned = captures.trim_end_matches('/').to_string();
                info!("Extracted URL from text: {}", cleaned);
                return cleaned;
            }
        }

        // 如果没有匹配到模式，尝试基本清理
        let mut cleaned = url.trim().to_string();

        // 移除末尾的斜杠和空格
        cleaned = cleaned.trim_end_matches('/').trim().to_string();

        // 确保有协议前缀
        if !cleaned.starts_with("http") {
            cleaned = format!("https://{}", cleaned.trim_start_matches('/'));
        }

        cleaned
    }

    /// 解析短链接获取重定向URL
    async fn resolve_short_url(&self, url: &str) -> anyhow::Result<String> {
        let full_url = if url.starts_with("http") {
            url.to_string()
        } else {
            format!("https://{}", url)
        };

        info!("Resolving short URL: {}", full_url);

        let redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .default_headers(self.default_headers())
            .build()?;

        let response = redirect_client.get(&full_url).send().await?;
        let final_url = response.url().to_string();

        if final_url.contains("douyin.com") || final_url.contains("iesdouyin.com") {
            info!("Resolved to: {}", final_url);
            return Ok(final_url);
        }

        let html = response.text().await?;
        info!("Page content length: {}", html.len());

        if let Some(id) = regex_capture(&html, r#"video[/\\](\d+)"#) {
            return Ok(format!("https://www.douyin.com/video/{}", id));
        }

        if let Some(id) = regex_capture(&html, r#"aweme_id[=:]["']?(\d+)"#) {
            return Ok(format!("https://www.douyin.com/video/{}", id));
        }

        if let Some(url) = regex_capture(&html, r#"(https?://www\.douyin\.com/video/\d+)"#) {
            return Ok(url);
        }

        Err(anyhow::anyhow!("Could not resolve short URL"))
    }

    /// 从URL中提取视频ID
    fn extract_id_from_url(&self, url: &str) -> anyhow::Result<String> {
        // 尝试多种模式匹配
        let patterns = [
            r"video[/\\](\d+)",
            r"note[/\\](\d+)",
            r"modal_id=(\d+)",
            r"aweme_id=(\d+)",
            r"/(\d{15,})",
        ];

        for pattern in &patterns {
            if let Some(id) = regex_capture(url, pattern) {
                return Ok(id);
            }
        }

        Err(anyhow::anyhow!("Could not extract video ID from URL: {}", url))
    }

    /// 获取视频信息
    async fn get_video_info(&self, video_id: &str) -> anyhow::Result<DouyinVideo> {
        // 使用抖音内部API获取视频信息
        let api_url = format!(
            "https://www.douyin.com/aweme/v1/web/aweme/detail/?aweme_id={}&aid=1128&version_name=23.5.0&device_platform=android&os_version=2333",
            video_id
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "Referer",
            HeaderValue::from_static("https://www.douyin.com/"),
        );
        headers.insert(
            "Cookie",
            HeaderValue::from_static("msToken=;odin_tt=;"),
        );

        let response = self.client
            .get(&api_url)
            .headers(headers)
            .send()
            .await?;

        let text = response.text().await?;

        // 尝试解析JSON响应
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
            return self.parse_video_response(&data, video_id);
        }

        // 如果API失败，尝试备用方案
        self.get_video_info_fallback(video_id).await
    }

    /// 解析视频响应
    fn parse_video_response(&self, data: &serde_json::Value, video_id: &str) -> anyhow::Result<DouyinVideo> {
        let aweme_detail = data.get("aweme_detail")
            .ok_or_else(|| anyhow::anyhow!("No aweme_detail in response"))?;

        let title = aweme_detail
            .get("desc")
            .and_then(|d| d.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let author = aweme_detail
            .get("author")
            .and_then(|a| a.get("nickname"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let author_avatar = aweme_detail
            .get("author")
            .and_then(|a| a.get("avatar_thumb"))
            .and_then(|a| a.get("url_list"))
            .and_then(|u| u.as_array())
            .and_then(|a| a.first())
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let cover = aweme_detail
            .get("video")
            .and_then(|v| v.get("cover"))
            .and_then(|c| c.get("url_list"))
            .and_then(|u| u.as_array())
            .and_then(|a| a.first())
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let duration = aweme_detail
            .get("video")
            .and_then(|v| v.get("duration"))
            .and_then(|d| d.as_f64())
            .unwrap_or(0.0) / 1000.0; // 转换为秒

        // 获取播放地址（带水印）
        let play_url = Self::extract_url_from_list(aweme_detail.get("video").and_then(|v| v.get("play_addr")))
            .unwrap_or_default();

        // 优先使用 download_addr 作为无水印地址
        let play_url_no_watermark = Self::extract_url_from_list(
            aweme_detail.get("video").and_then(|v| v.get("download_addr")),
        )
        .or_else(|| {
            aweme_detail
                .get("video")
                .and_then(|v| v.get("bit_rate"))
                .and_then(|b| b.as_array())
                .and_then(|arr| arr.first())
                .and_then(|br| Self::extract_url_from_list(br.get("play_addr")))
        })
        .unwrap_or_else(|| play_url.clone());

        let likes = aweme_detail
            .get("statistics")
            .and_then(|s| s.get("digg_count"))
            .and_then(|c| c.as_u64());

        let comments = aweme_detail
            .get("statistics")
            .and_then(|s| s.get("comment_count"))
            .and_then(|c| c.as_u64());

        let shares = aweme_detail
            .get("statistics")
            .and_then(|s| s.get("share_count"))
            .and_then(|c| c.as_u64());

        Ok(DouyinVideo {
            id: video_id.to_string(),
            title,
            author,
            author_avatar,
            cover,
            duration,
            play_url,
            play_url_no_watermark,
            share_url: format!("https://www.douyin.com/video/{}", video_id),
            description: aweme_detail.get("desc").and_then(|d| d.as_str()).map(|s| s.to_string()),
            likes,
            comments,
            shares,
        })
    }

    /// 备用方案获取视频信息
    async fn get_video_info_fallback(&self, video_id: &str) -> anyhow::Result<DouyinVideo> {
        warn!("Using fallback method for video {}", video_id);

        // 使用网页版获取信息
        let url = format!("https://www.douyin.com/video/{}", video_id);

        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;

        // 从HTML中提取视频信息
        let title = regex_capture(&html, r#""desc"\s*:\s*"([^"]+)""#)
            .unwrap_or_else(|| "Untitled".to_string());

        let author = regex_capture(&html, r#""nickname"\s*:\s*"([^"]+)""#)
            .unwrap_or_else(|| "Unknown".to_string());

        // 提取视频地址
        let play_url = regex_capture(&html, r#""play_addr"\s*:\s*\{[^}]*"url_list"\s*:\s*\["([^"]+)""#)
            .unwrap_or_default();

        Ok(DouyinVideo {
            id: video_id.to_string(),
            title,
            author,
            author_avatar: None,
            cover: None,
            duration: 0.0,
            play_url: play_url.clone(),
            play_url_no_watermark: play_url,
            share_url: url,
            description: None,
            likes: None,
            comments: None,
            shares: None,
        })
    }

    /// 从 url_list 中提取第一个 douyinvod CDN 地址
    fn extract_url_from_list(node: Option<&serde_json::Value>) -> Option<String> {
        let url_list = node?.get("url_list")?.as_array()?;
        url_list
            .iter()
            .filter_map(|u| u.as_str())
            .find(|url| url.starts_with("https://") && !url.contains("amemv.com"))
            .or_else(|| url_list.first()?.as_str())
            .map(|s| s.to_string())
    }

    /// 获取视频播放地址（用于直接播放）
    pub async fn get_play_url(&self, url: &str) -> anyhow::Result<String> {
        let video = self.parse_share_url(url).await?;

        // 优先使用无水印地址
        if !video.play_url_no_watermark.is_empty() {
            return Ok(video.play_url_no_watermark);
        }

        if !video.play_url.is_empty() {
            return Ok(video.play_url);
        }

        Err(anyhow::anyhow!("Could not get play URL"))
    }
}

/// 正则表达式辅助函数
fn regex_capture(text: &str, pattern: &str) -> Option<String> {
    let re = regex::Regex::new(pattern).ok()?;
    re.captures(text).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_string())
}

/// 检查播放地址是否来自允许的 CDN 域名（用于代理校验）
pub fn is_allowed_play_url(url: &str) -> bool {
    url.starts_with("https://")
        && (url.contains("douyinvod.com")
            || url.contains("amemv.com")
            || url.contains("douyinstatic.com")
            || url.contains("snssdk.com")
            || url.contains("bytecdn.cn")
            || url.contains("bytegoofy.com"))
}

/// 检查URL是否是抖音链接
pub fn is_douyin_url(url: &str) -> bool {
    url.contains("douyin.com")
        || url.contains("iesdouyin.com")
        || url.contains("v.douyin.com")
        || url.contains("vm.douyin.com")
}

/// 提取分享文本中的链接
pub fn extract_url_from_text(text: &str) -> Option<String> {
    // 抖音分享文本通常包含链接
    // 例如："7@9.com 03/17 a:/ 复制打开抖音，看看【xxx的作品】 https://v.douyin.com/xxxxx"

    // 使用宽松的正则表达式匹配抖音URL
    let patterns = [
        r"https?://v\.douyin\.com/[a-zA-Z0-9_\-]+",
        r"https?://vm\.douyin\.com/[a-zA-Z0-9_\-]+",
        r"https?://www\.douyin\.com/video/\d+",
        r"https?://www\.iesdouyin\.com/share/video/\d+",
        r"https?://[a-zA-Z0-9._\-]+douyin[a-zA-Z0-9._\-/]+",
    ];

    for pattern in &patterns {
        if let Some(url) = regex_capture(text, pattern) {
            // 移除末尾的斜杠
            let url = url.trim_end_matches('/').to_string();
            tracing::info!("extract_url_from_text: matched URL: {}", url);
            return Some(url);
        }
    }

    // 手动查找 douyin 链接
    if let Some(pos) = text.find("douyin.com") {
        // 向前查找 http:// 或 https://
        let before = &text[..pos];
        if let Some(http_pos) = before.rfind("http") {
            let url_start = http_pos;
            // 向后查找空格、中文字符或其他URL结束字符
            let after = &text[pos..];
            let mut url_end = pos;
            for (i, c) in after.char_indices() {
                if c.is_whitespace() || c == '\n' || c == '\r' || c == '\t' {
                    url_end = pos + i;
                    break;
                }
                // 如果是中文字符，也停止
                if c as u32 > 127 {
                    url_end = pos + i;
                    break;
                }
                url_end = pos + i + c.len_utf8();
            }
            let url = &text[url_start..url_end];
            let url = url.trim_end_matches('/').to_string();
            if url.len() > 20 {  // 确保URL足够长
                tracing::info!("extract_url_from_text: manually extracted URL: {}", url);
                return Some(url);
            }
        }
    }

    tracing::warn!("extract_url_from_text: no URL found in text");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_douyin_url() {
        assert!(is_douyin_url("https://v.douyin.com/abc123"));
        assert!(is_douyin_url("https://www.douyin.com/video/1234567890"));
        assert!(is_douyin_url("https://vm.douyin.com/abc123"));
        assert!(!is_douyin_url("https://www.youtube.com/watch?v=abc"));
    }

    #[test]
    fn test_extract_url_from_text() {
        // 测试普通链接
        let text = "7@9.com 03/17 a:/ 复制打开抖音，看看【xxx的作品】 https://v.douyin.com/abc123";
        let url = extract_url_from_text(text);
        assert!(url.is_some());
        assert!(url.unwrap().contains("v.douyin.com"));

        // 测试带下划线的链接
        let text = "2.87 03/15 n@Q.xf GVY:/ :6pm 地下交通站第一部 https://v.douyin.com/qgux_iZdwVw/ 复制此链接";
        let url = extract_url_from_text(text);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("v.douyin.com"));
        assert!(url.contains("qgux_iZdwVw"));
    }

    #[test]
    fn test_extract_video_id() {
        let parser = DouyinParser::new();

        // 测试从完整URL提取ID
        assert_eq!(
            parser.extract_id_from_url("https://www.douyin.com/video/7123456789012345678").unwrap(),
            "7123456789012345678"
        );
    }
}
