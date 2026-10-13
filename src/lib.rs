pub mod lightbox;
pub mod lightbox_image;
mod utils;

use futures::{StreamExt as _, channel::mpsc};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::utils::set_panic_hook;

// --- talc global_allocator

#[cfg(target_arch = "wasm32")]
use talc::{sync::TalcLock, wasm::*};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static TALC: TalcLock<spinning_top::RawSpinlock, WasmGrowAndClaim, WasmBinning> =
    TalcLock::new(WasmGrowAndClaim);

// --- lightbox worker funcs

#[wasm_bindgen]
pub fn lightbox_upload() {}

// --- lightbox image edit funcs

#[wasm_bindgen]
pub async fn lightbox_image_editor_session(
    canvas: HtmlCanvasElement,
    session: &mut LightboxEditorSession,
    data: Vec<u8>,
) -> Result<Option<Vec<u8>>, JsValue> {
    set_panic_hook(); // 虽然不应该 panic 但是有备无患

    use lightbox_image::{
        crop_image, final_image, open_image, preview_image,
        settings::{CropSettings, EditorCommand},
    };

    let mut rx = session
        .rx
        .take()
        .ok_or_else(|| JsValue::from_str("编辑会话启动失败，刷新页面试试"))?;

    let original_img =
        open_image(data).map_err(|e| JsValue::from_str(&format!("图像解析失败：\n{e}")))?;
    let mut current_settings = CropSettings::default();
    let mut current_img = crop_image(&original_img, &current_settings);

    let canvas_ctx = canvas
        .get_context("2d")?
        .ok_or(JsValue::from_str("Canvas 上下文获取失败"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    preview_image(current_img, canvas.clone(), canvas_ctx.clone())
        .map_err(|e| JsValue::from_str(&format!("图像渲染失败：\n{e}")))?;

    // event loop

    while let Some(cmd) = rx.next().await {
        match cmd {
            EditorCommand::UpdateSettings(new_settings) => {
                current_settings = new_settings;
                current_img = crop_image(&original_img, &current_settings);
                preview_image(current_img, canvas.clone(), canvas_ctx.clone())
                    .map_err(|e| JsValue::from_str(&format!("图像渲染失败：\n{e}")))?;
            }
            EditorCommand::Confirm => {
                current_img = crop_image(&original_img, &current_settings);
                let final_data = final_image(&current_img)
                    .map_err(|e| JsValue::from_str(&format!("图像保存失败：\n{e}")))?;
                return Ok(Some(final_data));
            }
            EditorCommand::Cancel => {
                return Ok(None);
            }
        }
    }

    // channel closed
    Ok(None)
}

#[wasm_bindgen]
pub struct LightboxEditorSession {
    tx: mpsc::UnboundedSender<lightbox_image::settings::EditorCommand>,
    rx: Option<mpsc::UnboundedReceiver<lightbox_image::settings::EditorCommand>>,
}

#[wasm_bindgen]
impl LightboxEditorSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self { tx, rx: Some(rx) }
    }

    pub fn update_settings(&self, val: JsValue) -> Result<(), JsValue> {
        use lightbox_image::settings::{CropSettings, EditorCommand};
        let settings: CropSettings = serde_wasm_bindgen::from_value(val)?;
        let _ = self
            .tx
            .unbounded_send(EditorCommand::UpdateSettings(settings));
        Ok(())
    }

    pub fn confirm(&self) {
        use lightbox_image::settings::EditorCommand;
        let _ = self.tx.unbounded_send(EditorCommand::Confirm);
    }

    pub fn cancel(&self) {
        use lightbox_image::settings::EditorCommand;
        let _ = self.tx.unbounded_send(EditorCommand::Cancel);
    }
}

impl Default for LightboxEditorSession {
    fn default() -> Self {
        Self::new()
    }
}
