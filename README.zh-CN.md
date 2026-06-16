# MediaVault

[English](README.md) | [简体中文](README.zh-CN.md)

基于 **Rust + React** 的本地媒体服务器，支持媒体库管理、HLS 串流、转码，以及 **抖音分享链接解析与播放**。

## 功能特性

- **媒体库管理** — 扫描文件夹，整理视频与音频
- **在线播放** — 内置 HLS 播放器，支持断点续播
- **抖音播放** — 解析分享链接，通过服务端代理播放（绕过 CDN Referer 限制）
- **播放历史** — 本地媒体与抖音视频统一显示在「Recent」，点击即可继续播放
- **转码** — 基于 FFmpeg 的 HLS 转码（可选）
- **搜索与收藏** — 快速查找、收藏媒体
- **统计与设置** — 库容量统计、路径、端口等配置

## 支持格式

| 类型 | 格式 |
|------|------|
| 视频 | MP4, MKV, AVI, MOV, WMV, FLV, WebM, TS, M2TS, MPG, MPEG, 3GP, OGV, VOB |
| 音频 | MP3, FLAC, AAC, OGG, WAV, WMA, M4A, OPUS, APE, ALAC |

## 环境要求

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Node.js](https://nodejs.org/) 18+
- [FFmpeg](https://ffmpeg.org/download.html)（可选，用于转码，需加入 `PATH`）

## 快速开始

### 克隆项目

```bash
git clone https://github.com/kongbaiming/media-server.git
cd media-server
```

### 安装依赖

```bash
cargo build
npm install
```

### 启动（开发模式）

**方式一 — Web 界面（推荐）**

```bash
# 终端 1：后端 API（端口 8080）
cargo run

# 终端 2：前端（端口 1420）
npm run dev
```

浏览器访问 http://localhost:1420

**方式二 — Windows 一键启动**

```bat
start-dev.bat
```

**方式三 — 仅后端**

```bash
cargo run
# API 地址：http://localhost:8080
```

**方式四 — Tauri 桌面应用**

```bash
npm run tauri dev
```

### 生产构建

```bash
npm run build          # 前端静态资源
cargo build --release  # 后端二进制
npm run tauri build    # 桌面应用（可选）
```

## 抖音功能使用

1. 在侧边栏打开 **Douyin** 页面
2. 粘贴分享链接或分享文案（如 `https://v.douyin.com/xxxxx`）
3. 点击 **Parse** 解析，再点击 **Play Video** 播放
4. 播放记录会自动写入 **Recent** 历史

支持的链接格式：

- 短链接：`https://v.douyin.com/xxxxx`
- 视频页：`https://www.douyin.com/video/xxxxx`
- 含链接的分享文案

## 项目结构

```
media-server/
├── src/                 # Rust 后端 + React 前端
│   ├── main.rs          # 服务入口
│   ├── lib.rs
│   ├── models/
│   ├── scanner/
│   ├── metadata/
│   ├── transcoder/
│   ├── server/          # Axum 路由与接口
│   ├── storage/         # JSON 持久化（~/.mediavault）
│   ├── douyin/          # 抖音解析与代理
│   ├── components/      # React 界面
│   ├── services/        # API 客户端
│   └── stores/
├── src-tauri/           # Tauri 桌面壳
├── package.json
└── Cargo.toml
```

## API 概览

| 模块 | 接口 | 说明 |
|------|------|------|
| 媒体库 | `GET /api/library` | 获取媒体列表 |
| 媒体库 | `POST /api/library/scan` | 扫描文件夹 |
| 串流 | `GET /api/stream/{id}/master.m3u8` | HLS 播放列表 |
| 历史 | `GET /api/history` | 最近播放（去重） |
| 历史 | `POST /api/history/douyin` | 记录抖音播放 |
| 历史 | `POST /api/history/{id}/progress` | 更新播放进度 |
| 抖音 | `POST /api/douyin/parse` | 解析分享链接 |
| 抖音 | `GET /api/douyin/proxy?url=...` | 代理视频流 |
| 配置 | `GET/PUT /api/config` | 应用设置 |
| 统计 | `GET /api/stats` | 库统计信息 |

默认服务端口：**8080**

## 配置说明

配置文件位于 `~/.mediavault/config.json`：

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

## 开发

```bash
cargo test
cargo fmt
cargo clippy
```

## 参与贡献

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/my-feature`）
3. 提交更改
4. 推送并发起 Pull Request

## 致谢

- [Axum](https://github.com/tokio-rs/axum) — HTTP 框架
- [Tauri](https://tauri.app/) — 桌面应用
- [FFmpeg](https://ffmpeg.org/) — 转码
- [Plyr](https://plyr.io/) & [HLS.js](https://github.com/video-dev/hls.js) — 播放器
