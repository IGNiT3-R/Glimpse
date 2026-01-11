mod qr_scanner;
mod screenshot;
mod tray;
mod selector_window;
mod ocr;

use qr_scanner::{QrCodeResult, scan_qr_codes, scan_multiple_images};
use screenshot::{capture_full_screen, capture_region};
use ocr::OcrResult;
use tauri::{AppHandle, Emitter, Manager};
use selector_window::{create_selector_window, create_selector_window_with_mode, close_selector_window, preload_selector_window};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use image::DynamicImage;

// 全局存储：预先截取的全屏图像和 base64 数据
static CAPTURED_SCREENSHOT: Mutex<Option<DynamicImage>> = Mutex::new(None);
static CAPTURED_SCREENSHOT_BASE64: Mutex<Option<String>> = Mutex::new(None);

// 退出行为设置：true = 最小化到托盘，false = 直接退出
static MINIMIZE_TO_TRAY: AtomicBool = AtomicBool::new(false);

// 当前快捷键配置
static CURRENT_FULLSCREEN_SHORTCUT: Mutex<String> = Mutex::new(String::new());
static CURRENT_REGION_SHORTCUT: Mutex<String> = Mutex::new(String::new());
static CURRENT_OCR_SHORTCUT: Mutex<String> = Mutex::new(String::new());

/// 全屏扫描命令
#[tauri::command]
async fn scan_full_screen() -> Result<Vec<QrCodeResult>, String> {
    println!("[DEBUG] 开始全屏扫描");

    // 捕获所有屏幕
    println!("[DEBUG] 正在捕获屏幕...");
    let images = capture_full_screen()?;
    println!("[DEBUG] 成功捕获 {} 个屏幕", images.len());

    // 扫描所有图像中的二维码
    println!("[DEBUG] 正在扫描二维码...");
    let results = scan_multiple_images(&images)?;
    println!("[DEBUG] 扫描完成，找到 {} 个二维码", results.len());

    Ok(results)
}

/// 开始区域选择 - 先截图再显示选择窗口
#[tauri::command]
async fn start_region_selection(app: AppHandle) -> Result<(), String> {
    println!("[DEBUG] ======== 开始区域选择 ========");

    // 1. 隐藏主窗口（确保截图不包含主窗口）
    // 使用 minimize + hide 组合，比单独 hide 更快完成
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
        let _ = window.hide();
    }

    // 使用异步等待，不阻塞线程（30ms 测试极限值）
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

    // 2. 截取全屏
    println!("[DEBUG] 正在截取全屏...");
    let images = capture_full_screen()?;
    let screenshot = images.into_iter().next().ok_or("未找到可用屏幕")?;

    println!("[DEBUG] 截图尺寸: {}x{}", screenshot.width(), screenshot.height());

    // 3. 将截图转换为 base64
    use screenshot::image_to_png_bytes;
    let png_bytes = image_to_png_bytes(&screenshot)?;
    let base64 = base64_encode(&png_bytes);
    let data_url = format!("data:image/png;base64,{}", base64);

    println!("[DEBUG] 截图已转换为 base64，长度: {} 字节", data_url.len());

    // 4. 保存截图和 base64 到全局变量
    {
        let mut guard = CAPTURED_SCREENSHOT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *guard = Some(screenshot);
    }
    {
        let mut guard = CAPTURED_SCREENSHOT_BASE64.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *guard = Some(data_url);
    }

    // 5. 显示选择器窗口
    create_selector_window(&app)?;

    println!("[DEBUG] 选择器窗口已显示");

    Ok(())
}

/// 获取预先截取的截图数据（供选择器窗口调用）
#[tauri::command]
async fn get_screenshot_data() -> Result<String, String> {
    println!("[DEBUG] 选择器窗口请求截图数据");

    let data_url = {
        let guard = CAPTURED_SCREENSHOT_BASE64.lock().map_err(|e| format!("锁定失败: {}", e))?;
        guard.clone().ok_or("未找到截图数据")?
    };

    println!("[DEBUG] 返回截图数据，长度: {} 字节", data_url.len());
    Ok(data_url)
}

