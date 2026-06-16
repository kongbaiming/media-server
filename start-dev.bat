@echo off
echo Starting MediaVault Development Environment...
echo.

echo [1/2] Starting Backend Server...
start "MediaVault Backend" cmd /k "cd /d %~dp0 && cargo run"

echo [2/2] Waiting for backend to initialize...
timeout /t 5 /nobreak > nul

echo [3/2] Starting Frontend...
start "MediaVault Frontend" cmd /k "cd /d %~dp0 && npm run dev"

echo.
echo MediaVault is starting...
echo Backend: http://localhost:8080
echo Frontend: http://localhost:1420
echo.
echo Press any key to exit this window...
pause > nul
