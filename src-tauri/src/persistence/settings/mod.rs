use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BanditsChoice {
    KillAll,
    HelpAlira,
    HelpOak,
    HelpKraityn,
}

impl Default for BanditsChoice {
    fn default() -> Self {
        Self::KillAll
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelingGuideSettings {
    #[serde(default = "default_true")]
    pub league_start: bool,
    #[serde(default)]
    pub overlay_position: Option<OverlayPosition>,
    #[serde(default = "default_true")]
    pub optional_quests: bool,
    #[serde(default = "default_true")]
    pub level_recommendations: bool,
    #[serde(default)]
    pub bandits_choice: BanditsChoice,
    #[serde(default)]
    pub client_log_path: Option<String>,
    #[serde(default)]
    pub gems_enabled: bool,
    #[serde(default)]
    pub pob_code: Option<String>,
}

impl Default for LevelingGuideSettings {
    fn default() -> Self {
        Self {
            league_start: true,
            overlay_position: None,
            optional_quests: true,
            level_recommendations: true,
            bandits_choice: BanditsChoice::KillAll,
            client_log_path: None,
            gems_enabled: false,
            pob_code: None,
        }
    }
}

impl LevelingGuideSettings {
    pub const STORE_KEY: &'static str = "leveling_guide_settings";
}