/// 完成区域选择并扫描 - 直接裁剪预先保存的截图
#[tauri::command]
async fn complete_region_selection(
    app: AppHandle,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    println!("[DEBUG] ======== 区域选择完成 ========");
    println!("[DEBUG] 前端传来的坐标: x={}, y={}, width={}, height={}", x, y, width, height);

    // 从全局变量获取预先截取的图像（在关闭窗口前获取）
    let screenshot = {
        let mut guard = CAPTURED_SCREENSHOT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        guard.take().ok_or("未找到预先截取的截图")?
    };

    // 清理 base64 数据
    {
        let mut guard = CAPTURED_SCREENSHOT_BASE64.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *guard = None;
    }

    // 关闭选择窗口
    close_selector_window(&app)?;

    // 等待窗口完全关闭
    std::thread::sleep(std::time::Duration::from_millis(50));

    println!("[DEBUG] 使用预先截取的截图，尺寸: {}x{}", screenshot.width(), screenshot.height());

    // 直接裁剪预先截取的图像（无需重新截图）
    let img_width = screenshot.width();
    let img_height = screenshot.height();

    // 确保裁剪区域不超出图像边界
    let safe_x = x.min(img_width.saturating_sub(1));
    let safe_y = y.min(img_height.saturating_sub(1));
    let safe_width = width.min(img_width.saturating_sub(safe_x));
    let safe_height = height.min(img_height.saturating_sub(safe_y));

    println!("[DEBUG] 安全裁剪区域: x={}, y={}, w={}, h={}", safe_x, safe_y, safe_width, safe_height);

    // 裁剪图像
    let cropped = screenshot.crop_imm(safe_x, safe_y, safe_width, safe_height);

    println!("[DEBUG] 裁剪完成，结果尺寸: {}x{}", cropped.width(), cropped.height());

    // 扫描二维码
    let results = scan_qr_codes(&cropped).unwrap_or_else(|e| {
        println!("[DEBUG] 二维码识别失败: {}，返回空结果", e);
        vec![]
    });

    println!("[DEBUG] 扫描完成，找到 {} 个二维码", results.len());

    // 显示主窗口
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    // 等待窗口完全显示
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 发送结果事件
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("region_scan_complete", &results);
        println!("[DEBUG] 已向主窗口发送 region_scan_complete 事件");
    }

    Ok(())
}

/// 取消区域选择
#[tauri::command]
async fn cancel_region_selection(app: AppHandle) -> Result<(), String> {
    // 关闭选择窗口
    close_selector_window(&app)?;

    // 显示主窗口
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    // 发送取消事件
    app.emit("region_scan_cancelled", ())
        .map_err(|e| format!("发送事件失败: {}", e))?;

    Ok(())
}

/// 开始区域扫描(先截图再由前端选择区域)
#[tauri::command]
async fn start_region_scan() -> Result<String, String> {
    // 捕获所有屏幕
    let images = capture_full_screen()?;

    // 对于简化实现,我们只返回第一个屏幕的截图信息
    if let Some(first_image) = images.first() {
        // 将图像转换为 base64 供前端显示
        use screenshot::image_to_png_bytes;
        let png_bytes = image_to_png_bytes(first_image)?;
        let base64 = base64_encode(&png_bytes);

        Ok(format!("data:image/png;base64,{}", base64))
    } else {
        Err("未找到可用屏幕".to_string())
    }
}

/// Base64 编码
fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = base64::write::EncoderWriter::new(&mut buf, &base64::engine::general_purpose::STANDARD);
        encoder.write_all(data).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

/// 区域扫描命令(基于坐标)
#[tauri::command]
async fn scan_region_coords(x: u32, y: u32, width: u32, height: u32) -> Result<Vec<QrCodeResult>, String> {
    println!("[DEBUG] 区域扫描 - 坐标: x={}, y={}, width={}, height={}", x, y, width, height);

    // 捕获指定区域
    let image = capture_region(x, y, width, height)?;
    println!("[DEBUG] 成功捕获区域，图像尺寸: {}x{}", image.width(), image.height());

    // 扫描二维码
    let results = scan_qr_codes(&image)?;

    Ok(results)
}

