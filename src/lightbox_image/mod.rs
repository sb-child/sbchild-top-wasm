pub mod settings;

use crate::lightbox_image::settings::CropSettings;
use photon_rs::{transform, PhotonImage};
use std::panic;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub fn open_image(data: Vec<u8>) -> Result<PhotonImage, String> {
    panic::catch_unwind(|| PhotonImage::new_from_byteslice(data)).map_err(|e| format!("{e:?}"))
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

    let x1_u32 = (x1.round() as u32).min(img.get_width());
    let y1_u32 = (y1.round() as u32).min(img.get_height());
    let x2_u32 = (x2.round() as u32).min(img.get_width()).max(x1_u32 + 1);
    let y2_u32 = (y2.round() as u32).min(img.get_height()).max(y1_u32 + 1);

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
    panic::catch_unwind(|| photon_rs::putImageData(canvas, ctx, img)).map_err(|e| format!("{e:?}"))
}

pub fn final_image(img: &PhotonImage) -> Result<Vec<u8>, String> {
    panic::catch_unwind(|| img.get_bytes_webp()).map_err(|e| format!("{e:?}"))
}
