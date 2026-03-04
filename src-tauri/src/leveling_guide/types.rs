use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::gem_db::GemDatabase;
use super::pob_parser::PobImportData;

pub type GuideData = Vec<GuideAct>;
pub type GuideAct = Vec<GuidePage>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct PersistedGuidePosition {
    pub act_index: usize,
    pub page_index: usize,
}

impl PersistedGuidePosition {
    pub fn to_runtime(self) -> GuidePosition {
        GuidePosition {
            act_index: self.act_index,
            page_index: self.page_index,
        }
    }
}

impl From<GuidePosition> for PersistedGuidePosition {
    fn from(position: GuidePosition) -> Self {
        Self {
            act_index: position.act_index,
            page_index: position.page_index,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedLevelingGuideProgress {
    pub guide_path: String,
    pub position: PersistedGuidePosition,
}

impl PersistedLevelingGuideProgress {
    pub fn default_for_resource(relative_resource_path: &str) -> Self {
        Self {
            guide_path: format!("resource:{relative_resource_path}"),
            position: PersistedGuidePosition::from(GuidePosition::start()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GuidePage {
    pub lines: Vec<String>,
    pub condition: Option<GuideCondition>,
}

#[derive(Debug, Clone)]
pub enum GuideCondition {
    LeagueStart { enabled: bool },
    Bandit { allowed: Vec<String> },
    OptionalQuests { enabled: bool },
    LevelRecommendations { enabled: bool },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LevelingGuideSpanDto {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<LevelingGuideHintDto>,
    },
    Image {
        key: String,
        #[serde(rename = "dataUri")]
        data_uri: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelingGuideHintDto {
    pub key: String,
    #[serde(rename = "dataUri")]
    pub data_uri: String,
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
    pub(crate) original_guide: GuideData,
    pub(crate) position: GuidePosition,
    pub(crate) icon_cache: HashMap<String, Option<String>>,
    pub(crate) area_name_by_id: HashMap<String, String>,
    pub(crate) hint_keys: Vec<String>,
    pub(crate) hint_image_path_by_key: HashMap<String, PathBuf>,
    pub(crate) hint_image_cache: HashMap<String, Option<String>>,
    pub(crate) target_area_id: Option<String>,
    pub(crate) gem_db: Option<GemDatabase>,
    pub(crate) pob_import_data: Option<PobImportData>,
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
    pub campaign_page_index: usize,
    pub campaign_page_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_area: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_area_id: Option<String>,
}
