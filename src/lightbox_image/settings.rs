use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CropSettings {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for CropSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

pub enum EditorCommand {
    UpdateSettings(CropSettings),
    Confirm,
    Cancel,
}
