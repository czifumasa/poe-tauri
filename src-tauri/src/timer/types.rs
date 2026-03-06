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
    pub run_id: Option<String>,
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
            run_id: None,
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
    pub run_id: Option<String>,
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
            run_id: state.run_id.clone(),
            current_act_index: state.current_act_index,
            act_elapsed_ms: state.act_elapsed_ms.clone(),
            current_act_elapsed_ms: state.current_act_elapsed_ms,
            campaign_elapsed_ms: state.campaign_elapsed_ms,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActRunStatus {
    Completed,
    InProgress,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActRun {
    pub act_name: String,
    pub elapsed_ms: u64,
    pub status: ActRunStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedRunStatus {
    Completed,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRun {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub league: String,
    pub hardcore: bool,
    pub ssf: bool,
    pub private_league: bool,
    pub character: String,
    pub character_class: String,
    pub run_details: String,
    pub status: SavedRunStatus,
    pub act_runs: Vec<ActRun>,
    pub campaign_elapsed_ms: u64,
    pub saved_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRunsData {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub runs: Vec<SavedRun>,
}

impl Default for SavedRunsData {
    fn default() -> Self {
        Self {
            schema_version: 1,
            runs: Vec::new(),
        }
    }
}

impl SavedRunsData {
    pub const STORE_KEY: &'static str = "saved_runs";
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActRunDto {
    pub act_name: String,
    pub elapsed_ms: u64,
    pub status: ActRunStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedRunDto {
    pub schema_version: u32,
    pub id: String,
    pub league: String,
    pub hardcore: bool,
    pub ssf: bool,
    pub private_league: bool,
    pub character: String,
    pub character_class: String,
    pub run_details: String,
    pub status: SavedRunStatus,
    pub act_runs: Vec<ActRunDto>,
    pub campaign_elapsed_ms: u64,
    pub saved_at: u64,
}

impl From<&SavedRun> for SavedRunDto {
    fn from(run: &SavedRun) -> Self {
        Self {
            schema_version: run.schema_version,
            id: run.id.clone(),
            league: run.league.clone(),
            hardcore: run.hardcore,
            ssf: run.ssf,
            private_league: run.private_league,
            character: run.character.clone(),
            character_class: run.character_class.clone(),
            run_details: run.run_details.clone(),
            status: run.status,
            act_runs: run
                .act_runs
                .iter()
                .map(|a| ActRunDto {
                    act_name: a.act_name.clone(),
                    elapsed_ms: a.elapsed_ms,
                    status: a.status,
                })
                .collect(),
            campaign_elapsed_ms: run.campaign_elapsed_ms,
            saved_at: run.saved_at,
        }
    }
}
