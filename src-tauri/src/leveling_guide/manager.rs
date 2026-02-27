use crate::error::{command_error, CommandError};
use crate::persistence::settings::LevelingGuideSettings;
use crate::persistence::store;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

use super::io::read_guide_content;
use super::parser::{clamp_position, current_page_dto, next_position, parse_guide_json, previous_position};
use super::types::{GuidePosition, LevelingGuidePageDto, LoadedGuide, PersistedGuidePosition, PersistedLevelingGuideProgress};

#[derive(Debug, Clone, serde::Deserialize)]
struct AreaEntry {
    id: String,
    name: String,
}

fn load_hint_index(
    app: &AppHandle,
    guide_path: &str,
) -> Result<(Vec<String>, HashMap<String, PathBuf>), CommandError> {
    let guide_relative_path = guide_path.strip_prefix("resource:").ok_or_else(|| {
        command_error(
            "hints_path_invalid",
            "Hints are only supported for resource: guides",
        )
    })?;

    let guide_dir = Path::new(guide_relative_path).parent().ok_or_else(|| {
        command_error("hints_path_invalid", "Failed to resolve guide directory")
    })?;

    let hints_relative_dir = guide_dir.join("img").join("hints");

    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e: tauri::Error| command_error("resource_dir_failed", e.to_string()))?;

    let hints_absolute_dir = resource_dir.join(&hints_relative_dir);
    let entries = match std::fs::read_dir(&hints_absolute_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Vec::new(), HashMap::new()));
        }
        Err(err) => {
            return Err(command_error(
                "hints_dir_read_failed",
                format!("{}: {}", hints_absolute_dir.display(), err),
            ));
        }
    };

    let mut hint_image_path_by_key: HashMap<String, PathBuf> = HashMap::new();
    for entry in entries {
        let entry = entry.map_err(|e| command_error("hints_dir_read_failed", e.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| command_error("hints_dir_read_failed", e.to_string()))?;
        if !file_type.is_file() {
            continue;
        }

        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        let key = stem.to_ascii_lowercase();
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        hint_image_path_by_key.insert(key, hints_relative_dir.join(file_name));
    }

    let mut hint_keys = hint_image_path_by_key
        .keys()
        .cloned()
        .collect::<Vec<String>>();
    hint_keys.sort_by(|left, right| {
        let left_len = left.replace('_', " ").len();
        let right_len = right.replace('_', " ").len();
        right_len.cmp(&left_len).then_with(|| left.cmp(right))
    });

    Ok((hint_keys, hint_image_path_by_key))
}

fn resolve_areas_path(guide_path: &str) -> Option<String> {
    let relative_resource_path = guide_path.strip_prefix("resource:");
    let base_path = relative_resource_path.unwrap_or(guide_path);

    let parent = Path::new(base_path).parent()?;
    let areas_path = parent.join("areas.json");

    if relative_resource_path.is_some() {
        return Some(format!("resource:{}", areas_path.to_string_lossy()));
    }

    Some(areas_path.to_string_lossy().to_string())
}

fn load_area_name_by_id(app: &AppHandle, guide_path: &str) -> Result<HashMap<String, String>, CommandError> {
    let areas_path = resolve_areas_path(guide_path)
        .ok_or_else(|| command_error("areas_path_invalid", "Failed to resolve areas.json path"))?;

    let content = read_guide_content(app, &areas_path)?;

    let parsed: Vec<Vec<AreaEntry>> = serde_json::from_str(&content)
        .map_err(|e| command_error("areas_parse_failed", e.to_string()))?;

    let mut lookup: HashMap<String, String> = HashMap::new();
    for act in parsed {
        for area in act {
            lookup.entry(area.id).or_insert(area.name);
        }
    }

    Ok(lookup)
}

#[derive(Default)]
pub struct LevelingGuideManager {
    loaded: Mutex<Option<LoadedGuide>>,
}

impl LevelingGuideManager {
    fn load_settings(app: &AppHandle) -> Result<LevelingGuideSettings, CommandError> {
        let maybe_settings = store::get_optional::<LevelingGuideSettings>(
            app,
            LevelingGuideSettings::STORE_KEY,
        )?;
        Ok(maybe_settings.unwrap_or_default())
    }

    pub fn is_loaded(&self) -> Result<bool, CommandError> {
        let guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;
        Ok(guard.is_some())
    }

    pub fn load(
        &self,
        app: &AppHandle,
        progress: PersistedLevelingGuideProgress,
    ) -> Result<LevelingGuidePageDto, CommandError> {
        let content = read_guide_content(app, &progress.guide_path)?;

        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| command_error("guide_parse_failed", e.to_string()))?;

        let guide = parse_guide_json(value)?;
        let settings = Self::load_settings(app)?;
        let position = clamp_position(&guide, progress.position.to_runtime(), &settings);

        let area_name_by_id = load_area_name_by_id(app, &progress.guide_path)?;

        let (hint_keys, hint_image_path_by_key) = load_hint_index(app, &progress.guide_path)?;

        let loaded = LoadedGuide {
            guide_path: progress.guide_path,
            guide,
            position,
            icon_cache: HashMap::new(),
            area_name_by_id,
            hint_keys,
            hint_image_path_by_key,
            hint_image_cache: HashMap::new(),
        };

        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;
        *guard = Some(loaded);

        let current = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        current_page_dto(app, current, &settings)
    }

    pub fn get_current_page(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        let settings = Self::load_settings(app)?;
        current_page_dto(app, loaded, &settings)
    }

    pub fn get_current_progress(&self) -> Result<PersistedLevelingGuideProgress, CommandError> {
        let guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_ref()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        Ok(PersistedLevelingGuideProgress {
            guide_path: loaded.guide_path.clone(),
            position: PersistedGuidePosition::from(loaded.position),
        })
    }

    pub fn reset_progress(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        loaded.position = GuidePosition::start();
        let settings = Self::load_settings(app)?;
        current_page_dto(app, loaded, &settings)
    }

    pub fn next_page(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        let settings = Self::load_settings(app)?;
        loaded.position = next_position(&loaded.guide, loaded.position, &settings);
        current_page_dto(app, loaded, &settings)
    }

    pub fn previous_page(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        let settings = Self::load_settings(app)?;
        loaded.position = previous_position(&loaded.guide, loaded.position, &settings);
        current_page_dto(app, loaded, &settings)
    }
}
