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
            gems_enabled: true,
        }
    }
}

impl LevelingGuideSettings {
    pub const STORE_KEY: &'static str = "leveling_guide_settings";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobSlot {
    pub pob_code: String,
    pub class: String,
    pub gem_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobSettings {
    #[serde(default)]
    pub slots: Vec<PobSlot>,
    #[serde(default)]
    pub current_slot_index: Option<usize>,
}

impl Default for PobSettings {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            current_slot_index: None,
        }
    }
}

impl PobSettings {
    pub const STORE_KEY: &'static str = "pob_settings";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub display_act_timer: bool,
    #[serde(default = "default_true")]
    pub display_campaign_timer: bool,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            display_act_timer: true,
            display_campaign_timer: true,
        }
    }
}

impl TimerSettings {
    pub const STORE_KEY: &'static str = "timer_settings";
}
