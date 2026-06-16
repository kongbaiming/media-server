# MediaVault - Local Media Server

A powerful local media server built with Rust + Tauri, featuring media library management, transcoding, and HLS streaming.

## Features

- 📁 **Media Library Management** - Scan and organize your media files
- 🎬 **Video Playback** - Built-in HLS player with progress tracking
- 🎵 **Music Support** - Full audio file support
- 🔄 **Real-time Transcoding** - FFmpeg-powered transcoding with HLS output
- 🔍 **Search** - Full-text search across your media library
- ❤️ **Favorites** - Mark your favorite media
- 📊 **Statistics** - Track your media library stats
- ⚙️ **Configurable** - Customize library paths, server port, and more

## Supported Formats

### Video
MP4, MKV, AVI, MOV, WMV, FLV, WebM, TS, M2TS, MPG, MPEG, 3GP, OGV, VOB

### Audio
MP3, FLAC, AAC, OGG, WAV, WMA, M4A, OPUS, APE, ALAC

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- [FFmpeg](https://ffmpeg.org/download.html) (must be in PATH)

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/media-vault.git
cd media-vault
```

### 2. Install dependencies

```bash
# Install Rust dependencies
cargo build

# Install frontend dependencies
npm install
```

### 3. Run in development mode

```bash
# Run with Tauri (desktop app)
npm run tauri dev

# Or run backend only (API server)
cargo run
```

### 4. Build for production

```bash
npm run tauri build
```

## Project Structure

```
media-server/
├── src/                    # Rust backend
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library exports
│   ├── models/            # Data models
│   ├── scanner/           # Media file scanner
│   ├── metadata/          # Metadata extraction
│   ├── transcoder/        # FFmpeg transcoding
│   ├── server/            # HTTP API server
│   └── storage/           # JSON storage manager
├── src-tauri/             # Tauri desktop app
├── src/                   # React frontend
│   ├── components/        # UI components
│   ├── stores/            # State management
│   ├── services/          # API services
│   └── types/             # TypeScript types
├── static/                # Static assets
└── package.json           # Frontend dependencies
```

## API Endpoints

### Media Library
- `GET /api/library` - List all media
- `GET /api/library/:id` - Get media details
- `DELETE /api/library/:id` - Delete media
- `POST /api/library/scan` - Start library scan
- `GET /api/library/scan/progress` - Get scan progress

### Search
- `GET /api/search?q=:query` - Search media

### Favorites
- `GET /api/favorites` - List favorites
- `POST /api/favorites/:id` - Toggle favorite

### Playback
- `GET /api/stream/:id/master.m3u8` - HLS stream
- `GET /api/stream/:id/thumbnail` - Get thumbnail
- `GET /api/stream/:id/direct` - Direct stream URL

### History
- `GET /api/history` - Get play history
- `POST /api/history/:id/progress` - Update progress

### Transcoding
- `POST /api/transcode` - Start transcoding
- `GET /api/transcode/:id` - Get transcode status

### Configuration
- `GET /api/config` - Get configuration
- `PUT /api/config` - Update configuration

### Statistics
- `GET /api/stats` - Get library statistics

## Usage

1. **Add Library Paths**: Go to Settings and add your media folders
2. **Scan Library**: Click "Scan Library" to index your media files
3. **Browse Media**: Use the Library view to browse your collection
4. **Play Media**: Click on any media to start playback
5. **Search**: Use the search bar to find specific media
6. **Favorites**: Click the heart icon to add media to favorites

## Configuration

The application stores its configuration in `~/.mediavault/config.json`:

```json
{
  "library_paths": ["C:/Users/Videos", "D:/Music"],
  "auto_scan": true,
  "scan_interval": 300,
  "transcode_quality": "Auto",
  "hardware_acceleration": false,
  "server_port": 8080
}
```

## Development

### Running tests

```bash
cargo test
```

### Code formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Desktop application framework
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [FFmpeg](https://ffmpeg.org/) - Media processing
- [Plyr](https://plyr.io/) - Media player
- [HLS.js](https://github.com/video-dev/hls.js) - HLS player
