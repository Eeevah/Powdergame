@echo off
setlocal
cd /d "%~dp0"
set "RUST_LOG=warn"

echo [Powdergame] Building G7 Activity Observatory...
cargo build --release -p powdergame-windows
if errorlevel 1 goto :build_error

echo.
echo ===================================================
echo Powdergame G7 Active / Sleep Observatory
echo Controls:
echo   SPACE = Play / Pause
echo   F     = Fast-forward x1 / x4 / x16
echo   N     = Step exactly 1 tick (while paused)
echo   R     = Reset simulation and metrics (back to x1)
echo   ESC   = Quit
echo ===================================================
"%~dp0target\release\powdergame-windows.exe" --activity-demo
exit /b %ERRORLEVEL%

:build_error
echo.
echo [Powdergame] Build failed.
pause
exit /b 1
