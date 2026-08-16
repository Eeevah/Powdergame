@echo off
cd /d "%~dp0"
echo ===================================================
echo Powdergame G5 Boiler User Validation Demo
echo Controls:
echo   SPACE = Play / Pause
echo   N     = Step 1 tick (while paused)
echo   R     = Reset simulation
echo   ESC   = Quit
echo ===================================================
cargo run -p powdergame-windows -- --pressure-demo
pause
