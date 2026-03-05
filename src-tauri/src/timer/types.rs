use serde::{Deserialize, Serialize};

const ACT_COUNT: usize = 10;

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
}

impl Default for TimerStatus {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTimerState {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub status: TimerStatus,
    #[serde(default)]
    pub current_act_index: usize,
    #[serde(default = "default_act_splits")]
    pub act_elapsed_ms: Vec<u64>,
    #[serde(default)]
    pub current_act_elapsed_ms: u64,
    #[serde(default)]
    pub campaign_elapsed_ms: u64,
}

fn default_act_splits() -> Vec<u64> {
    vec![0; ACT_COUNT]
}

impl Default for PersistedTimerState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            status: TimerStatus::Idle,
            current_act_index: 0,
            act_elapsed_ms: vec![0; ACT_COUNT],
            current_act_elapsed_ms: 0,
            campaign_elapsed_ms: 0,
        }
    }
}

impl PersistedTimerState {
    pub const STORE_KEY: &'static str = "timer_state";
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSettingsDto {
    pub schema_version: u32,
    pub enabled: bool,
    pub display_act_timer: bool,
    pub display_campaign_timer: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerStateDto {
    pub schema_version: u32,
    pub status: TimerStatus,
    pub current_act_index: usize,
    pub act_elapsed_ms: Vec<u64>,
    pub current_act_elapsed_ms: u64,
    pub campaign_elapsed_ms: u64,
}

impl From<&PersistedTimerState> for TimerStateDto {
    fn from(state: &PersistedTimerState) -> Self {
        Self {
            schema_version: state.schema_version,
            status: state.status,
            current_act_index: state.current_act_index,
            act_elapsed_ms: state.act_elapsed_ms.clone(),
            current_act_elapsed_ms: state.current_act_elapsed_ms,
            campaign_elapsed_ms: state.campaign_elapsed_ms,
        }
    }
}
