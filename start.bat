@echo off
echo ========================================
echo   MediaVault - Local Media Server
echo ========================================
echo.

echo Checking prerequisites...
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust is not installed!
    echo Please install Rust from https://www.rust-lang.org/tools/install
    pause
    exit /b 1
)

REM Check if Node.js is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not installed!
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

REM Check FFmpeg (optional)
where ffmpeg >nul 2>nul
if %errorlevel% neq 0 (
    echo [WARNING] FFmpeg is not installed.
    echo Transcoding features will be disabled.
    echo Install FFmpeg from https://ffmpeg.org/download.html
    echo.
)

echo Starting MediaVault...
echo.

echo [Step 1] Installing frontend dependencies...
call npm install
if %errorlevel% neq 0 (
    echo [ERROR] Failed to install dependencies
    pause
    exit /b 1
)

echo.
echo [Step 2] Starting development server...
echo.
echo Backend API: http://localhost:8080
echo Frontend:    http://localhost:1420
echo.
echo Press Ctrl+C to stop the server
echo.

call npm run tauri dev

pause
