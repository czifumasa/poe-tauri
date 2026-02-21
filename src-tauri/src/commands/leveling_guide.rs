use crate::error::{command_error, CommandError};
use crate::leveling_guide::{LevelingGuideManager, LevelingGuidePageDto};
use crate::leveling_guide::progress::{
    load_leveling_guide_progress, save_leveling_guide_progress,
};
use tauri::Emitter;
use tauri::AppHandle;
use tauri::State;

const DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH: &str =
    "data/english/poe1/level_tracker/default_guide.json";

const LEVELING_GUIDE_PAGE_UPDATED_EVENT: &str = "leveling_guide_page_updated";

fn emit_page_updated(app: &AppHandle, page: &LevelingGuidePageDto) -> Result<(), CommandError> {
    app.emit(LEVELING_GUIDE_PAGE_UPDATED_EVENT, page)
        .map_err(|e| command_error("leveling_guide_emit_failed", e.to_string()))
}

fn ensure_loaded(app: &AppHandle, manager: &LevelingGuideManager) -> Result<(), CommandError> {
    if manager.is_loaded()? {
        return Ok(());
    }

    let progress = load_leveling_guide_progress(app, DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH)?;
    let _ = manager.load(app, progress)?;
    Ok(())
}

fn persist_current_progress(app: &AppHandle, manager: &LevelingGuideManager) -> Result<(), CommandError> {
    let progress = manager.get_current_progress()?;
    save_leveling_guide_progress(app, &progress)
}

#[tauri::command]
pub fn load_guide(
    app: tauri::AppHandle,
    manager: State<'_, LevelingGuideManager>,
) -> Result<LevelingGuidePageDto, CommandError> {
    if manager.is_loaded()? {
        let page = manager.get_current_page()?;
        emit_page_updated(&app, &page)?;
        return Ok(page);
    }

    let progress = load_leveling_guide_progress(&app, DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH)?;
    let page = manager.load(&app, progress)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command]
pub fn leveling_guide_get_current_page(
    manager: State<'_, LevelingGuideManager>,
) -> Result<Option<LevelingGuidePageDto>, CommandError> {
    if !manager.is_loaded()? {
        return Ok(None);
    }

    let page = manager.get_current_page()?;
    Ok(Some(page))
}

#[tauri::command]
pub fn leveling_guide_next_page(
    app: tauri::AppHandle,
    manager: State<'_, LevelingGuideManager>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.next_page()?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command]
pub fn leveling_guide_previous_page(
    app: tauri::AppHandle,
    manager: State<'_, LevelingGuideManager>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.previous_page()?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command]
pub fn leveling_guide_reset_progress(
    app: tauri::AppHandle,
    manager: State<'_, LevelingGuideManager>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.reset_progress()?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}
