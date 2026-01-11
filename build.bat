@echo off
chcp 65001 >nul
echo ======================================
echo   灵视 Glimpse 构建脚本
echo ======================================
echo.

echo [1/4] 检查环境...
where rustc >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ 错误: 未找到 Rust 工具链
    echo 请访问 https://rustup.rs 安装 Rust
    pause
    exit /b 1
)

where pnpm >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ 错误: 未找到 pnpm
    echo 请运行: npm install -g pnpm
    pause
    exit /b 1
)

echo ✓ 环境检查通过
echo.

echo [2/4] 安装依赖...
call pnpm install
if %ERRORLEVEL% NEQ 0 (
    echo ❌ 依赖安装失败
    pause
    exit /b 1
)
echo ✓ 依赖安装完成
echo.

echo [3/4] 构建项目 (Release 模式)...
echo 这可能需要几分钟时间...
call pnpm tauri build --no-bundle
if %ERRORLEVEL% NEQ 0 (
    echo ❌ 构建失败
    echo 请检查上方错误信息
    pause
    exit /b 1
)
echo ✓ 构建完成
echo.

echo [4/4] 复制到 dist-portable...
if not exist "dist-portable" mkdir "dist-portable"
copy /Y "src-tauri\target\release\glimpse.exe" "dist-portable\glimpse.exe"
echo.

echo ======================================
echo   构建成功完成!
echo   输出: dist-portable\glimpse.exe
echo ======================================
echo.
pause
