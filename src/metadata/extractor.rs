// Additional extraction utilities can be added here

use crate::models::MediaFile;

/// 从文件名解析媒体信息
pub fn parse_media_info_from_filename(media_file: &mut MediaFile) {
    let name = media_file.name.clone();

    // 尝试解析常见的命名格式
    // 例如: Movie.Name.2024.1080p.BluRay.x264-GROUP
    let parts: Vec<&str> = name.split(&['.', ' ', '-', '_'][..]).collect();

    for (i, part) in parts.iter().enumerate() {
        let part_lower = part.to_lowercase();

        // 检测分辨率
        match part_lower.as_str() {
            "2160p" | "4k" | "uhd" => {
                media_file.metadata.width = Some(3840);
                media_file.metadata.height = Some(2160);
            }
            "1080p" | "fhd" => {
                media_file.metadata.width = Some(1920);
                media_file.metadata.height = Some(1080);
            }
            "720p" | "hd" => {
                media_file.metadata.width = Some(1280);
                media_file.metadata.height = Some(720);
            }
            "480p" | "sd" => {
                media_file.metadata.width = Some(854);
                media_file.metadata.height = Some(480);
            }
            _ => {}
        }

        // 检测编码
        match part_lower.as_str() {
            "x264" | "h264" | "avc" => {
                media_file.metadata.video_codec = Some("h264".to_string());
            }
            "x265" | "h265" | "hevc" => {
                media_file.metadata.video_codec = Some("h265".to_string());
            }
            "xvid" => {
                media_file.metadata.video_codec = Some("xvid".to_string());
            }
            _ => {}
        }

        // 检测来源
        if i > 0 {
            match part_lower.as_str() {
                "bluray" | "bdrip" | "brrip" => {
                    media_file.tags.push("BluRay".to_string());
                }
                "webdl" | "webrip" | "web" => {
                    media_file.tags.push("WEB".to_string());
                }
                "dvdrip" | "dvd" => {
                    media_file.tags.push("DVD".to_string());
                }
                "hdtv" => {
                    media_file.tags.push("HDTV".to_string());
                }
                "cam" | "hdcam" => {
                    media_file.tags.push("CAM".to_string());
                }
                _ => {}
            }
        }
    }
}

/// 清理文件名（移除扩展名和特殊字符）
pub fn clean_filename(name: &str) -> String {
    // 移除文件扩展名
    let name = if let Some(dot_pos) = name.rfind('.') {
        &name[..dot_pos]
    } else {
        name
    };

    // 替换特殊字符为空格
    let cleaned = name
        .replace('.', " ")
        .replace('_', " ")
        .replace('-', " ")
        .replace('[', " ")
        .replace(']', " ")
        .replace('(', " ")
        .replace(')', " ")
        .replace('{', " ")
        .replace('}', " ");

    // 移除多余空格
    let mut result = String::new();
    let mut prev_space = false;

    for ch in cleaned.chars() {
        if ch == ' ' {
            if !prev_space {
                result.push(ch);
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }

    result.trim().to_string()
}

/// 检测媒体语言
pub fn detect_language(filename: &str) -> Option<String> {
    let filename_lower = filename.to_lowercase();

    // 常见的语言标记
    let language_patterns = [
        ("chinese", "zh"),
        ("mandarin", "zh"),
        ("cantonese", "zh"),
        ("english", "en"),
        ("japanese", "ja"),
        ("korean", "ko"),
        ("french", "fr"),
        ("german", "de"),
        ("spanish", "es"),
        ("russian", "ru"),
        ("hindi", "hi"),
    ];

    for (pattern, lang) in &language_patterns {
        if filename_lower.contains(pattern) {
            return Some(lang.to_string());
        }
    }

    // 检测语言代码
    let lang_codes = ["chi", "eng", "jpn", "kor", "fre", "ger", "spa", "rus", "hin"];

    for code in &lang_codes {
        if filename_lower.contains(code) {
            return Some(code.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_filename() {
        assert_eq!(clean_filename("Movie.Name.2024.1080p.mp4"), "Movie Name 2024 1080p");
        assert_eq!(clean_filename("my_movie_hd.mkv"), "my movie hd");
        assert_eq!(clean_filename("test"), "test");
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("Movie.English.2024.mp4"), Some("en".to_string()));
        assert_eq!(detect_language("Movie.chi.mp4"), Some("zh".to_string()));
        assert_eq!(detect_language("Movie.mp4"), None);
    }
}
