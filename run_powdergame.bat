@echo off
setlocal
cd /d "%~dp0"

set "APP=%~dp0target\release\powdergame-windows.exe"

cargo build --locked --release -p powdergame-windows
set "BUILD_RC=%ERRORLEVEL%"
if not "%BUILD_RC%"=="0" exit /b %BUILD_RC%

if not exist "%APP%" (
    echo FATAL: canonical Powdergame executable was not produced: "%APP%" 1>&2
    exit /b 1
)

"%APP%" %*
set "APP_RC=%ERRORLEVEL%"
exit /b %APP_RC%
