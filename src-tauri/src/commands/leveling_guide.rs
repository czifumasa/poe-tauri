use std::sync::Arc;

use crate::error::{command_error, CommandError};
use crate::leveling_guide::pob_parser::{self, PobImportData};
use crate::leveling_guide::progress::{has_persisted_leveling_guide_progress, load_leveling_guide_progress, save_leveling_guide_progress};
use crate::leveling_guide::{LevelingGuideManager, LevelingGuidePageDto};
use crate::persistence::settings::{PobSettings, PobSlot};
use crate::persistence::store;
use crate::timer::TimerManager;
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

fn ensure_loaded(
    app: &AppHandle,
    manager: &Arc<LevelingGuideManager>,
    timer_manager: &Arc<TimerManager>,
) -> Result<(), CommandError> {
    if manager.is_loaded()? {
        return Ok(());
    }

    let progress = load_leveling_guide_progress(app, DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH)?;
    let _ = manager.load(app, progress)?;
    start_log_watcher_if_configured(app, manager, timer_manager);
    Ok(())
}

fn persist_current_progress(
    app: &AppHandle,
    manager: &LevelingGuideManager,
) -> Result<(), CommandError> {
    let progress = manager.get_current_progress()?;
    save_leveling_guide_progress(app, &progress)
}

fn start_log_watcher_if_configured(
    app: &AppHandle,
    manager: &Arc<LevelingGuideManager>,
    timer_manager: &Arc<TimerManager>,
) {
    if let Err(err) = manager.restart_log_watcher_if_configured(app, timer_manager) {
        eprintln!("Failed to start log watcher: {:?}", err);
    }
}

#[tauri::command(async)]
pub fn load_guide(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    timer_manager: State<'_, Arc<TimerManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    if manager.is_loaded()? {
        let page = manager.get_current_page(&app)?;
        emit_page_updated(&app, &page)?;
        return Ok(page);
    }

    let progress = load_leveling_guide_progress(&app, DEFAULT_GUIDE_RELATIVE_RESOURCE_PATH)?;
    let page = manager.load(&app, progress)?;
    persist_current_progress(&app, &manager)?;
    start_log_watcher_if_configured(&app, &manager, &timer_manager);
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_has_persisted_progress(
    app: tauri::AppHandle,
) -> Result<bool, CommandError> {
    has_persisted_leveling_guide_progress(&app)
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
    timer_manager: State<'_, Arc<TimerManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager, &timer_manager)?;
    let page = manager.next_page(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_previous_page(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    timer_manager: State<'_, Arc<TimerManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager, &timer_manager)?;
    let page = manager.previous_page(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_reset_progress(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    timer_manager: State<'_, Arc<TimerManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager, &timer_manager)?;
    let page = manager.reset_progress(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}

#[tauri::command(async)]
pub fn leveling_guide_import_pob(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    timer_manager: State<'_, Arc<TimerManager>>,
    pob_code: String,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager, &timer_manager)?;
    let pob_data = pob_parser::parse_pob_export(&pob_code)?;
    let page = manager.import_pob(&app, pob_data.clone())?;

    let mut pob_settings =
        store::get_optional::<PobSettings>(&app, PobSettings::STORE_KEY)?.unwrap_or_default();
    let slot = PobSlot {
        pob_code,
        class: pob_data.class,
        ascend_class: pob_data.ascend_class,
        gem_count: pob_data.gem_names.len(),
    };
    pob_settings.slots.push(slot);
    pob_settings.current_slot_index = Some(pob_settings.slots.len() - 1);
    store::set_value(&app, PobSettings::STORE_KEY, &pob_settings)?;

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
    timer_manager: State<'_, Arc<TimerManager>>,
) -> Result<LevelingGuidePageDto, CommandError> {
    ensure_loaded(&app, &manager, &timer_manager)?;
    let page = manager.reapply_gems(&app)?;
    persist_current_progress(&app, &manager)?;
    emit_page_updated(&app, &page)?;
    Ok(page)
}
