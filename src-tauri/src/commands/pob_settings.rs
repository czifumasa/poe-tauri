use std::sync::Arc;

use tauri::State;

use crate::error::{command_error, CommandError};
use crate::leveling_guide::pob_parser;
use crate::leveling_guide::LevelingGuideManager;
use crate::persistence::settings::{PobSettings, PobSlot};
use crate::persistence::store;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PobSlotDto {
    pub pob_code: String,
    pub class: String,
    pub ascend_class: Option<String>,
    pub gem_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PobSettingsDto {
    pub slots: Vec<PobSlotDto>,
    pub current_slot_index: Option<usize>,
}

fn pob_slot_to_dto(slot: &PobSlot) -> PobSlotDto {
    PobSlotDto {
        pob_code: slot.pob_code.clone(),
        class: slot.class.clone(),
        ascend_class: slot.ascend_class.clone(),
        gem_count: slot.gem_count,
    }
}

fn pob_settings_to_dto(settings: &PobSettings) -> PobSettingsDto {
    PobSettingsDto {
        slots: settings.slots.iter().map(pob_slot_to_dto).collect(),
        current_slot_index: settings.current_slot_index,
    }
}

fn load_pob_settings(app: &tauri::AppHandle) -> Result<PobSettings, CommandError> {
    let maybe = store::get_optional::<PobSettings>(app, PobSettings::STORE_KEY)?;
    Ok(maybe.unwrap_or_default())
}

fn save_pob_settings(app: &tauri::AppHandle, settings: &PobSettings) -> Result<(), CommandError> {
    store::set_value(app, PobSettings::STORE_KEY, settings)
}

#[tauri::command(async)]
pub fn pob_settings_get(app: tauri::AppHandle) -> Result<PobSettingsDto, CommandError> {
    let settings = load_pob_settings(&app)?;
    Ok(pob_settings_to_dto(&settings))
}

#[tauri::command(async)]
pub fn pob_settings_add_slot(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    pob_code: String,
) -> Result<PobSettingsDto, CommandError> {
    let pob_data = pob_parser::parse_pob_export(&pob_code)?;

    let slot = PobSlot {
        pob_code: pob_code.clone(),
        class: pob_data.class.clone(),
        ascend_class: pob_data.ascend_class.clone(),
        gem_count: pob_data.gem_names.len(),
    };

    let mut settings = load_pob_settings(&app)?;
    settings.slots.push(slot);
    let new_index = settings.slots.len() - 1;
    settings.current_slot_index = Some(new_index);
    save_pob_settings(&app, &settings)?;

    if manager.is_loaded()? {
        let _ = manager.import_pob(&app, pob_data);
    }

    Ok(pob_settings_to_dto(&settings))
}

#[tauri::command(async)]
pub fn pob_settings_remove_slot(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    slot_index: usize,
) -> Result<PobSettingsDto, CommandError> {
    let mut settings = load_pob_settings(&app)?;

    if slot_index >= settings.slots.len() {
        return Err(command_error(
            "pob_slot_index_out_of_range",
            format!(
                "Slot index {} is out of range (total slots: {})",
                slot_index,
                settings.slots.len()
            ),
        ));
    }

    settings.slots.remove(slot_index);

    settings.current_slot_index = match settings.current_slot_index {
        Some(current) if current == slot_index => {
            if settings.slots.is_empty() {
                None
            } else if current >= settings.slots.len() {
                Some(settings.slots.len() - 1)
            } else {
                Some(current)
            }
        }
        Some(current) if current > slot_index => Some(current - 1),
        other => other,
    };

    save_pob_settings(&app, &settings)?;

    if manager.is_loaded()? {
        apply_current_slot_to_manager(&app, &manager, &settings)?;
    }

    Ok(pob_settings_to_dto(&settings))
}

#[tauri::command(async)]
pub fn pob_settings_set_current_slot(
    app: tauri::AppHandle,
    manager: State<'_, Arc<LevelingGuideManager>>,
    slot_index: usize,
) -> Result<PobSettingsDto, CommandError> {
    let mut settings = load_pob_settings(&app)?;

    if slot_index >= settings.slots.len() {
        return Err(command_error(
            "pob_slot_index_out_of_range",
            format!(
                "Slot index {} is out of range (total slots: {})",
                slot_index,
                settings.slots.len()
            ),
        ));
    }

    settings.current_slot_index = Some(slot_index);
    save_pob_settings(&app, &settings)?;

    if manager.is_loaded()? {
        apply_current_slot_to_manager(&app, &manager, &settings)?;
    }

    Ok(pob_settings_to_dto(&settings))
}

fn apply_current_slot_to_manager(
    app: &tauri::AppHandle,
    manager: &Arc<LevelingGuideManager>,
    settings: &PobSettings,
) -> Result<(), CommandError> {
    match settings.current_slot_index {
        Some(index) => {
            let slot = &settings.slots[index];
            let pob_data = pob_parser::parse_pob_export(&slot.pob_code)?;
            let _ = manager.import_pob(app, pob_data)?;
            Ok(())
        }
        None => Ok(()),
    }
}
