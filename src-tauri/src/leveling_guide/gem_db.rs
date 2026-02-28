use crate::error::{command_error, CommandError};
use serde::Deserialize;
use std::collections::HashMap;
use tauri::AppHandle;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct QuestMeta {
    #[allow(dead_code)]
    pub(crate) act: u32,
    #[allow(dead_code)]
    pub(crate) npc: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GemQuestAvailability {
    #[serde(default)]
    pub(crate) quest: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) vendor: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GemEntry {
    #[serde(default)]
    pub(crate) attribute: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) level: Option<u32>,
    #[serde(default)]
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) quests: HashMap<String, GemQuestAvailability>,
}

#[derive(Debug, Clone)]
pub(crate) struct GemDatabase {
    pub(crate) quests: HashMap<String, QuestMeta>,
    pub(crate) gems: HashMap<String, GemEntry>,
}

pub(crate) fn load_gem_database(
    app: &AppHandle,
    guide_path: &str,
) -> Result<GemDatabase, CommandError> {
    let gems_path = resolve_gems_path(guide_path)
        .ok_or_else(|| command_error("gems_path_invalid", "Failed to resolve gems.json path"))?;

    let content = super::io::read_guide_content(app, &gems_path)?;

    let raw: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| command_error("gems_parse_failed", e.to_string()))?;

    let obj = raw.as_object().ok_or_else(|| {
        command_error(
            "gems_parse_failed",
            "Expected gems.json root to be an object",
        )
    })?;

    let quests: HashMap<String, QuestMeta> = match obj.get("_quests") {
        Some(val) => serde_json::from_value(val.clone()).map_err(|e| {
            command_error("gems_parse_failed", format!("Failed to parse _quests: {e}"))
        })?,
        None => HashMap::new(),
    };

    let mut gems: HashMap<String, GemEntry> = HashMap::new();
    for (key, value) in obj {
        if key.starts_with('_') {
            continue;
        }
        let entry: GemEntry = serde_json::from_value(value.clone()).map_err(|e| {
            command_error(
                "gems_parse_failed",
                format!("Failed to parse gem '{key}': {e}"),
            )
        })?;
        gems.insert(key.clone(), entry);
    }

    Ok(GemDatabase { quests, gems })
}

fn resolve_gems_path(guide_path: &str) -> Option<String> {
    let relative_resource_path = guide_path.strip_prefix("resource:");
    let base_path = relative_resource_path.unwrap_or(guide_path);

    let parent = std::path::Path::new(base_path).parent()?;
    let gems_path = parent.join("gems.json");

    if relative_resource_path.is_some() {
        return Some(format!("resource:{}", gems_path.to_string_lossy()));
    }

    Some(gems_path.to_string_lossy().to_string())
}

const STAT_COLORS: [&str; 3] = ["D81C1C", "00BF40", "0077FF"];

pub(crate) fn attribute_color(attribute: Option<u32>) -> Option<String> {
    let attr = attribute?;
    if attr >= 1 && attr <= 3 {
        Some(STAT_COLORS[(attr - 1) as usize].to_string())
    } else {
        None
    }
}
