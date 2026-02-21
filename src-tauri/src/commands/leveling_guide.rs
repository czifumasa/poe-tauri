use crate::error::CommandError;
use crate::leveling_guide::{LevelingGuideManager, LevelingGuidePageDto};
use crate::leveling_guide::progress::{
    load_leveling_guide_progress, save_leveling_guide_progress,
};
use tauri::AppHandle;
use tauri::State;

const DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH: &str =
    "data/english/poe1/level_tracker/default_guide.json";

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
        return manager.get_current_page();
    }

    let progress = load_leveling_guide_progress(&app, DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH)?;
    manager.load(&app, progress)
}

#[tauri::command]
pub fn leveling_guide_next_page(
    app: tauri::AppHandle,
    manager: State<'_, LevelingGuideManager>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.next_page()?;
    persist_current_progress(&app, &manager)?;
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
    Ok(page)
}
