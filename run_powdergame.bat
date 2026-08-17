@echo off
setlocal
cd /d "%~dp0"
if not defined RUST_LOG set "RUST_LOG=warn"
set "APP=%~dp0target\release\powdergame-windows.exe"
set "APP_ARGS=%*"

if "%~1"=="" goto build

set "FIRST_ARG=%~1"
if "%FIRST_ARG:~0,2%"=="--" goto build

if not "%~2"=="" goto usage
if /i "%~1"=="normal" set "APP_ARGS="& goto build
if /i "%~1"=="movement" set "APP_ARGS=--movement-demo"& goto build
if /i "%~1"=="density" set "APP_ARGS=--density-demo"& goto build
if /i "%~1"=="thermal" set "APP_ARGS=--thermal-demo"& goto build
if /i "%~1"=="pressure" set "APP_ARGS=--pressure-demo"& goto build
if /i "%~1"=="parallel-integrity" set "APP_ARGS=--parallel-integrity-demo"& goto build
if /i "%~1"=="activity" set "APP_ARGS=--activity-demo"& goto build
if /i "%~1"=="gallery" set "APP_ARGS=--benchmark-gallery"& goto build
goto usage

:build
echo [Powdergame] Building the canonical app binary...
cargo build --locked --release -p powdergame-windows
set "BUILD_RC=%ERRORLEVEL%"
if not "%BUILD_RC%"=="0" exit /b %BUILD_RC%

if not exist "%APP%" (
    echo [Powdergame] ERROR: canonical app binary was not produced: 1>&2
    echo   %APP% 1>&2
    exit /b 1
)

"%APP%" %APP_ARGS%
set "APP_RC=%ERRORLEVEL%"
exit /b %APP_RC%

:usage
echo Usage: run_powdergame.bat [normal^|movement^|density^|thermal^|pressure^|parallel-integrity^|activity^|gallery^|app CLI args...] 1>&2
exit /b 2
