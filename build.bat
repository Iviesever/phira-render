@echo off
setlocal enabledelayedexpansion

echo ===================================================
echo       Phira-Render Build and Package Script
echo ===================================================
echo.

set "ROOT_DIR=%~dp0"
cd /d "%ROOT_DIR%"

echo [1/3] Building frontend assets (npm run build)...
call npm run build
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Frontend build failed. Please check your code.
    pause
    exit /b %ERRORLEVEL%
)
echo [OK] Frontend build succeeded.
echo.

echo [2/3] Setting up local Rust / Cargo environment...
set "RUSTUP_HOME=%ROOT_DIR%.rustup"
set "CARGO_HOME=%ROOT_DIR%.cargo"

if not exist "!CARGO_HOME!\bin\cargo.exe" (
    if exist "%ROOT_DIR%..\..\.cargo\bin\cargo.exe" (
        set "CARGO_HOME=%ROOT_DIR%..\..\.cargo"
        set "RUSTUP_HOME=%ROOT_DIR%..\..\.rustup"
    ) else if exist "%ROOT_DIR%..\.cargo\bin\cargo.exe" (
        set "CARGO_HOME=%ROOT_DIR%..\.cargo"
        set "RUSTUP_HOME=%ROOT_DIR%..\.rustup"
    )
)

if exist "!CARGO_HOME!\bin\cargo.exe" (
    set "PATH=!CARGO_HOME!\bin;%PATH%"
)

echo [3/3] Building release binary (cargo build --release --features custom-protocol)...
cargo build --release --features custom-protocol --manifest-path "%ROOT_DIR%src-tauri\Cargo.toml"
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Rust backend build failed.
    pause
    exit /b %ERRORLEVEL%
)

if not exist "%ROOT_DIR%src-tauri\target\release\assets" (
    echo Syncing assets directory...
    xcopy /E /I /Y "%ROOT_DIR%src-tauri\assets" "%ROOT_DIR%src-tauri\target\release\assets" >nul
)

echo.
echo ===================================================
echo [SUCCESS] phira-render.exe built successfully!
echo Output: %ROOT_DIR%src-tauri\target\release\phira-render.exe
echo ===================================================
echo.
pause
