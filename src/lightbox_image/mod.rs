pub mod settings;

use crate::lightbox_image::settings::CropSettings;
use photon_rs::{PhotonImage, transform};
use std::io::Cursor;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

/// 解码时允许的最大边长，用于挡住解压炸弹。
const MAX_IMAGE_DIM: u32 = 16384;
/// 解码时允许的最大分配字节数。
const MAX_ALLOC_BYTES: u64 = 512 * 1024 * 1024;

pub fn open_image(data: Vec<u8>) -> Result<PhotonImage, String> {
    // 用 Reader + Limits 显式限制尺寸与内存，而不是 photon 内部会 unwrap 的
    // `new_from_byteslice`。这里全程返回 Result，不会 panic。
    let mut limits = image::io::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIM);
    limits.max_image_height = Some(MAX_IMAGE_DIM);
    limits.max_alloc = Some(MAX_ALLOC_BYTES);

    let mut reader = image::io::Reader::new(Cursor::new(&data[..]));
    reader.limits(limits);

    let img = reader
        .with_guessed_format()
        .map_err(|e| format!("读取失败: {}", e))?
        .decode()
        .map_err(|e| format!("解码失败: {}", e))?;

    let width = img.width();
    let height = img.height();
    if width < 16 || height < 16 {
        return Err("尺寸太小: 要求宽度和高度至少 16 像素。".to_string());
    }

    let rgba = img.into_rgba8();
    Ok(PhotonImage::new(rgba.into_raw(), width, height))
}

pub fn crop_image(img: &PhotonImage, sett: &CropSettings) -> PhotonImage {
    let width = img.get_width() as f64;
    let height = img.get_height() as f64;

    let target_width = 1200.0;
    let target_height = 900.0;
    let target_ratio = target_width / target_height; // 4:3

    let scale = (sett.scale as f64).max(1.0);
    let offset_x = (sett.offset_x as f64).clamp(-1.0, 1.0);
    let offset_y = (sett.offset_y as f64).clamp(-1.0, 1.0);

    let img_ratio = width / height;
    let (base_crop_w, base_crop_h) = if img_ratio > target_ratio {
        // 原图比 4:3 更宽，裁剪宽度方向
        (height * target_ratio, height)
    } else {
        // 原图比 4:3 更窄，裁剪高度方向
        (width, width / target_ratio)
    };

    let crop_w = base_crop_w / scale;
    let crop_h = base_crop_h / scale;

    // 剩余可平移空间
    let slack_x = width - crop_w;
    let slack_y = height - crop_h;

    // offset -1.0: 最左/最顶
    // offset 0.0: 居中
    // offset 1.0: 最右/最底
    let x1 = (slack_x / 2.0) * (1.0 + offset_x);
    let y1 = (slack_y / 2.0) * (1.0 + offset_y);
    let x2 = x1 + crop_w;
    let y2 = y1 + crop_h;

    let w = img.get_width();
    let h = img.get_height();

    let x1_u32 = (x1.round() as u32).min(w.saturating_sub(1));
    let y1_u32 = (y1.round() as u32).min(h.saturating_sub(1));
    let x2_u32 = (x2.round() as u32).clamp(x1_u32 + 1, w);
    let y2_u32 = (y2.round() as u32).clamp(y1_u32 + 1, h);

    let cropped_img = transform::crop(img, x1_u32, y1_u32, x2_u32, y2_u32);
    transform::resize(
        &cropped_img,
        target_width as u32,
        target_height as u32,
        transform::SamplingFilter::Lanczos3,
    )
}

pub fn preview_image(
    img: PhotonImage,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
) -> Result<(), String> {
    // 直接用 web-sys 构建 ImageData（返回 Result），避免 photon `putImageData`
    // 内部对 `ImageData::new_..` 的 unwrap/expect。
    let width = img.get_width();
    let height = img.get_height();
    let mut pixels = img.get_raw_pixels();
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut pixels), width, height)
        .map_err(|e| format!("ImageData 创建失败: {e:?}"))?;

    canvas.set_width(width);
    canvas.set_height(height);
    ctx.put_image_data(&data, 0.0, 0.0)
        .map_err(|e| format!("putImageData() 失败: {e:?}"))
}

pub fn final_image(img: &PhotonImage) -> Result<Vec<u8>, String> {
    // 用 image crate 的 write_to（返回 Result）替换 photon `get_bytes_webp` 的 unwrap。
    let width = img.get_width();
    let height = img.get_height();
    let buffer = image::RgbaImage::from_raw(width, height, img.get_raw_pixels())
        .ok_or_else(|| "像素数据与尺寸不匹配".to_string())?;

    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(buffer)
        .write_to(&mut Cursor::new(&mut out), image::ImageOutputFormat::WebP)
        .map_err(|e| format!("写出 WebP 失败: {}", e))?;
    Ok(out)
}
