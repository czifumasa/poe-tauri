use tauri::Manager;
use crate::error::{command_error, CommandError};

#[tauri::command]
pub fn load_guide(app: tauri::AppHandle) -> Result<serde_json::Value, CommandError> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e: tauri::Error| command_error("resource_dir_failed", e.to_string()))?;
    
    let guide_path = resource_path.join("data/english/poe1/level_tracker/default_guide.json");
    
    let guide_content = std::fs::read_to_string(&guide_path)
        .map_err(|e| command_error("guide_read_failed", format!("Failed to read guide file: {}", e)))?;
    
    let guide_data: serde_json::Value = serde_json::from_str(&guide_content)
        .map_err(|e| command_error("guide_parse_failed", format!("Failed to parse guide JSON: {}", e)))?;
    
    Ok(guide_data)
}
