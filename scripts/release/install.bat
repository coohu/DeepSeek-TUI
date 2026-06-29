@echo off
setlocal enabledelayedexpansion
:: DeepSeek Windows installer
:: Copies deepseek.exe, codew.exe, and deepseek-tui.exe to %USERPROFILE%\bin

set "BIN_DIR=%USERPROFILE%\bin"
set "SCRIPT_DIR=%~dp0"

if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"

echo Installing deepseek to %BIN_DIR%...

copy /Y "%SCRIPT_DIR%deepseek.exe" "%BIN_DIR%\deepseek.exe" >nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: Failed to copy deepseek.exe
    exit /b 1
)

copy /Y "%SCRIPT_DIR%codew.exe" "%BIN_DIR%\codew.exe" >nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: Failed to copy codew.exe
    exit /b 1
)

copy /Y "%SCRIPT_DIR%deepseek-tui.exe" "%BIN_DIR%\deepseek-tui.exe" >nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: Failed to copy deepseek-tui.exe
    exit /b 1
)

echo.
echo Done. Commands installed to %BIN_DIR%.
echo.
echo Add %BIN_DIR% to your PATH:
echo   1. Open Start, search "environment variables"
echo   2. Click "Environment Variables..."
echo   3. Under "User variables", select "Path" and click "Edit"
echo   4. Click "New" and add: %BIN_DIR%
echo   5. Click OK, then restart your terminal
echo.
echo Or run this in an admin PowerShell:
echo   [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';%BIN_DIR%', 'User')
echo.
echo Then run: deepseek
