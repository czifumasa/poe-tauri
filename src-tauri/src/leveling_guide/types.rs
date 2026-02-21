use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type GuideData = Vec<GuideAct>;
pub type GuideAct = Vec<GuidePage>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuidePosition {
    pub act_index: usize,
    pub page_index: usize,
}

impl GuidePosition {
    pub fn start() -> Self {
        Self {
            act_index: 0,
            page_index: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedLevelingGuideProgress {
    pub guide_path: String,
    pub position: GuidePosition,
}

impl PersistedLevelingGuideProgress {
    pub fn default_for_resource(relative_resource_path: &str) -> Self {
        Self {
            guide_path: format!("resource:{relative_resource_path}"),
            position: GuidePosition::start(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GuidePage {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LevelingGuideSpanDto {
    Text { text: String },
    Image {
        key: String,
        #[serde(rename = "dataUri")]
        data_uri: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelingGuideLineDto {
    pub is_hint: bool,
    pub spans: Vec<LevelingGuideSpanDto>,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedGuide {
    pub(crate) guide_path: String,
    pub(crate) guide: GuideData,
    pub(crate) position: GuidePosition,
    pub(crate) icon_cache: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelingGuidePageDto {
    pub guide_path: String,
    pub position: GuidePosition,
    pub act_count: usize,
    pub page_count_in_act: usize,
    pub lines: Vec<LevelingGuideLineDto>,
    pub has_previous: bool,
    pub has_next: bool,
}
