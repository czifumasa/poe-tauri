use std::sync::Arc;

use tauri::State;

use crate::error::CommandError;
use crate::leveling_guide::LevelingGuideManager;
use crate::persistence::store;
use crate::persistence::settings::{BanditsChoice, LevelingGuideSettings};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelingGuideSettingsDto {
    pub league_start: bool,
    pub overlay_position: Option<crate::persistence::settings::OverlayPosition>,
    pub optional_quests: bool,
    pub level_recommendations: bool,
    pub bandits_choice: BanditsChoice,
    pub client_log_path: Option<String>,
}

fn leveling_guide_settings_to_dto(settings: LevelingGuideSettings) -> LevelingGuideSettingsDto {
    LevelingGuideSettingsDto {
        league_start: settings.league_start,
        overlay_position: settings.overlay_position,
        optional_quests: settings.optional_quests,
        level_recommendations: settings.level_recommendations,
        bandits_choice: settings.bandits_choice,
        client_log_path: settings.client_log_path,
    }
}

fn leveling_guide_settings_from_dto(dto: LevelingGuideSettingsDto) -> LevelingGuideSettings {
    LevelingGuideSettings {
        league_start: dto.league_start,
        overlay_position: dto.overlay_position,
        optional_quests: dto.optional_quests,
        level_recommendations: dto.level_recommendations,
        bandits_choice: dto.bandits_choice,
        client_log_path: dto.client_log_path,
    }
}

#[tauri::command]
pub fn settings_get_leveling_guide(app: tauri::AppHandle) -> Result<LevelingGuideSettingsDto, CommandError> {
    let maybe_settings = store::get_optional::<LevelingGuideSettings>(&app, LevelingGuideSettings::STORE_KEY)?;
    Ok(leveling_guide_settings_to_dto(maybe_settings.unwrap_or_default()))
}

#[tauri::command]
pub fn settings_set_leveling_guide(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    settings: LevelingGuideSettingsDto,
) -> Result<(), CommandError> {
    let persisted = leveling_guide_settings_from_dto(settings);
    store::set_value(&app, LevelingGuideSettings::STORE_KEY, &persisted)?;

    if let Err(err) = manager.restart_log_watcher_if_configured(&app) {
        eprintln!("Failed to restart log watcher after settings change: {:?}", err);
    }

    Ok(())
}

#[tauri::command]
pub fn settings_wipe(app: tauri::AppHandle) -> Result<(), CommandError> {
    store::wipe(&app)
}
