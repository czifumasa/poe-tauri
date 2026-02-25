use crate::error::CommandError;
use crate::persistence::store;
use crate::persistence::settings::LevelingGuideSettings;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelingGuideSettingsDto {
    pub league_start: bool,
    pub overlay_position: Option<crate::persistence::settings::OverlayPosition>,
}

fn leveling_guide_settings_to_dto(settings: LevelingGuideSettings) -> LevelingGuideSettingsDto {
    LevelingGuideSettingsDto {
        league_start: settings.league_start,
        overlay_position: settings.overlay_position,
    }
}

fn leveling_guide_settings_from_dto(dto: LevelingGuideSettingsDto) -> LevelingGuideSettings {
    LevelingGuideSettings {
        league_start: dto.league_start,
        overlay_position: dto.overlay_position,
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
    settings: LevelingGuideSettingsDto,
) -> Result<(), CommandError> {
    let persisted = leveling_guide_settings_from_dto(settings);
    store::set_value(&app, LevelingGuideSettings::STORE_KEY, &persisted)
}

#[tauri::command]
pub fn settings_wipe(app: tauri::AppHandle) -> Result<(), CommandError> {
    store::wipe(&app)
}
