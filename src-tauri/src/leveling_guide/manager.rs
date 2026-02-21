use crate::error::{command_error, CommandError};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;

use super::io::read_guide_content;
use super::parser::{clamp_position, current_page_dto, parse_guide_json};
use super::types::{GuidePosition, LevelingGuidePageDto, LoadedGuide, PersistedLevelingGuideProgress};

#[derive(Default)]
pub struct LevelingGuideManager {
    loaded: Mutex<Option<LoadedGuide>>,
}

impl LevelingGuideManager {
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
        let position = clamp_position(&guide, progress.position);

        let loaded = LoadedGuide {
            guide_path: progress.guide_path,
            guide,
            position,
            icon_cache: HashMap::new(),
        };

        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;
        *guard = Some(loaded);

        let current = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        current_page_dto(app, current)
    }

    pub fn get_current_page(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        current_page_dto(app, loaded)
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
            position: loaded.position,
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
        current_page_dto(app, loaded)
    }

    pub fn next_page(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        let act = loaded.guide.get(loaded.position.act_index).ok_or_else(|| {
            command_error("guide_position_invalid", "Act index out of bounds")
        })?;

        if loaded.position.page_index + 1 < act.len() {
            loaded.position.page_index += 1;
            return current_page_dto(app, loaded);
        }

        if loaded.position.act_index + 1 < loaded.guide.len() {
            loaded.position.act_index += 1;
            loaded.position.page_index = 0;
            return current_page_dto(app, loaded);
        }

        current_page_dto(app, loaded)
    }

    pub fn previous_page(&self, app: &AppHandle) -> Result<LevelingGuidePageDto, CommandError> {
        let mut guard = self
            .loaded
            .lock()
            .map_err(|_| command_error("guide_state_poisoned", "Guide state poisoned"))?;

        let loaded = guard
            .as_mut()
            .ok_or_else(|| command_error("guide_not_loaded", "Guide not loaded"))?;

        if loaded.position.page_index > 0 {
            loaded.position.page_index -= 1;
            return current_page_dto(app, loaded);
        }

        if loaded.position.act_index == 0 {
            return current_page_dto(app, loaded);
        }

        loaded.position.act_index -= 1;
        let act = loaded.guide.get(loaded.position.act_index).ok_or_else(|| {
            command_error("guide_position_invalid", "Act index out of bounds")
        })?;

        loaded.position.page_index = act.len().saturating_sub(1);
        current_page_dto(app, loaded)
    }
}
