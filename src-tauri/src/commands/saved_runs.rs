use std::sync::Arc;

use tauri::{Emitter, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::{command_error, CommandError};
use crate::timer::{SavedRunDto, TimerManager, TimerStateDto};

const TIMER_STATE_UPDATED_EVENT: &str = "timer_state_updated";

#[tauri::command(async)]
pub fn saved_runs_save(
    app: tauri::AppHandle,
    manager: State<'_, Arc<TimerManager>>,
    league: String,
    hardcore: bool,
    ssf: bool,
    private_league: bool,
    character: String,
    character_class: String,
    run_details: String,
) -> Result<SavedRunDto, CommandError> {
    manager.save_run(
        &app,
        league,
        hardcore,
        ssf,
        private_league,
        character,
        character_class,
        run_details,
    )
}

#[tauri::command(async)]
pub fn saved_runs_load(app: tauri::AppHandle) -> Result<Vec<SavedRunDto>, CommandError> {
    TimerManager::load_runs(&app)
}

#[tauri::command(async)]
pub fn saved_runs_delete(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Vec<SavedRunDto>, CommandError> {
    TimerManager::delete_run(&app, run_id)
}

#[tauri::command(async)]
pub fn saved_runs_continue(
    app: tauri::AppHandle,
    manager: State<'_, Arc<TimerManager>>,
    run_id: String,
) -> Result<TimerStateDto, CommandError> {
    let dto = manager.continue_run(&app, run_id)?;
    app.emit(TIMER_STATE_UPDATED_EVENT, &dto)
        .map_err(|e| command_error("timer_emit_failed", e.to_string()))?;
    Ok(dto)
}

#[tauri::command(async)]
pub fn saved_runs_get(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<Option<SavedRunDto>, CommandError> {
    TimerManager::get_run(&app, run_id)
}

#[tauri::command(async)]
pub fn saved_runs_edit(
    app: tauri::AppHandle,
    run_id: String,
    league: String,
    hardcore: bool,
    ssf: bool,
    private_league: bool,
    character: String,
    character_class: String,
    run_details: String,
) -> Result<SavedRunDto, CommandError> {
    TimerManager::edit_run(
        &app,
        run_id,
        league,
        hardcore,
        ssf,
        private_league,
        character,
        character_class,
        run_details,
    )
}

#[tauri::command(async)]
pub fn saved_runs_export(app: tauri::AppHandle) -> Result<bool, CommandError> {
    let runs = TimerManager::load_runs(&app)?;
    let json = serde_json::to_string_pretty(&runs)
        .map_err(|e| command_error("export_serialize_failed", e.to_string()))?;

    let mut builder = app
        .dialog()
        .file()
        .set_file_name("poe-tauri-runs.json")
        .add_filter("JSON", &["json"]);

    if let Some(downloads) = dirs::download_dir() {
        builder = builder.set_directory(downloads);
    }

    let path = builder.blocking_save_file();

    let Some(file_path) = path else {
        return Ok(false);
    };

    std::fs::write(file_path.as_path().unwrap(), &json)
        .map_err(|e| command_error("export_write_failed", e.to_string()))?;

    Ok(true)
}
