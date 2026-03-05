use crate::error::{command_error, CommandError};
use crate::logging;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = if cfg!(debug_assertions) {
    "settings.dev.json"
} else {
    "settings.json"
};

pub fn get_optional<T: DeserializeOwned>(
    app: &AppHandle,
    key: &str,
) -> Result<Option<T>, CommandError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| {
            logging::error("store", &format!("get_optional({key}): open failed: {e}"));
            command_error("store_open_failed", e.to_string())
        })?;

    let maybe_value = store.get(key);

    let parsed = maybe_value
        .map(|value| {
            serde_json::from_value::<T>(value)
                .map_err(|e| {
                    logging::error("store", &format!("get_optional({key}): deserialize failed: {e}"));
                    command_error("store_deserialize_failed", e.to_string())
                })
        })
        .transpose()?;

    Ok(parsed)
}

pub fn set_value<T: Serialize>(app: &AppHandle, key: &str, value: &T) -> Result<(), CommandError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| {
            logging::error("store", &format!("set_value({key}): open failed: {e}"));
            command_error("store_open_failed", e.to_string())
        })?;

    let json_value = serde_json::to_value(value)
        .map_err(|e| {
            logging::error("store", &format!("set_value({key}): serialize failed: {e}"));
            command_error("store_serialize_failed", e.to_string())
        })?;

    store.set(key, json_value);

    store
        .save()
        .map_err(|e| {
            logging::error("store", &format!("set_value({key}): save to disk failed: {e}"));
            command_error("store_save_failed", e.to_string())
        })?;

    Ok(())
}

pub fn wipe(app: &AppHandle) -> Result<(), CommandError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| command_error("store_open_failed", e.to_string()))?;

    store.clear();

    store
        .save()
        .map_err(|e| command_error("store_save_failed", e.to_string()))?;

    Ok(())
}
