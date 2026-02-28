use crate::error::{command_error, CommandError};
use crate::persistence::settings::LevelingGuideSettings;
use crate::persistence::store;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri::Manager;

use super::gem_db::load_gem_database;
use super::gem_rewards::{inject_gem_rewards, prune_gem_quest_lines};
use super::io::read_guide_content;
use super::log_watcher::{spawn_log_watcher, LogWatcherHandle};
use super::parser::{clamp_position, current_page_dto, next_position, parse_guide_json, previous_position};
use super::pob_parser::PobImportData;
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

fn reapply_gem_rewards_to_guide(loaded: &mut LoadedGuide, settings: &LevelingGuideSettings) {
    loaded.guide = loaded.original_guide.clone();

    let should_inject = settings.gems_enabled;
    let has_pob = loaded.pob_import_data.is_some();
    let has_gem_db = loaded.gem_db.is_some();

    if should_inject && has_pob && has_gem_db {
        let gem_db = loaded.gem_db.as_ref().unwrap();
        let pob = loaded.pob_import_data.as_ref().unwrap();
        inject_gem_rewards(&mut loaded.guide, gem_db, pob, settings.league_start);
    } else if has_gem_db {
        let gem_db = loaded.gem_db.as_ref().unwrap();
        prune_gem_quest_lines(&mut loaded.guide, gem_db);
    }
}

pub struct LevelingGuideManager {
    loaded: Mutex<Option<LoadedGuide>>,
    watcher_handle: Mutex<Option<LogWatcherHandle>>,
}

impl Default for LevelingGuideManager {
    fn default() -> Self {
        Self {
            loaded: Mutex::new(None),
            watcher_handle: Mutex::new(None),
        }
    }
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

        let gem_db = load_gem_database(app, &progress.guide_path).ok();
        let original_guide = guide.clone();

        let loaded = LoadedGuide {
            guide_path: progress.guide_path,
            guide,
            original_guide,
            position,
            icon_cache: HashMap::new(),
            area_name_by_id,
            hint_keys,
            hint_image_path_by_key,
            hint_image_cache: HashMap::new(),
            target_area_id: None,
            gem_db,
            pob_import_data: None,
        };

        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;
        *guard = Some(loaded);

        let current = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        reapply_gem_rewards_to_guide(current, &settings);
        current.position = clamp_position(&current.guide, current.position, &settings);
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

    pub fn import_pob(&self, app: &AppHandle, pob_data: PobImportData) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        loaded.pob_import_data = Some(pob_data);

        let settings = Self::load_settings(app)?;
        reapply_gem_rewards_to_guide(loaded, &settings);
        loaded.position = clamp_position(&loaded.guide, loaded.position, &settings);
        current_page_dto(app, loaded, &settings)
    }

    pub fn reapply_gems(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        let settings = Self::load_settings(app)?;
        reapply_gem_rewards_to_guide(loaded, &settings);
        loaded.position = clamp_position(&loaded.guide, loaded.position, &settings);
        current_page_dto(app, loaded, &settings)
    }

    pub fn get_pob_import_status(&self) -> Result<Option<PobImportData>, CommandError> {
        let guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let data = guard
            .as_ref()
            .and_then(|loaded| loaded.pob_import_data.clone());

        Ok(data)
    }

    #[allow(dead_code)]
    pub fn get_target_area_id(&self) -> Result<Option<String>, CommandError> {
        let guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let target = guard
            .as_ref()
            .and_then(|loaded| loaded.target_area_id.clone());

        Ok(target)
    }

    pub fn try_auto_advance(
        &self,
        app: &AppHandle,
        area_id: &str,
    ) -> Result<Option<LevelingGuidePageDto>, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = match guard.as_mut() {
            Some(loaded) => loaded,
            None => return Ok(None),
        };

        let matches = loaded
            .target_area_id
            .as_deref()
            .is_some_and(|target| target == area_id);

        if !matches {
            return Ok(None);
        }

        let settings = Self::load_settings(app)?;
        loaded.position = next_position(&loaded.guide, loaded.position, &settings);
        let page = current_page_dto(app, loaded, &settings)?;
        Ok(Some(page))
    }

    pub fn start_log_watcher(
        self: &Arc<Self>,
        app: &AppHandle,
        log_path: &str,
    ) -> Result<(), CommandError> {
        self.stop_log_watcher();

        let handle = spawn_log_watcher(app, self, log_path)
            .map_err(|e| command_error("log_watcher_start_failed", e))?;

        let mut guard = self
            .watcher_handle
            .lock()
            .map_err(|_| command_error("watcher_state_poisoned", "Watcher state poisoned"))?;
        *guard = Some(handle);

        Ok(())
    }

    pub fn stop_log_watcher(&self) {
        if let Ok(mut guard) = self.watcher_handle.lock() {
            if let Some(handle) = guard.take() {
                handle.stop();
            }
        }
    }

    pub fn restart_log_watcher_if_configured(
        self: &Arc<Self>,
        app: &AppHandle,
    ) -> Result<(), CommandError> {
        let settings = Self::load_settings(app)?;
        let Some(log_path) = settings.client_log_path.as_deref() else {
            return Ok(());
        };

        if log_path.is_empty() {
            return Ok(());
        }

        if !self.is_loaded()? {
            return Ok(());
        }

        self.start_log_watcher(app, log_path)
    }
}
