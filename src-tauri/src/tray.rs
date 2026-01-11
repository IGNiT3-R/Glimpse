use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

/// 创建系统托盘
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // 创建托盘菜单
    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)
        .map_err(|e| format!("创建菜单项失败: {}", e))?;

    let scan_full_item = MenuItem::with_id(
        app,
        "scan_full",
        "全屏扫描 (Ctrl+Shift+S)",
        true,
        None::<&str>,
    )
    .map_err(|e| format!("创建菜单项失败: {}", e))?;

    let scan_region_item = MenuItem::with_id(
        app,
        "scan_region",
        "区域扫描 (Ctrl+Shift+A)",
        true,
        None::<&str>,
    )
    .map_err(|e| format!("创建菜单项失败: {}", e))?;

    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| format!("创建菜单项失败: {}", e))?;

    let menu = Menu::with_items(
        app,
        &[&show_item, &scan_full_item, &scan_region_item, &quit_item],
    )
    .map_err(|e| format!("创建菜单失败: {}", e))?;

    // 构建托盘图标
    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("灵视 Glimpse")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "scan_full" => {
                // 触发全屏扫描
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("trigger_scan_full", ());
                }
            }
            "scan_region" => {
                // 触发区域扫描
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("trigger_scan_region", ());
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 处理托盘图标点击事件
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)
        .map_err(|e| format!("构建托盘图标失败: {}", e))?;

    Ok(())
}
