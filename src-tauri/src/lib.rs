use crate::window::identifiers::{
    HINT_TOOLTIP_WINDOW_LABEL, MAIN_WINDOW_LABEL, OVERLAY_WINDOW_LABEL,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;

mod commands;
mod error;
mod leveling_guide;
mod persistence;
mod timer;
mod window;

use commands::hint_tooltip::*;
use commands::leveling_guide::*;
use commands::overlay::*;
use commands::pob_settings::*;
use commands::settings::*;
use commands::timer::*;
use leveling_guide::LevelingGuideManager;
use timer::TimerManager;

fn lock_webview_window_inner_size(
    window: &tauri::WebviewWindow,
    target_inner_size: tauri::PhysicalSize<u32>,
) -> bool {
    let current_inner = match window.inner_size() {
        Ok(size) => size,
        Err(_) => return false,
    };
    let current_outer = match window.outer_size() {
        Ok(size) => size,
        Err(_) => return false,
    };

    let decoration_width = current_outer.width.saturating_sub(current_inner.width);
    let decoration_height = current_outer.height.saturating_sub(current_inner.height);

    let new_outer_width = target_inner_size.width.saturating_add(decoration_width);
    let new_outer_height = target_inner_size.height.saturating_add(decoration_height);

    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: new_outer_width,
        height: new_outer_height,
    }));

    let updated_inner = match window.inner_size() {
        Ok(size) => size,
        Err(_) => return false,
    };

    if updated_inner.width != target_inner_size.width
        || updated_inner.height != target_inner_size.height
    {
        return false;
    }

    let locked = tauri::Size::Physical(tauri::PhysicalSize {
        width: new_outer_width,
        height: new_outer_height,
    });
    let _ = window.set_min_size(Some(locked));
    let _ = window.set_max_size(Some(locked));
    let _ = window.set_resizable(false);

    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let manager = Arc::new(LevelingGuideManager::default());
    let timer_manager = Arc::new(TimerManager::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::clone(&manager))
        .manage(Arc::clone(&timer_manager))
        .invoke_handler(tauri::generate_handler![
            load_guide,
            leveling_guide_has_persisted_progress,
            leveling_guide_get_current_page,
            leveling_guide_next_page,
            leveling_guide_previous_page,
            leveling_guide_reset_progress,
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
            hint_tooltip_get_last_content,
            pob_settings_get,
            pob_settings_add_slot,
            pob_settings_remove_slot,
            pob_settings_set_current_slot,
            timer_get_settings,
            timer_set_settings,
            timer_load_state,
            timer_start,
            timer_pause,
            timer_resume,
            timer_reset,
            get_ascendancy_classes
        ])
        .setup(move |app| {
            window::init_native_window();

            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let did_adjust_main_window = Arc::new(AtomicBool::new(false));

                let main_window_for_startup = main_window.clone();
                let did_adjust_main_window_for_startup = Arc::clone(&did_adjust_main_window);
                std::thread::spawn(move || {
                    let target_inner_size = tauri::PhysicalSize {
                        width: 800,
                        height: 700,
                    };

                    for _ in 0..10 {
                        if did_adjust_main_window_for_startup.load(Ordering::Relaxed) {
                            return;
                        }

                        if lock_webview_window_inner_size(
                            &main_window_for_startup,
                            target_inner_size,
                        ) {
                            did_adjust_main_window_for_startup.store(true, Ordering::Relaxed);
                            return;
                        }

                        std::thread::sleep(Duration::from_millis(100));
                    }
                });

                let close_handle = app_handle.clone();
                let main_window_for_events = main_window.clone();
                let did_adjust_main_window_for_events = Arc::clone(&did_adjust_main_window);
                main_window.on_window_event(move |event| {
                    let should_adjust = matches!(
                        event,
                        tauri::WindowEvent::Resized(_)
                            | tauri::WindowEvent::ScaleFactorChanged { .. }
                    );

                    if should_adjust && !did_adjust_main_window_for_events.load(Ordering::Relaxed) {
                        let target_inner_size = tauri::PhysicalSize {
                            width: 800,
                            height: 700,
                        };
                        if lock_webview_window_inner_size(
                            &main_window_for_events,
                            target_inner_size,
                        ) {
                            did_adjust_main_window_for_events.store(true, Ordering::Relaxed);
                        }
                    }

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

            if let Err(err) = manager.restart_log_watcher_if_configured(&app_handle, &timer_manager)
            {
                eprintln!("Failed to start log watcher on startup: {:?}", err);
            }

            timer_manager.start_auto_save(&app_handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
