@echo off
setlocal

:: Set paths
set PROJECT_ROOT=%~dp0
set BIN_DIR=%PROJECT_ROOT%bin
set CONFIG_DIR=%PROJECT_ROOT%config
set LOG_DIR=%PROJECT_ROOT%logs
set PID_FILE=%PROJECT_ROOT%telemetry_tool.pid

:: Create necessary directories if they don't exist
if not exist %BIN_DIR% mkdir %BIN_DIR%
if not exist %CONFIG_DIR% mkdir %CONFIG_DIR%
if not exist %LOG_DIR% mkdir %LOG_DIR%

:: Build the Rust project
echo Building the project...
cargo build --release

if %ERRORLEVEL% neq 0 (
    echo Build failed. Exiting...
    exit /b %ERRORLEVEL%
)

:: Copy the executable to the bin directory
echo Copying executable to bin directory...
copy /Y %PROJECT_ROOT%target\release\telemetry_tool.exe %BIN_DIR%

if %ERRORLEVEL% neq 0 (
    echo Failed to copy the executable. Exiting...
    exit /b %ERRORLEVEL%
)

:: Start the executable in the background
echo Starting the application...
start "" /B %BIN_DIR%\telemetry_tool.exe

:: Get the PID of the last started process and save it to a file
echo %! > %PID_FILE%

echo Application started with PID %!

endlocal
pause
