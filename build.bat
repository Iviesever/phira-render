@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo ===================================================
echo       Phira-Render 一键构建与打包脚本
echo ===================================================
echo.

set "ROOT_DIR=%~dp0"
cd /d "%ROOT_DIR%"

echo [1/3] 正在编译前端静态资源 (npm run build)...
call npm run build
if %ERRORLEVEL% neq 0 (
    echo.
    echo [错误] 前端编译失败，请检查代码！
    pause
    exit /b %ERRORLEVEL%
)
echo [OK] 前端编译成功！
echo.

echo [2/3] 配置本地 Rust / Cargo 编译环境...
set "RUSTUP_HOME=%ROOT_DIR%.rustup"
set "CARGO_HOME=%ROOT_DIR%.cargo"
set "PATH=%ROOT_DIR%.cargo\bin;%PATH%"

echo [3/3] 正在编译 Release 离线可执行程序 (cargo build --release --features custom-protocol)...
cargo build --release --features custom-protocol --manifest-path "%ROOT_DIR%src-tauri\Cargo.toml"
if %ERRORLEVEL% neq 0 (
    echo.
    echo [错误] Rust 后端编译失败！
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo ===================================================
echo [成功] phira-render.exe 构建完成！
echo 产物路径: %ROOT_DIR%src-tauri\target\release\phira-render.exe
echo ===================================================
echo.
pause
