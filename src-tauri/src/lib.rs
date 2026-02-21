use tauri::Manager;

mod error;
mod persistence;
mod leveling_guide;
mod window;
mod commands;

use commands::common::*;
use commands::overlay::*;
use commands::leveling_guide::*;
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
            leveling_guide_next_page,
            leveling_guide_previous_page,
            leveling_guide_reset_progress,
            show_overlay,
            hide_overlay,
            set_overlay_click_through,
            set_overlay_interactive,
            set_overlay_input_region,
            set_overlay_panel_size
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window("main") {
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        if let Some(overlay_panel) = app_handle.get_webview_window("overlay_panel") {
                            let _ = overlay_panel.close();
                        }
                        if let Some(overlay) = app_handle.get_webview_window("overlay") {
                            let _ = overlay.close();
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
