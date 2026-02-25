use tauri::Manager;
use crate::window::identifiers::{HINT_TOOLTIP_WINDOW_LABEL, MAIN_WINDOW_LABEL, OVERLAY_WINDOW_LABEL};

mod error;
mod persistence;
mod leveling_guide;
mod window;
mod commands;

use commands::common::*;
use commands::overlay::*;
use commands::leveling_guide::*;
use commands::settings::*;
use commands::hint_tooltip::*;
use leveling_guide::LevelingGuideManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(LevelingGuideManager::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            load_guide,
            leveling_guide_get_current_page,
            leveling_guide_next_page,
            leveling_guide_previous_page,
            leveling_guide_reset_progress,
            settings_get_leveling_guide,
            settings_set_leveling_guide,
            settings_wipe,
            show_overlay,
            hide_overlay,
            overlay_get_position,
            overlay_set_position,
            overlay_apply_position,
            set_overlay_panel_size,
            hint_tooltip_show,
            hint_tooltip_hide,
            hint_tooltip_get_last_content
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        if let Some(overlay) = app_handle.get_webview_window(OVERLAY_WINDOW_LABEL) {
                            let _ = overlay.close();
                        }
                        if let Some(tooltip) = app_handle.get_webview_window(HINT_TOOLTIP_WINDOW_LABEL) {
                            let _ = tooltip.close();
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

