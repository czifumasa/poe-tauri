use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelingGuideSettings {
    pub league_start: bool,
    pub overlay_position: Option<OverlayPosition>,
}

impl Default for LevelingGuideSettings {
    fn default() -> Self {
        Self {
            league_start: true,
            overlay_position: None,
        }
    }
}

impl LevelingGuideSettings {
    pub const STORE_KEY: &'static str = "leveling_guide_settings";
}
