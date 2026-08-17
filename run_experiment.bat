@echo off
setlocal
cd /d "%~dp0"

set "CODEX_PYTHON=%USERPROFILE%\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe"
if exist "%CODEX_PYTHON%" (
    "%CODEX_PYTHON%" -c "import PIL" >nul 2>nul
    if not errorlevel 1 goto run_codex_python
)

where py >nul 2>nul
if not errorlevel 1 (
    py -3 -c "import PIL" >nul 2>nul
    if not errorlevel 1 goto run_py_launcher
)

where python >nul 2>nul
if not errorlevel 1 (
    python -c "import PIL" >nul 2>nul
    if not errorlevel 1 goto run_python
)

echo FATAL: Python 3 with Pillow is required. 1>&2
exit /b 1

:run_codex_python
"%CODEX_PYTHON%" -B "%~dp0tools\experiment\run_experiment.py" %*
set "RUN_RC=%ERRORLEVEL%"
exit /b %RUN_RC%

:run_py_launcher
py -3 -B "%~dp0tools\experiment\run_experiment.py" %*
set "RUN_RC=%ERRORLEVEL%"
exit /b %RUN_RC%

:run_python
python "%~dp0tools\experiment\run_experiment.py" %*
set "RUN_RC=%ERRORLEVEL%"
exit /b %RUN_RC%
