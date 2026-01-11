@echo off
chcp 65001 >nul
echo ======================================
echo   灵视 Glimpse 开发模式启动
echo ======================================
echo.

echo [1/2] 检查依赖...
if not exist "node_modules" (
    echo 📦 正在安装依赖...
    call pnpm install
    if %ERRORLEVEL% NEQ 0 (
        echo ❌ 依赖安装失败
        pause
        exit /b 1
    )
)
echo ✓ 依赖检查完成
echo.

echo [2/2] 启动开发服务器...
echo 🚀 正在启动 Tauri 开发模式...
echo 提示: 按 Ctrl+C 停止服务器
echo.
call pnpm tauri dev
