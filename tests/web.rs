//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use sbchild_top_wasm::lightbox_image::settings::CropSettings;
use sbchild_top_wasm::{lightbox_image_editor_session, LightboxEditorSession};

wasm_bindgen_test_configure!(run_in_browser);

/// 预览/裁剪输出固定为 1200x900（见 lightbox_image::crop_image）。
const PREVIEW_W: u32 = 1200;
const PREVIEW_H: u32 = 900;

/// 生成一张渐变 RGBA PNG，作为会话输入，保证 open_image 可解码。
fn make_png(width: u32, height: u32) -> Vec<u8> {
    let mut raw = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            raw.push((x * 255 / width.max(1)) as u8);
            raw.push((y * 255 / height.max(1)) as u8);
            raw.push(128);
            raw.push(255);
        }
    }
    photon_rs::PhotonImage::new(raw, width, height).get_bytes()
}

/// 创建与预览尺寸一致的离屏 canvas 及 2D 上下文。
fn make_canvas() -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let document = web_sys::window()
        .expect("应存在 window")
        .document()
        .expect("应存在 document");
    let canvas = document
        .create_element("canvas")
        .expect("创建 canvas 失败")
        .dyn_into::<HtmlCanvasElement>()
        .expect("应能转换为 HtmlCanvasElement");
    canvas.set_width(PREVIEW_W);
    canvas.set_height(PREVIEW_H);
    let ctx = canvas
        .get_context("2d")
        .expect("get_context 调用失败")
        .expect("应返回 2d 上下文")
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("应能转换为 CanvasRenderingContext2d");
    (canvas, ctx)
}

/// 构造 update_settings 所需的 JsValue。
fn settings(scale: f32, offset_x: f32, offset_y: f32) -> JsValue {
    serde_wasm_bindgen::to_value(&CropSettings {
        scale,
        offset_x,
        offset_y,
    })
    .expect("序列化 CropSettings 失败")
}

/// 跑一遍编辑会话：先下发命令（会进缓冲），再驱动会话直到结束。
async fn run<F>(data: Vec<u8>, setup: F) -> Result<Option<Vec<u8>>, JsValue>
where
    F: FnOnce(&LightboxEditorSession),
{
    let (canvas, _ctx) = make_canvas();
    let mut session = LightboxEditorSession::new();
    setup(&session);
    lightbox_image_editor_session(canvas, &mut session, data).await
}

/// confirm 分支：返回 WEBP 数据（RIFF/WEBP 魔数）。
#[wasm_bindgen_test]
async fn confirm_returns_webp_data() {
    let out = run(make_png(1600, 1200), |s| s.confirm())
        .await
        .expect("会话不应失败")
        .expect("confirm 应返回数据");
    assert!(out.len() > 12, "webp 数据不应为空");
    assert_eq!(&out[0..4], b"RIFF", "应为 RIFF 容器");
    assert_eq!(&out[8..12], b"WEBP", "应为 WEBP 编码");
}

/// cancel 分支：返回 None，且预览已绘制到 canvas。
#[wasm_bindgen_test]
async fn cancel_returns_none_and_draws_preview() {
    let (canvas, ctx) = make_canvas();
    let mut session = LightboxEditorSession::new();
    session.cancel();

    let out = lightbox_image_editor_session(canvas, &mut session, make_png(1600, 1200))
        .await
        .expect("会话不应失败");
    assert!(out.is_none(), "cancel 应返回 None");

    let img = ctx
        .get_image_data(0.0, 0.0, PREVIEW_W as f64, PREVIEW_H as f64)
        .expect("读取 canvas 失败");
    let pixels = img.data();
    assert_eq!(pixels.len(), (PREVIEW_W * PREVIEW_H * 4) as usize);
    assert_eq!(pixels[3], 255, "预览左上角应不透明，说明已绘制");
}

/// 设置生效 + 多次更新以最后一次为准（渲染确定性）。
#[wasm_bindgen_test]
async fn settings_change_output_and_last_wins() {
    let data = make_png(1600, 1200);

    let base = run(data.clone(), |s| s.confirm())
        .await
        .expect("会话不应失败")
        .expect("confirm 应返回数据");

    let scaled = run(data.clone(), |s| {
        s.update_settings(settings(2.0, 0.0, 0.0)).unwrap();
        s.confirm();
    })
    .await
    .expect("会话不应失败")
    .expect("confirm 应返回数据");
    assert_ne!(base, scaled, "缩放设置应改变输出图像");

    let multi = run(data.clone(), |s| {
        s.update_settings(settings(1.5, 1.0, 1.0)).unwrap();
        s.update_settings(settings(3.0, -1.0, 1.0)).unwrap();
        s.update_settings(settings(2.0, 0.0, 0.0)).unwrap();
        s.confirm();
    })
    .await
    .expect("会话不应失败")
    .expect("confirm 应返回数据");
    assert_eq!(scaled, multi, "多次更新设置应以最后一次为准");
}

/// 过小图像：open_image 的尺寸校验分支（不依赖 catch_unwind）。
#[wasm_bindgen_test]
async fn too_small_image_returns_error() {
    let err = run(make_png(8, 8), |s| s.confirm())
        .await
        .expect_err("过小图像应报错");
    assert!(
        err.as_string().unwrap_or_default().contains("尺寸太小"),
        "错误信息应提示尺寸过小，实际：{err:?}"
    );
}

/// 无效数据：解码返回 Err，而不是 panic/trap。
#[wasm_bindgen_test]
async fn invalid_image_returns_error() {
    let err = run(vec![0u8; 128], |s| s.confirm())
        .await
        .expect_err("无效图像应报错");
    assert!(
        err.as_string().unwrap_or_default().contains("图像解析失败"),
        "错误信息应提示图像解析失败，实际：{err:?}"
    );
}

/// 会话只能使用一次：rx 被 take 后二次调用报错。
#[wasm_bindgen_test]
async fn session_cannot_be_reused() {
    let data = make_png(1600, 1200);
    let (canvas, _ctx) = make_canvas();
    let mut session = LightboxEditorSession::new();
    session.cancel();
    let first = lightbox_image_editor_session(canvas, &mut session, data.clone())
        .await
        .expect("首次会话不应失败");
    assert!(first.is_none());

    let (canvas2, _ctx2) = make_canvas();
    session.confirm();
    let err = lightbox_image_editor_session(canvas2, &mut session, data)
        .await
        .expect_err("重复使用会话应失败");
    assert!(
        err.as_string()
            .unwrap_or_default()
            .contains("编辑会话启动失败"),
        "错误信息应提示会话启动失败，实际：{err:?}"
    );
}

/// update_settings 反序列化失败分支：非法 JsValue 应报错。
#[wasm_bindgen_test]
async fn update_settings_rejects_malformed_value() {
    let session = LightboxEditorSession::new();
    session
        .update_settings(JsValue::from_str("not a settings object"))
        .expect_err("非法设置应报错");
}
