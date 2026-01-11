use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// 创建并显示选择器窗口
pub fn create_selector_window(app: &AppHandle) -> Result<(), String> {
    create_selector_window_with_mode(app, "qr")
}

/// 创建选择器窗口（带模式参数）
pub fn create_selector_window_with_mode(app: &AppHandle, mode: &str) -> Result<(), String> {
    // 如果窗口已存在，先关闭它
    if let Some(old_window) = app.get_webview_window("selector") {
        println!("[DEBUG] 发现旧的选择器窗口，正在关闭...");
        let _ = old_window.close();
        // 等待窗口完全关闭
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 再次检查确保窗口已关闭
    if app.get_webview_window("selector").is_some() {
        println!("[WARN] 选择器窗口仍然存在，强制等待...");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 创建全屏透明覆盖窗口，通过 URL 参数传递模式
    let url = format!("selector.html?mode={}", mode);
    let _selector_window = WebviewWindowBuilder::new(
        app,
        "selector",
        WebviewUrl::App(url.into()),
    )
    .title("区域选择")
    .fullscreen(true)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .build()
    .map_err(|e| format!("创建选择窗口失败: {}", e))?;

    println!("[DEBUG] 选择器窗口已创建，模式: {}", mode);
    Ok(())
}

/// 关闭区域选择窗口
pub fn close_selector_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("selector") {
        println!("[DEBUG] 正在关闭选择器窗口...");
        // 使用 close 而不是 destroy，更安全
        window.close().map_err(|e| format!("关闭选择窗口失败: {}", e))?;
        // 等待窗口关闭
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Ok(())
}

/// 预加载函数（不再使用，保留空实现以兼容）
pub fn preload_selector_window(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}
