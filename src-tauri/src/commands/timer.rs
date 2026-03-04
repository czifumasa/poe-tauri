use std::sync::Arc;

use tauri::{Emitter, State};

use crate::error::{command_error, CommandError};
use crate::timer::{TimerManager, TimerSettingsDto, TimerStateDto};

const TIMER_STATE_UPDATED_EVENT: &str = "timer_state_updated";

fn emit_timer_state(app: &tauri::AppHandle, state: &TimerStateDto) -> Result<(), CommandError> {
    app.emit(TIMER_STATE_UPDATED_EVENT, state)
        .map_err(|e| command_error("timer_emit_failed", e.to_string()))
}

#[tauri::command(async)]
pub fn timer_get_settings(app: tauri::AppHandle) -> Result<TimerSettingsDto, CommandError> {
    TimerManager::get_settings(&app)
}

#[tauri::command(async)]
pub fn timer_set_settings(
    app: tauri::AppHandle,
    act_timer_enabled: bool,
    campaign_timer_enabled: bool,
) -> Result<TimerSettingsDto, CommandError> {
    TimerManager::set_settings(&app, act_timer_enabled, campaign_timer_enabled)
}

#[tauri::command(async)]
pub fn timer_get_state(
    manager: State<'_, Arc<TimerManager>>,
) -> Result<TimerStateDto, CommandError> {
    manager.get_state()
}

#[tauri::command(async)]
pub fn timer_load_state(
    app: tauri::AppHandle,
    manager: State<'_, Arc<TimerManager>>,
) -> Result<TimerStateDto, CommandError> {
    manager.load_state(&app)
}

#[tauri::command(async)]
pub fn timer_start(
    app: tauri::AppHandle,
    manager: State<'_, Arc<TimerManager>>,
) -> Result<TimerStateDto, CommandError> {
    let dto = manager.start(&app)?;
    emit_timer_state(&app, &dto)?;
    Ok(dto)
}

#[tauri::command(async)]
pub fn timer_pause(
    app: tauri::AppHandle,
    manager: State<'_, Arc<TimerManager>>,
) -> Result<TimerStateDto, CommandError> {
    let dto = manager.pause(&app)?;
    emit_timer_state(&app, &dto)?;
    Ok(dto)
}

#[tauri::command(async)]
pub fn timer_resume(
    app: tauri::AppHandle,
    manager: State<'_, Arc<TimerManager>>,
) -> Result<TimerStateDto, CommandError> {
    let dto = manager.resume(&app)?;
    emit_timer_state(&app, &dto)?;
    Ok(dto)
}

#[tauri::command(async)]
pub fn timer_reset(
    app: tauri::AppHandle,
    manager: State<'_, Arc<TimerManager>>,
) -> Result<TimerStateDto, CommandError> {
    let dto = manager.reset(&app)?;
    emit_timer_state(&app, &dto)?;
    Ok(dto)
}