/// 显示主窗口
#[tauri::command]
async fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .show()
            .map_err(|e| format!("显示窗口失败: {}", e))?;
        window
            .unminimize()
            .map_err(|e| format!("还原窗口失败: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("窗口聚焦失败: {}", e))?;
    }
    Ok(())
}

/// 最小化主窗口
#[tauri::command]
async fn minimize_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .minimize()
            .map_err(|e| format!("最小化窗口失败: {}", e))?;
    }
    Ok(())
}

/// 隐藏主窗口
#[tauri::command]
async fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("隐藏窗口失败: {}", e))?;
    }
    Ok(())
}

/// 设置关闭窗口行为
#[tauri::command]
async fn set_close_behavior(behavior: String) -> Result<(), String> {
    let minimize_to_tray = behavior == "tray";
    MINIMIZE_TO_TRAY.store(minimize_to_tray, Ordering::SeqCst);
    println!("[DEBUG] 退出行为已设置为: {}", if minimize_to_tray { "最小化到托盘" } else { "直接退出" });
    Ok(())
}

/// 获取当前关闭窗口行为
#[tauri::command]
async fn get_close_behavior() -> Result<String, String> {
    let minimize_to_tray = MINIMIZE_TO_TRAY.load(Ordering::SeqCst);
    Ok(if minimize_to_tray { "tray".to_string() } else { "exit".to_string() })
}

/// 更新全局快捷键
#[tauri::command]
async fn update_shortcuts(
    app: AppHandle,
    fullscreen_shortcut: String,
    region_shortcut: String,
    ocr_shortcut: String,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    println!("[DEBUG] 更新快捷键: 全屏={}, 区域={}, OCR={}", fullscreen_shortcut, region_shortcut, ocr_shortcut);

    // 获取当前快捷键
    let (old_fullscreen, old_region, old_ocr) = {
        let fs = CURRENT_FULLSCREEN_SHORTCUT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        let rg = CURRENT_REGION_SHORTCUT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        let ocr = CURRENT_OCR_SHORTCUT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        (fs.clone(), rg.clone(), ocr.clone())
    };

    // 注销旧的快捷键（如果存在）
    if !old_fullscreen.is_empty() {
        if let Ok(shortcut) = old_fullscreen.parse::<Shortcut>() {
            let _ = app.global_shortcut().unregister(shortcut);
        }
    }
    if !old_region.is_empty() {
        if let Ok(shortcut) = old_region.parse::<Shortcut>() {
            let _ = app.global_shortcut().unregister(shortcut);
        }
    }
    if !old_ocr.is_empty() {
        if let Ok(shortcut) = old_ocr.parse::<Shortcut>() {
            let _ = app.global_shortcut().unregister(shortcut);
        }
    }

    // 注册新的全屏扫描快捷键
    let app_handle_full = app.clone();
    let shortcut_full: Shortcut = fullscreen_shortcut.parse()
        .map_err(|e| format!("解析全屏快捷键失败: {:?}", e))?;

    app.global_shortcut().on_shortcut(shortcut_full, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            if let Some(window) = app_handle_full.get_webview_window("main") {
                let _ = window.emit("trigger_scan_full", ());
            }
        }
    })
    .map_err(|e| format!("注册全屏扫描快捷键失败: {}", e))?;

    // 注册新的区域扫描快捷键
    let app_handle_region = app.clone();
    let shortcut_region: Shortcut = region_shortcut.parse()
        .map_err(|e| format!("解析区域快捷键失败: {:?}", e))?;

    app.global_shortcut().on_shortcut(shortcut_region, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            if let Some(window) = app_handle_region.get_webview_window("main") {
                let _ = window.emit("trigger_scan_region", ());
            }
        }
    })
    .map_err(|e| format!("注册区域扫描快捷键失败: {}", e))?;

    // 注册新的 OCR 快捷键
    let app_handle_ocr = app.clone();
    let shortcut_ocr: Shortcut = ocr_shortcut.parse()
        .map_err(|e| format!("解析 OCR 快捷键失败: {:?}", e))?;

    app.global_shortcut().on_shortcut(shortcut_ocr, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            if let Some(window) = app_handle_ocr.get_webview_window("main") {
                let _ = window.emit("trigger_ocr_region", ());
            }
        }
    })
    .map_err(|e| format!("注册 OCR 快捷键失败: {}", e))?;

    // 更新存储的快捷键
    {
        let mut fs = CURRENT_FULLSCREEN_SHORTCUT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *fs = fullscreen_shortcut;
    }
    {
        let mut rg = CURRENT_REGION_SHORTCUT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *rg = region_shortcut;
    }
    {
        let mut ocr = CURRENT_OCR_SHORTCUT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *ocr = ocr_shortcut;
    }

    println!("[DEBUG] 快捷键更新成功");
    Ok(())
}

