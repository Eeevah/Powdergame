@echo off
setlocal
cd /d "%~dp0"
set "RUST_LOG=warn"

echo [Powdergame] Building G8-B Benchmark Scenario Gallery...
cargo build --locked --release -p powdergame-windows
if errorlevel 1 goto :build_error

echo.
echo ==============================================================
echo Powdergame G8-B Benchmark Scenario Gallery
echo Scenarios:
echo   1 = Sand Fall
echo   2 = Water Flow
echo   3 = Fire / Heat
echo   4 = Pressure Burst
echo   5 = Heavy Mixed World
echo   6 = G7 Active / Sleep Regression
echo Controls:
echo   1-6   = Switch scenario ^(starts paused^)
echo   SPACE = Play / Pause
echo   F     = Fast-forward x1 / x4 / x16
echo   N     = Step exactly 1 simulation tick ^(while paused^)
echo   R     = Exact pristine reset ^(back to paused, x1^)
echo   ESC   = Quit
echo ==============================================================
"%~dp0target\release\powdergame-windows.exe" --benchmark-gallery
exit /b %ERRORLEVEL%

:build_error
echo.
echo [Powdergame] Gallery build failed.
pause
exit /b 1
