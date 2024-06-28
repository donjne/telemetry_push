@echo off
:: Batch script to run telemetry_tool.exe as an administrator
SETLOCAL
SET EXE=telemetry_tool.exe
:: Check for admin privileges
NET SESSION >NUL 2>&1
IF ERRORLEVEL 1 (
    :: Not running as admin, relaunch script with admin privileges
    ECHO Requesting administrative privileges...
    POWERSHELL START -Verb runAs -ArgumentList '%EXE%'
    EXIT /B
)
:: Run the executable with administrative privileges
%EXE%
ENDLOCAL