/// 检查是否应该最小化到托盘
pub fn should_minimize_to_tray() -> bool {
    MINIMIZE_TO_TRAY.load(Ordering::SeqCst)
}

/// 开始 OCR 区域选择
#[tauri::command]
async fn start_ocr_region_selection(app: AppHandle) -> Result<(), String> {
    println!("[DEBUG] ======== 开始 OCR 区域选择 ========");

    // 1. 隐藏主窗口
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
        let _ = window.hide();
    }

    // 等待窗口隐藏
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

    // 2. 截取全屏
    println!("[DEBUG] 正在截取全屏...");
    let images = capture_full_screen()?;
    let screenshot = images.into_iter().next().ok_or("未找到可用屏幕")?;

    println!("[DEBUG] 截图尺寸: {}x{}", screenshot.width(), screenshot.height());

    // 3. 将截图转换为 base64
    use screenshot::image_to_png_bytes;
    let png_bytes = image_to_png_bytes(&screenshot)?;
    let base64 = base64_encode(&png_bytes);
    let data_url = format!("data:image/png;base64,{}", base64);

    // 4. 保存截图和 base64 到全局变量
    {
        let mut guard = CAPTURED_SCREENSHOT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *guard = Some(screenshot);
    }
    {
        let mut guard = CAPTURED_SCREENSHOT_BASE64.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *guard = Some(data_url);
    }

    // 5. 显示选择器窗口（OCR 模式）
    create_selector_window_with_mode(&app, "ocr")?;

    println!("[DEBUG] OCR 选择器窗口已显示");
    Ok(())
}

