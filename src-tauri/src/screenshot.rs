use image::{DynamicImage, ImageBuffer, Rgba};
use screenshots::Screen;
use std::io::Cursor;

/// 捕获全屏截图
/// 返回包含所有屏幕的截图数据
pub fn capture_full_screen() -> Result<Vec<DynamicImage>, String> {
    let screens = Screen::all().map_err(|e| format!("获取屏幕列表失败: {}", e))?;

    let mut images = Vec::new();

    for screen in screens {
        let image = screen
            .capture()
            .map_err(|e| format!("捕获屏幕失败: {}", e))?;

        // 将 screenshots::Image 转换为 image::DynamicImage
        let width = image.width();
        let height = image.height();
        let rgba = image.rgba();

        let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, rgba.to_vec())
                .ok_or("图像缓冲区创建失败")?;

        images.push(DynamicImage::ImageRgba8(img_buffer));
    }

    Ok(images)
}

/// 捕获屏幕指定区域（自动处理 DPI 缩放和屏幕偏移）
///
/// # 参数
/// * `x` - 起始 X 坐标（屏幕绝对坐标）
/// * `y` - 起始 Y 坐标（屏幕绝对坐标）
/// * `width` - 区域宽度
/// * `height` - 区域高度
pub fn capture_region(x: u32, y: u32, width: u32, height: u32) -> Result<DynamicImage, String> {
    // 获取所有屏幕
    let screens = Screen::all().map_err(|e| format!("获取屏幕列表失败: {}", e))?;

    println!("[DEBUG] ======== 区域截图开始 ========");
    println!("[DEBUG] 请求坐标: x={}, y={}, w={}, h={}", x, y, width, height);
    println!("[DEBUG] 检测到 {} 个屏幕", screens.len());

    // 打印所有屏幕信息
    for (i, s) in screens.iter().enumerate() {
        let info = s.display_info;
        println!("[DEBUG] 屏幕 {}: 位置=({}, {}), 尺寸={}x{}, 缩放={}",
            i, info.x, info.y, info.width, info.height, info.scale_factor);
    }

    // 查找包含目标区域的屏幕
    let target_screen = screens.iter().find(|s| {
        let info = s.display_info;
        let sx = info.x as i32;
        let sy = info.y as i32;
        let sw = info.width as i32;
        let sh = info.height as i32;
        let tx = x as i32;
        let ty = y as i32;

        tx >= sx && tx < sx + sw && ty >= sy && ty < sy + sh
    }).or_else(|| screens.first())
      .ok_or("未找到可用屏幕")?;

    let display_info = target_screen.display_info;
    println!("[DEBUG] 使用屏幕: 位置=({}, {}), 尺寸={}x{}, 缩放={}",
        display_info.x, display_info.y, display_info.width, display_info.height, display_info.scale_factor);

    // 获取屏幕在虚拟桌面中的偏移
    let screen_offset_x = display_info.x as i32;
    let screen_offset_y = display_info.y as i32;

    // 将绝对坐标转换为相对于该屏幕的坐标
    let relative_x = (x as i32 - screen_offset_x).max(0) as u32;
    let relative_y = (y as i32 - screen_offset_y).max(0) as u32;

    println!("[DEBUG] 屏幕偏移: ({}, {})", screen_offset_x, screen_offset_y);
    println!("[DEBUG] 相对坐标: x={}, y={}", relative_x, relative_y);

    // 捕获该屏幕
    let full_image = target_screen
        .capture()
        .map_err(|e| format!("捕获屏幕失败: {}", e))?;

    // 转换为 DynamicImage
    let img_width = full_image.width();
    let img_height = full_image.height();
    let rgba = full_image.rgba();

    println!("[DEBUG] 截图物理分辨率: {}x{}", img_width, img_height);

    // 获取屏幕的逻辑分辨率信息
    let logical_width = display_info.width as f64;
    let logical_height = display_info.height as f64;

    println!("[DEBUG] 屏幕逻辑分辨率: {}x{}", logical_width, logical_height);

    // 计算 DPI 缩放比例（物理像素 / 逻辑像素）
    let scale_x = img_width as f64 / logical_width;
    let scale_y = img_height as f64 / logical_height;

    println!("[DEBUG] 计算的 DPI 缩放: x={:.3}, y={:.3}", scale_x, scale_y);

    // 将相对逻辑坐标转换为物理坐标
    let physical_x = (relative_x as f64 * scale_x) as u32;
    let physical_y = (relative_y as f64 * scale_y) as u32;
    let physical_width = (width as f64 * scale_x) as u32;
    let physical_height = (height as f64 * scale_y) as u32;

    println!("[DEBUG] 物理像素坐标: x={}, y={}, w={}, h={}", physical_x, physical_y, physical_width, physical_height);

    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(img_width, img_height, rgba.to_vec())
            .ok_or("图像缓冲区创建失败")?;

    let full_dynamic = DynamicImage::ImageRgba8(img_buffer);

    // 确保裁剪区域不超出图像边界
    let safe_x = physical_x.min(img_width.saturating_sub(1));
    let safe_y = physical_y.min(img_height.saturating_sub(1));
    let safe_width = physical_width.min(img_width.saturating_sub(safe_x));
    let safe_height = physical_height.min(img_height.saturating_sub(safe_y));

    println!("[DEBUG] 安全裁剪区域: x={}, y={}, w={}, h={}", safe_x, safe_y, safe_width, safe_height);

    // 裁剪指定区域（使用物理像素坐标）
    let cropped = full_dynamic.crop_imm(safe_x, safe_y, safe_width, safe_height);

    println!("[DEBUG] ======== 区域截图完成 ========");

    Ok(cropped)
}

/// 将 DynamicImage 转换为 PNG 字节数组（用于传递给前端）
pub fn image_to_png_bytes(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, image::ImageFormat::Png)
        .map_err(|e| format!("图像编码失败: {}", e))?;

    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_full_screen() {
        let result = capture_full_screen();
        assert!(result.is_ok(), "全屏截图应该成功");
        let images = result.unwrap();
        assert!(!images.is_empty(), "至少应该有一个屏幕");
    }

    #[test]
    fn test_capture_region() {
        let result = capture_region(0, 0, 100, 100);
        assert!(result.is_ok(), "区域截图应该成功");
        let image = result.unwrap();
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);
    }
}
