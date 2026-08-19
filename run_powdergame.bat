@echo off
setlocal
cd /d "%~dp0"
if not defined RUST_LOG set "RUST_LOG=warn"
set "APP=%~dp0target\release\powdergame-windows.exe"
set "APP_ARGS=%*"

if "%~1"=="" goto route_gallery

set "FIRST_ARG=%~1"
if "%FIRST_ARG:~0,2%"=="--" goto build

if /i "%~1"=="sandbox" goto route_sandbox
if /i "%~1"=="play" goto route_sandbox

if not "%~2"=="" goto usage
if /i "%~1"=="normal" goto route_gallery
if /i "%~1"=="gallery" goto route_gallery
if /i "%~1"=="runtime" goto route_runtime
if /i "%~1"=="g0" goto route_runtime
if /i "%~1"=="movement" goto route_movement
if /i "%~1"=="density" goto route_density
if /i "%~1"=="thermal" goto route_thermal
if /i "%~1"=="pressure" goto route_pressure
if /i "%~1"=="parallel-integrity" goto route_parallel_integrity
if /i "%~1"=="activity" goto route_activity
goto usage

:route_gallery
set "APP_ARGS=--benchmark-gallery"
goto build

:route_runtime
set "APP_ARGS=--runtime-baseline"
goto build

:route_movement
set "APP_ARGS=--movement-demo"
goto build

:route_density
set "APP_ARGS=--density-demo"
goto build

:route_thermal
set "APP_ARGS=--thermal-demo"
goto build

:route_pressure
set "APP_ARGS=--pressure-demo"
goto build

:route_parallel_integrity
set "APP_ARGS=--parallel-integrity-demo"
goto build

:route_activity
set "APP_ARGS=--activity-demo"
goto build

:route_sandbox
set "APP_ARGS=--sandbox"
if "%~2"=="" goto build
if /i not "%~2"=="--smoke-frames" goto usage
if "%~3"=="" goto usage
if not "%~4"=="" goto usage
set "APP_ARGS=--sandbox --smoke-frames %~3"
goto build

:build
if defined POWDERGAME_LAUNCHER_AUDIT_ONLY (
    if "%POWDERGAME_LAUNCHER_AUDIT_ONLY%"=="%POWDERGAME_LAUNCHER_AUDIT_NONCE%" goto launcher_audit
)
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

:launcher_audit
echo POWDERGAME_LAUNCHER_AUDIT_ARGS=%APP_ARGS%
exit /b 0

:usage
echo Usage: run_powdergame.bat [sandbox^|play^|normal^|gallery^|runtime^|g0^|movement^|density^|thermal^|pressure^|parallel-integrity^|activity^|app CLI args...] 1>&2
echo   default = Gallery ^(no args, normal, gallery^) 1>&2
echo   sandbox/play = G9-A first playable Sandbox 1>&2
echo   runtime/g0 = technical empty G0 baseline 1>&2
exit /b 2
