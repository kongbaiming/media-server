@echo off
setlocal

echo ==============================================
echo   MediaVault Desktop App
echo ==============================================
echo.
echo   Backend API  : http://127.0.0.1:8080
echo   Window       : closes to system tray
echo   Quit         : tray icon -^> Quit, or File -^> Quit MediaVault
echo.
echo ==============================================
echo.

cd /d %~dp0
npm run tauri dev
