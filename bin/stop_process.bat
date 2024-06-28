@echo off
setlocal

:: Set paths
set PROJECT_ROOT=%~dp0
set PID_FILE=%PROJECT_ROOT%telemetry_tool.pid

:: Check if PID file exists
if not exist %PID_FILE% (
    echo PID file not found. Is the application running?
    endlocal
    pause
    exit /b 1
)

:: Read the PID from the file
set /p PID=<%PID_FILE%

:: Kill the process
echo Stopping application with PID %PID%...
taskkill /PID %PID% /F

if %ERRORLEVEL% neq 0 (
    echo Failed to stop the application.
    endlocal
    pause
    exit /b 1
)

:: Delete the PID file
del %PID_FILE%

echo Application stopped.

endlocal
pause
