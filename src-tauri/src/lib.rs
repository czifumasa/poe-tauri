use crate::window::identifiers::{
    HINT_TOOLTIP_WINDOW_LABEL, MAIN_WINDOW_LABEL, OVERLAY_WINDOW_LABEL,
};
use std::sync::Arc;
use tauri::Manager;

mod commands;
mod error;
mod leveling_guide;
mod persistence;
mod window;

use commands::common::*;
use commands::hint_tooltip::*;
use commands::leveling_guide::*;
use commands::overlay::*;
use commands::settings::*;
use leveling_guide::LevelingGuideManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let manager = Arc::new(LevelingGuideManager::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::clone(&manager))
        .invoke_handler(tauri::generate_handler![
            greet,
            load_guide,
            leveling_guide_get_current_page,
            leveling_guide_next_page,
            leveling_guide_previous_page,
            leveling_guide_reset_progress,
            leveling_guide_import_pob,
            leveling_guide_get_pob_status,
            leveling_guide_reapply_gems,
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
        .setup(move |app| {
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let close_handle = app_handle.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        if let Some(overlay) = close_handle.get_webview_window(OVERLAY_WINDOW_LABEL)
                        {
                            let _ = overlay.close();
                        }
                        if let Some(tooltip) =
                            close_handle.get_webview_window(HINT_TOOLTIP_WINDOW_LABEL)
                        {
                            let _ = tooltip.close();
                        }
                    }
                });
            }

            if let Err(err) = manager.restart_log_watcher_if_configured(&app_handle) {
                eprintln!("Failed to start log watcher on startup: {:?}", err);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
