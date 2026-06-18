//! Magnet URI parser (BEP 9).
//!
//! Extracts the info hash, tracker URLs, web seeds, display name, and any
//! "exact source" (.torrent file URL) from a `magnet:?xt=urn:btih:...&tr=...`
//! URI. Tolerates a wide range of real-world inputs (extra query keys,
//! case-insensitive, etc.).

use percent_encoding::percent_decode_str;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Magnet {
    /// 40-character hex (or 32-char base32) SHA1 of the bencoded info dict.
    pub info_hash_hex: Option<String>,
    /// Display name from `dn=`, if present.
    pub display_name: Option<String>,
    /// Tracker announce URLs (`tr=`).
    pub trackers: Vec<String>,
    /// Web seed URLs from `ws=` (BEP 19) or `as=` (BEP 17, httpseeds).
    pub web_seeds: Vec<String>,
    /// "Exact source" URL pointing at a `.torrent` file (`xs=`).
    pub xs: Vec<String>,
}

impl Magnet {
    /// Returns the 40-char hex info hash, normalising base32 to hex.
    pub fn normalized_info_hash(&self) -> Option<String> {
        if let Some(hex) = &self.info_hash_hex {
            if hex.len() == 40 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(hex.to_lowercase());
            }
            if hex.len() == 32 {
                // base32 (per RFC 4648, no padding) → hex
                if let Ok(decoded) = base32_decode(hex) {
                    if decoded.len() == 20 {
                        return Some(decoded.iter().map(|b| format!("{:02x}", b)).collect());
                    }
                }
            }
        }
        None
    }
}

pub fn parse_magnet(uri: &str) -> Result<Magnet, String> {
    let rest = uri
        .strip_prefix("magnet:?")
        .ok_or_else(|| "Not a magnet URI (missing 'magnet:?')".to_string())?;
    let params: HashMap<String, Vec<String>> = rest
        .split('&')
        .filter_map(|p| {
            let mut it = p.splitn(2, '=');
            let k = it.next()?.to_lowercase();
            let v = it.next().unwrap_or("").to_string();
            if k.is_empty() {
                None
            } else {
                Some((k, v))
            }
        })
        .fold(HashMap::new(), |mut acc, (k, v)| {
            acc.entry(k).or_default().push(v);
            acc });

    let mut m = Magnet::default();
    for raw in params.get("xt").cloned().unwrap_or_default() {
        if let Some(hash) = raw
            .strip_prefix("urn:btih:")
            .or_else(|| raw.strip_prefix("URN:BTIH:"))
        {
            m.info_hash_hex = Some(decode(hash).to_string());
        }
    }
    for raw in params.get("dn").cloned().unwrap_or_default() {
        if let Ok(s) = percent_decode_str(&raw).decode_utf8() {
            m.display_name = Some(s.into_owned());
            break;
        }
    }
    for raw in params.get("tr").cloned().unwrap_or_default() {
        if let Ok(s) = percent_decode_str(&raw).decode_utf8() {
            m.trackers.push(s.into_owned());
        }
    }
    for raw in params.get("ws").cloned().unwrap_or_default() {
        if let Ok(s) = percent_decode_str(&raw).decode_utf8() {
            m.web_seeds.push(s.into_owned());
        }
    }
    for raw in params.get("as").cloned().unwrap_or_default() {
        if let Ok(s) = percent_decode_str(&raw).decode_utf8() {
            m.web_seeds.push(s.into_owned());
        }
    }
    for raw in params.get("xs").cloned().unwrap_or_default() {
        if let Ok(s) = percent_decode_str(&raw).decode_utf8() {
            m.xs.push(s.into_owned());
        }
    }

    if m.info_hash_hex.is_none() {
        return Err("Magnet has no xt=urn:btih:<hash>".to_string());
    }
    Ok(m)
}

fn decode(s: &str) -> String {
    percent_decode_str(s)
        .decode_utf8_lossy()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

// Minimal RFC 4648 base32 decoder (no padding) used to normalise magnet
// info hashes that arrive in base32 instead of hex.
fn base32_decode(input: &str) -> Result<Vec<u8>, ()> {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let input = input.to_ascii_uppercase();
    let mut out = Vec::with_capacity(input.len() * 5 / 8);
    let mut buffer: u64 = 0;
    let mut bits: u32 = 0;
    for ch in input.bytes() {
        let v = match ALPHABET.iter().position(|c| *c == ch) {
            Some(v) => v as u64,
            None => return Err(()),
        };
        buffer = (buffer << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_magnet() {
        let uri = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp%3A%2F%2Fexplodie.org%3A6969&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&ws=https%3A%2F%2Fwebseed.example.com%2Ffile.mp4";
        let m = parse_magnet(uri).unwrap();
        assert_eq!(m.normalized_info_hash().unwrap(), "dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c");
        assert_eq!(m.display_name.as_deref(), Some("Big Buck Bunny"));
        assert_eq!(m.trackers.len(), 2);
        assert_eq!(m.web_seeds.len(), 1);
    }

    #[test]
    fn rejects_non_magnet() {
        assert!(parse_magnet("https://example.com/file.torrent").is_err());
    }
}
