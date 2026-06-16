# MediaVault

[English](README.md) | [简体中文](README.zh-CN.md)

A local media server built with **Rust + React**, featuring library management, HLS streaming, transcoding, and **Douyin (TikTok China) link playback**.

## Features

- **Media Library** — Scan folders and organize video/audio files
- **Video & Audio Playback** — HLS player with resume progress
- **Douyin Player** — Parse share links, play in browser via server-side proxy (bypasses CDN Referer restrictions)
- **Play History** — Local media and Douyin videos appear in Recent; click to resume
- **Transcoding** — FFmpeg-powered HLS output (optional)
- **Search & Favorites** — Find and bookmark media quickly
- **Statistics & Settings** — Library stats, paths, port, and more

## Supported Formats

| Type  | Formats |
|-------|---------|
| Video | MP4, MKV, AVI, MOV, WMV, FLV, WebM, TS, M2TS, MPG, MPEG, 3GP, OGV, VOB |
| Audio | MP3, FLAC, AAC, OGG, WAV, WMA, M4A, OPUS, APE, ALAC |

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Node.js](https://nodejs.org/) 18+
- [FFmpeg](https://ffmpeg.org/download.html) (optional, for transcoding; must be in `PATH`)

## Quick Start

### Clone

```bash
git clone https://github.com/kongbaiming/media-server.git
cd media-server
```

### Install dependencies

```bash
cargo build
npm install
```

### Run (development)

**Option A — Web UI (recommended)**

```bash
# Terminal 1: backend API (port 8080)
cargo run

# Terminal 2: frontend (port 1420)
npm run dev
```

Open http://localhost:1420

**Option B — Windows one-click**

```bat
start-dev.bat
```

**Option C — Backend only**

```bash
cargo run
# API available at http://localhost:8080
```

**Option D — Tauri desktop app**

```bash
npm run tauri dev
```

### Production build

```bash
npm run build          # frontend static assets
cargo build --release  # backend binary
npm run tauri build    # desktop app (optional)
```

## Douyin Usage

1. Open the **Douyin** page in the sidebar
2. Paste a share link or share text (e.g. `https://v.douyin.com/xxxxx`)
3. Click **Parse**, then **Play Video**
4. The entry is saved to **Recent** automatically

Supported link formats:

- Short links: `https://v.douyin.com/xxxxx`
- Video URLs: `https://www.douyin.com/video/xxxxx`
- Share text containing an embedded link

## Project Structure

```
media-server/
├── src/                 # Rust backend + React frontend
│   ├── main.rs          # Server entry
│   ├── lib.rs
│   ├── models/
│   ├── scanner/
│   ├── metadata/
│   ├── transcoder/
│   ├── server/          # Axum routes & handlers
│   ├── storage/         # JSON persistence (~/.mediavault)
│   ├── douyin/          # Douyin link parser & proxy
│   ├── components/      # React UI
│   ├── services/        # API client
│   └── stores/
├── src-tauri/           # Tauri desktop wrapper
├── package.json
└── Cargo.toml
```

## API Overview

| Group | Endpoint | Description |
|-------|----------|-------------|
| Library | `GET /api/library` | List media |
| Library | `POST /api/library/scan` | Scan folders |
| Stream | `GET /api/stream/{id}/master.m3u8` | HLS playlist |
| History | `GET /api/history` | Recent playback (deduplicated) |
| History | `POST /api/history/douyin` | Record Douyin play |
| History | `POST /api/history/{id}/progress` | Update progress |
| Douyin | `POST /api/douyin/parse` | Parse share URL |
| Douyin | `GET /api/douyin/proxy?url=...` | Proxy video stream |
| Config | `GET/PUT /api/config` | App settings |
| Stats | `GET /api/stats` | Library statistics |

Default server port: **8080**

## Configuration

Config is stored at `~/.mediavault/config.json`:

```json
{
  "library_paths": ["C:/Users/Videos"],
  "auto_scan": true,
  "scan_interval": 300,
  "transcode_quality": "Auto",
  "hardware_acceleration": false,
  "server_port": 8080
}
```

## Development

```bash
cargo test
cargo fmt
cargo clippy
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Push and open a Pull Request

## Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) — HTTP framework
- [Tauri](https://tauri.app/) — Desktop shell
- [FFmpeg](https://ffmpeg.org/) — Transcoding
- [Plyr](https://plyr.io/) & [HLS.js](https://github.com/video-dev/hls.js) — Playback
