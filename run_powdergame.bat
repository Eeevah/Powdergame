@echo off
setlocal
cd /d "%~dp0"
set "RUST_LOG=warn"
set "APP_MODE="

if "%~1"=="" goto build
if not "%~2"=="" goto usage
if /i "%~1"=="normal" goto build
if /i "%~1"=="movement" set "APP_MODE=--movement-demo"& goto build
if /i "%~1"=="density" set "APP_MODE=--density-demo"& goto build
if /i "%~1"=="thermal" set "APP_MODE=--thermal-demo"& goto build
if /i "%~1"=="pressure" set "APP_MODE=--pressure-demo"& goto build
if /i "%~1"=="parallel-integrity" set "APP_MODE=--parallel-integrity-demo"& goto build
if /i "%~1"=="activity" set "APP_MODE=--activity-demo"& goto build
if /i "%~1"=="gallery" set "APP_MODE=--benchmark-gallery"& goto build
goto usage

:build
echo [Powdergame] Building the canonical app binary...
cargo build --locked --release -p powdergame-windows
if errorlevel 1 exit /b %ERRORLEVEL%

set "APP=%~dp0target\release\powdergame-windows.exe"
if not exist "%APP%" (
    echo [Powdergame] ERROR: canonical app binary was not produced: 1>&2
    echo   %APP% 1>&2
    exit /b 1
)

"%APP%" %APP_MODE%
exit /b %ERRORLEVEL%

:usage
echo Usage: run_powdergame.bat [normal^|movement^|density^|thermal^|pressure^|parallel-integrity^|activity^|gallery] 1>&2
exit /b 2
