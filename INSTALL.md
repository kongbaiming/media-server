# MediaVault 安装指南

## 系统要求

### 必需软件
- **Rust** 1.70+ - [安装指南](https://www.rust-lang.org/tools/install)
- **Node.js** 18+ - [下载](https://nodejs.org/)
- **npm** 或 **yarn**

### 可选软件
- **FFmpeg** - 用于转码功能 [下载](https://ffmpeg.org/download.html)

## Windows 安装步骤

### 1. 安装 Rust

访问 https://www.rust-lang.org/tools/install 下载并运行 `rustup-init.exe`

安装完成后，打开新的命令行窗口，验证安装：
```bash
rustc --version
cargo --version
```

### 2. 安装 Node.js

访问 https://nodejs.org/ 下载 LTS 版本并安装

验证安装：
```bash
node --version
npm --version
```

### 3. 安装 FFmpeg (可选)

#### 方法 1: 使用 Chocolatey
```bash
choco install ffmpeg
```

#### 方法 2: 使用 Scoop
```bash
scoop install ffmpeg
```

#### 方法 3: 手动安装
1. 访问 https://github.com/BtbN/FFmpeg-Builds/releases
2. 下载 `ffmpeg-master-latest-win64-gpl.zip`
3. 解压到 `C:\ffmpeg`
4. 将 `C:\ffmpeg\bin` 添加到系统 PATH 环境变量

验证安装：
```bash
ffmpeg -version
```

### 4. 克隆项目

```bash
git clone <repository-url>
cd media-server
```

### 5. 安装依赖并运行

#### 方法 1: 使用启动脚本
```bash
start.bat
```

#### 方法 2: 手动启动
```bash
# 安装前端依赖
npm install

# 启动开发模式
npm run tauri dev
```

## Linux/macOS 安装步骤

### 1. 安装 Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. 安装 Node.js

#### Ubuntu/Debian
```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

#### macOS (使用 Homebrew)
```bash
brew install node
```

### 3. 安装 FFmpeg (可选)

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install ffmpeg
```

#### macOS
```bash
brew install ffmpeg
```

### 4. 安装依赖并运行
```bash
npm install
npm run tauri dev
```

## 常见问题

### Q: 编译错误 "linking with `cc` failed"

安装系统开发工具：

#### Ubuntu/Debian
```bash
sudo apt install build-essential pkg-config libssl-dev
```

#### macOS
```bash
xcode-select --install
```

### Q: FFmpeg 未找到

确保 FFmpeg 已安装并在 PATH 中：
```bash
ffmpeg -version
```

如果显示 "command not found"，请重新安装 FFmpeg 或手动添加到 PATH。

### Q: 端口被占用

默认端口：
- 后端 API: 8080
- 前端开发: 1420

修改端口编辑 `~/.mediavault/config.json`：
```json
{
  "server_port": 9090
}
```

### Q: 前端显示 "Loading settings..."

这表示后端服务器未启动。确保：
1. 运行 `npm run tauri dev` 而不是 `npm run dev`
2. 或者单独启动后端：`cargo run`

## 开发模式

### 仅启动后端
```bash
cargo run
```

### 仅启动前端
```bash
npm run dev
```

### 启动完整应用 (Tauri)
```bash
npm run tauri dev
```

### 构建生产版本
```bash
npm run tauri build
```

## 项目结构

```
media-server/
├── src/                    # Rust 后端代码
│   ├── main.rs            # 程序入口
│   ├── models/            # 数据模型
│   ├── scanner/           # 文件扫描
│   ├── metadata/          # 元数据提取
│   ├── transcoder/        # 转码服务
│   ├── server/            # HTTP 服务器
│   └── storage/           # 数据存储
├── src/                   # React 前端代码
│   ├── components/        # UI 组件
│   ├── stores/            # 状态管理
│   └── services/          # API 服务
├── src-tauri/             # Tauri 配置
├── Cargo.toml             # Rust 依赖
└── package.json           # Node.js 依赖
```

## 配置文件位置

- Windows: `C:\Users\<用户名>\.mediavault\`
- Linux/macOS: `~/.mediavault/`

配置文件：
- `config.json` - 应用配置
- `library.json` - 媒体库数据
- `history.json` - 播放历史
- `thumbnails/` - 缩略图缓存
- `transcode/` - 转码缓存