/// 完成 OCR 区域选择并识别
#[tauri::command]
async fn complete_ocr_region_selection(
    app: AppHandle,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    println!("[DEBUG] ======== OCR 区域选择完成 ========");
    println!("[DEBUG] 坐标: x={}, y={}, width={}, height={}", x, y, width, height);

    // 从全局变量获取预先截取的图像
    let screenshot = {
        let mut guard = CAPTURED_SCREENSHOT.lock().map_err(|e| format!("锁定失败: {}", e))?;
        guard.take().ok_or("未找到预先截取的截图")?
    };

    // 清理 base64 数据
    {
        let mut guard = CAPTURED_SCREENSHOT_BASE64.lock().map_err(|e| format!("锁定失败: {}", e))?;
        *guard = None;
    }

    // 关闭选择窗口
    close_selector_window(&app)?;

    // 等待窗口完全关闭
    std::thread::sleep(std::time::Duration::from_millis(50));

    println!("[DEBUG] 使用预先截取的截图，尺寸: {}x{}", screenshot.width(), screenshot.height());

    // 裁剪图像
    let img_width = screenshot.width();
    let img_height = screenshot.height();

    let safe_x = x.min(img_width.saturating_sub(1));
    let safe_y = y.min(img_height.saturating_sub(1));
    let safe_width = width.min(img_width.saturating_sub(safe_x));
    let safe_height = height.min(img_height.saturating_sub(safe_y));

    let cropped = screenshot.crop_imm(safe_x, safe_y, safe_width, safe_height);

    println!("[DEBUG] 裁剪完成，结果尺寸: {}x{}", cropped.width(), cropped.height());

    // 执行 OCR 识别
    let result = ocr::recognize_text(&cropped).unwrap_or_else(|e| {
        println!("[DEBUG] OCR 识别失败: {}，返回空结果", e);
        OcrResult {
            text: String::new(),
            language: "unknown".to_string(),
        }
    });

    println!("[DEBUG] OCR 识别完成，文本长度: {}", result.text.len());

    // 显示主窗口
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    // 等待窗口完全显示
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 发送结果事件，增加重试机制
    let mut retry_count = 0;
    while retry_count < 3 {
        if let Some(window) = app.get_webview_window("main") {
            match window.emit("ocr_scan_complete", &result) {
                Ok(_) => {
                    println!("[DEBUG] 已向主窗口发送 ocr_scan_complete 事件");
                    break;
                }
                Err(e) => {
                    println!("[WARN] 发送事件失败 (尝试 {}): {}", retry_count + 1, e);
                    retry_count += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        } else {
            println!("[WARN] 无法获取主窗口 (尝试 {})", retry_count + 1);
            retry_count += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            // 处理窗口关闭事件
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if should_minimize_to_tray() {
                    // 最小化到托盘，阻止关闭
                    api.prevent_close();
                    let _ = window.hide();
                }
                // 否则允许正常关闭
            }
        })
        .setup(|app| {
            // 创建系统托盘
            tray::create_tray(&app.handle())?;

            // 预加载选择器窗口（加速区域扫描）
            if let Err(e) = preload_selector_window(&app.handle()) {
                println!("[WARN] 预加载选择器窗口失败: {}", e);
            }

            // 注册全局快捷键
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

            let app_handle_full = app.handle().clone();
            let shortcut_full: Shortcut = "Ctrl+Shift+S".parse()
                .map_err(|e| format!("解析快捷键失败: {:?}", e))?;

            app.global_shortcut().on_shortcut(shortcut_full, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app_handle_full.get_webview_window("main") {
                        let _ = window.emit("trigger_scan_full", ());
                    }
                }
            })
            .map_err(|e| format!("注册全屏扫描快捷键失败: {}", e))?;

            let app_handle_region = app.handle().clone();
            let shortcut_region: Shortcut = "Ctrl+Shift+A".parse()
                .map_err(|e| format!("解析快捷键失败: {:?}", e))?;

            app.global_shortcut().on_shortcut(shortcut_region, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app_handle_region.get_webview_window("main") {
                        let _ = window.emit("trigger_scan_region", ());
                    }
                }
            })
            .map_err(|e| format!("注册区域扫描快捷键失败: {}", e))?;

            // 注册 OCR 快捷键
            let app_handle_ocr = app.handle().clone();
            let shortcut_ocr: Shortcut = "Ctrl+Shift+D".parse()
                .map_err(|e| format!("解析 OCR 快捷键失败: {:?}", e))?;

            app.global_shortcut().on_shortcut(shortcut_ocr, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = app_handle_ocr.get_webview_window("main") {
                        let _ = window.emit("trigger_ocr_region", ());
                    }
                }
            })
            .map_err(|e| format!("注册 OCR 快捷键失败: {}", e))?;

            // 初始化默认快捷键存储
            {
                let mut fs = CURRENT_FULLSCREEN_SHORTCUT.lock().unwrap();
                *fs = "ctrl+shift+s".to_string();
            }
            {
                let mut rg = CURRENT_REGION_SHORTCUT.lock().unwrap();
                *rg = "ctrl+shift+a".to_string();
            }
            {
                let mut ocr = CURRENT_OCR_SHORTCUT.lock().unwrap();
                *ocr = "ctrl+shift+d".to_string();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_full_screen,
            start_region_scan,
            scan_region_coords,
            show_window,
            hide_window,
            minimize_window,
            start_region_selection,
            complete_region_selection,
            cancel_region_selection,
            get_screenshot_data,
            set_close_behavior,
            get_close_behavior,
            update_shortcuts,
            start_ocr_region_selection,
            complete_ocr_region_selection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

