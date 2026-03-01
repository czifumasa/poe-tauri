use std::sync::Arc;

use crate::error::{command_error, CommandError};
use crate::leveling_guide::pob_parser::{self, PobImportData};
use crate::leveling_guide::progress::{load_leveling_guide_progress, save_leveling_guide_progress};
use crate::leveling_guide::{LevelingGuideManager, LevelingGuidePageDto};
use crate::persistence::settings::LevelingGuideSettings;
use crate::persistence::store;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::State;

const DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH: &str =
    "data/english/poe1/level_tracker/default_guide.json";

const LEVELING_GUIDE_PAGE_UPDATED_EVENT: &str = "leveling_guide_page_updated";

fn emit_page_updated(app: &AppHandle, page: &LevelingGuidePageDto) -> Result<(), CommandError> {
    app.emit(LEVELING_GUIDE_PAGE_UPDATED_EVENT, page)
        .map_err(|e| command_error("leveling_guide_emit_failed", e.to_string()))
}

fn ensure_loaded(app: &AppHandle, manager: &Arc<LevelingGuideManager>) -> Result<(), CommandError> {
    if manager.is_loaded()? {
        return Ok(());
    }

    let progress = load_leveling_guide_progress(app, DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH)?;
    let _ = manager.load(app, progress)?;
    start_log_watcher_if_configured(app, manager);
    Ok(())
}

fn persist_current_progress(
    app: &AppHandle,
    manager: &LevelingGuideManager,
) -> Result<(), CommandError> {
    let progress = manager.get_current_progress()?;
    save_leveling_guide_progress(app, &progress)
}

fn start_log_watcher_if_configured(app: &AppHandle, manager: &Arc<LevelingGuideManager>) {
    if let Err(err) = manager.restart_log_watcher_if_configured(app) {
        eprintln!("Failed to start log watcher: {:?}", err);
    }
}

#[tauri::command(async)]
pub fn load_guide(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    if manager.is_loaded()? {
        let page = manager.get_current_page(&app)?;
        emit_page_updated(&app, &page)?;
        return Ok(page);
    }

    let progress = load_leveling_guide_progress(&app, DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH)?;
    let page = manager.load(&app, progress)?;
    start_log_watcher_if_configured(&app, &manager);
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_get_current_page(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
) -> Result<Option<LevelingGuidePageDto>, CommandError> {
    if !manager.is_loaded()? {
        return Ok(None);
    }

    let page = manager.get_current_page(&app)?;
    Ok(Some(page))
}

#[tauri::command(async)]
pub fn leveling_guide_next_page(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.next_page(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_previous_page(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.previous_page(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_reset_progress(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.reset_progress(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_import_pob(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    pob_code: String,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let pob_data = pob_parser::parse_pob_export(&pob_code)?;
    let page = manager.import_pob(&app, pob_data)?;

    let mut settings = store::get_optional::<LevelingGuideSettings>(
        &app,
        LevelingGuideSettings::STORE_KEY,
    )?
    .unwrap_or_default();
    settings.pob_code = Some(pob_code);
    store::set_value(&app, LevelingGuideSettings::STORE_KEY, &settings)?;

    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_get_pob_status(
    manager: State<'_, Arc<LevelingGuideManager>>,
) -> Result<Option<PobImportData>, CommandError> {
    manager.get_pob_import_status()
}

#[tauri::command(async)]
pub fn leveling_guide_reapply_gems(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager)?;
    let page = manager.reapply_gems(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}
