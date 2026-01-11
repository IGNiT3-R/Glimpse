use image::{DynamicImage, GrayImage, ImageBuffer};
use rqrr::PreparedImage;
use serde::{Deserialize, Serialize};

/// 二维码识别结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeResult {
    /// 二维码内容
    pub content: String,
    /// 二维码类型（URL、文本等）
    pub qr_type: QrCodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QrCodeType {
    Url,
    Text,
    Email,
    Phone,
    Other,
}

impl QrCodeType {
    /// 根据内容推断二维码类型
    pub fn from_content(content: &str) -> Self {
        if content.starts_with("http://") || content.starts_with("https://") {
            QrCodeType::Url
        } else if content.starts_with("mailto:") {
            QrCodeType::Email
        } else if content.starts_with("tel:") {
            QrCodeType::Phone
        } else if content.contains("://") {
            QrCodeType::Other
        } else {
            QrCodeType::Text
        }
    }
}

/// 扫描图像中的所有二维码
///
/// # 参数
/// * `image` - 待扫描的图像
///
/// # 返回
/// 返回识别到的所有二维码结果
pub fn scan_qr_codes(image: &DynamicImage) -> Result<Vec<QrCodeResult>, String> {
    println!("[DEBUG] 开始扫描二维码，图像尺寸: {}x{}", image.width(), image.height());

    let mut all_results = Vec::new();

    // 优化：先尝试最常用的组合，减少尝试次数
    let strategies = vec![
        (1.0, PreprocessMethod::None),              // 1. 原图
        (1.0, PreprocessMethod::ContrastBoost),     // 2. 增强对比度
        (1.0, PreprocessMethod::Brightness),        // 3. 增加亮度
        (2.0, PreprocessMethod::None),              // 4. 放大2倍
        (1.0, PreprocessMethod::Sharpen),           // 5. 锐化
        (1.5, PreprocessMethod::ContrastBoost),     // 6. 放大1.5倍+对比度
        (0.5, PreprocessMethod::ContrastBoost),     // 7. 缩小0.5倍+对比度
        (3.0, PreprocessMethod::ContrastBoost),     // 8. 放大3倍+对比度
        (1.0, PreprocessMethod::Invert),            // 9. 反色（处理白色二维码）
    ];

    for (idx, (scale, method)) in strategies.iter().enumerate() {
        println!("[DEBUG] 尝试策略 {}/{}: 缩放={}, 预处理={:?}",
            idx + 1, strategies.len(), scale, method);

        let processed_image = if *scale != 1.0 {
            let new_width = (image.width() as f32 * scale) as u32;
            let new_height = (image.height() as f32 * scale) as u32;
            image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            image.clone()
        };

        let processed = apply_preprocessing(&processed_image, *method);
        let gray_image = processed.to_luma8();

        // 准备图像
        let mut prepared = prepare_image(&gray_image)?;

        // 识别所有二维码
        let grids = prepared.detect_grids();
        println!("[DEBUG]   检测到 {} 个网格", grids.len());

        // 如果没检测到网格，快速跳过
        if grids.is_empty() {
            continue;
        }

        for grid in grids {
            match grid.decode() {
                Ok((_meta, content)) => {
                    println!("[DEBUG]   ✓ 成功解码: {}", content);
                    // 避免重复结果
                    if !all_results.iter().any(|r: &QrCodeResult| r.content == content) {
                        all_results.push(QrCodeResult {
                            qr_type: QrCodeType::from_content(&content),
                            content,
                        });
                    }
                }
                Err(e) => {
                    println!("[DEBUG]   ✗ 解码失败: {:?}", e);
                    continue;
                }
            }
        }

        // 如果已经找到结果,立即返回
        if !all_results.is_empty() {
            println!("[DEBUG] ✓ 找到 {} 个二维码，停止尝试", all_results.len());
            return Ok(all_results);
        }
    }

    println!("[DEBUG] ✗ 扫描完成，共找到 {} 个二维码", all_results.len());
    Ok(all_results)
}

/// 图像预处理方法
#[derive(Debug, Clone, Copy)]
enum PreprocessMethod {
    None,              // 不处理
    ContrastBoost,     // 增强对比度
    Brightness,        // 增加亮度
    Sharpen,           // 锐化
    Invert,            // 反色
}

/// 应用图像预处理
fn apply_preprocessing(image: &DynamicImage, method: PreprocessMethod) -> DynamicImage {
    use image::imageops;
    match method {
        PreprocessMethod::None => image.clone(),
        PreprocessMethod::ContrastBoost => {
            // 增强对比度（从30增加到50，更强）
            image.adjust_contrast(50.0)
        }
        PreprocessMethod::Brightness => {
            // 增加亮度
            image.brighten(30)
        }
        PreprocessMethod::Sharpen => {
            // 锐化
            image.unsharpen(2.0, 5)
        }
        PreprocessMethod::Invert => {
            // 反色（处理白色二维码或深色背景）
            let mut inverted = image.clone();
            imageops::invert(&mut inverted);
            inverted
        }
    }
}

/// 准备图像用于二维码识别（包含图像增强）
fn prepare_image(gray_image: &GrayImage) -> Result<PreparedImage<ImageBuffer<image::Luma<u8>, Vec<u8>>>, String> {
    let prepared = PreparedImage::prepare(gray_image.clone());
    Ok(prepared)
}

/// 批量扫描多张图像（用于多屏幕场景）
pub fn scan_multiple_images(images: &[DynamicImage]) -> Result<Vec<QrCodeResult>, String> {
    let mut all_results = Vec::new();

    for (index, image) in images.iter().enumerate() {
        match scan_qr_codes(image) {
            Ok(mut results) => {
                // 如果是多屏幕，可以在这里添加屏幕索引信息
                all_results.append(&mut results);
            }
            Err(e) => {
                eprintln!("扫描第 {} 张图像时出错: {}", index + 1, e);
            }
        }
    }

    Ok(all_results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_type_detection() {
        assert!(matches!(
            QrCodeType::from_content("https://example.com"),
            QrCodeType::Url
        ));
        assert!(matches!(
            QrCodeType::from_content("mailto:test@example.com"),
            QrCodeType::Email
        ));
        assert!(matches!(
            QrCodeType::from_content("tel:+1234567890"),
            QrCodeType::Phone
        ));
        assert!(matches!(
            QrCodeType::from_content("这是普通文本"),
            QrCodeType::Text
        ));
    }
}
