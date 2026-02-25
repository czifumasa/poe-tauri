use crate::error::CommandError;
use crate::persistence::store;
use tauri::AppHandle;

const LEAGUESTART_STORE_KEY: &str = "leaguestart";

fn ensure_leaguestart_initialized(app: &AppHandle) -> Result<bool, CommandError> {
    let existing = store::get_optional::<bool>(app, LEAGUESTART_STORE_KEY)?;
    if let Some(value) = existing {
        return Ok(value);
    }

    let default_value = true;
    store::set_value(app, LEAGUESTART_STORE_KEY, &default_value)?;

    Ok(default_value)
}

#[tauri::command]
pub fn settings_get_leaguestart(app: tauri::AppHandle) -> Result<bool, CommandError> {
    ensure_leaguestart_initialized(&app)
}

#[tauri::command]
pub fn settings_set_leaguestart(app: tauri::AppHandle, leaguestart: bool) -> Result<(), CommandError> {
    store::set_value(&app, LEAGUESTART_STORE_KEY, &leaguestart)
}

#[tauri::command]
pub fn settings_wipe(app: tauri::AppHandle) -> Result<(), CommandError> {
    store::wipe(&app)
}
