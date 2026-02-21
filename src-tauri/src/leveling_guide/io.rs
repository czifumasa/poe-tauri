use crate::error::{command_error, CommandError};
use tauri::AppHandle;
use tauri::Manager;

pub(crate) fn read_guide_content(app: &AppHandle, guide_path: &str) -> Result<String, CommandError> {
    if let Some(relative_resource_path) = guide_path.strip_prefix("resource:") {
        let resource_path = app
            .path()
            .resource_dir()
            .map_err(|e: tauri::Error| command_error("resource_dir_failed", e.to_string()))?;

        let absolute_path = resource_path.join(relative_resource_path);
        return std::fs::read_to_string(&absolute_path).map_err(|e| {
            command_error(
                "guide_read_failed",
                format!("Failed to read guide file: {e}"),
            )
        });
    }

    std::fs::read_to_string(guide_path).map_err(|e| {
        command_error(
            "guide_read_failed",
            format!("Failed to read guide file: {e}"),
        )
    })
}
