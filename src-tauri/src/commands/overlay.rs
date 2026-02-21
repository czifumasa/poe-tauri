use tauri::Manager;
use crate::error::{command_error, CommandError};
use crate::window::overlay_windows::{ensure_overlay_panel_window, ensure_overlay_window};

#[cfg(linux_bsd_target_os)]
use gtk_layer_shell::{KeyboardMode, LayerShell};

#[cfg(linux_bsd_target_os)]
use gtk::prelude::WidgetExt;

#[cfg(linux_bsd_target_os)]
use gtk::cairo::{RectangleInt, Region};

#[cfg(linux_bsd_target_os)]
use std::sync::mpsc;

#[tauri::command]
pub fn show_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    let panel_window = ensure_overlay_panel_window(&app)?;
    panel_window
        .show()
        .map_err(|e| command_error("overlay_panel_window_show_failed", e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn hide_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window("overlay_panel") {
        window
            .hide()
            .map_err(|e| command_error("overlay_panel_window_hide_failed", e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_overlay_panel_size(app: tauri::AppHandle, width: u32, height: u32) -> Result<(), CommandError> {
    let window = ensure_overlay_panel_window(&app)?;
    let width = width.max(1);
    let height = height.max(1);
    window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
        .map_err(|e| command_error("overlay_panel_window_set_size_failed", e.to_string()))?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct OverlayInputRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[tauri::command]
pub fn set_overlay_input_region(
    app: tauri::AppHandle,
    region: OverlayInputRegion,
) -> Result<(), CommandError> {
    let window = ensure_overlay_window(&app)?;

    #[cfg(linux_bsd_target_os)]
    {
        let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
        let window_for_closure = window.clone();

        window
            .run_on_main_thread(move || {
                let result = (|| {
                    let gtk_window = window_for_closure
                        .gtk_window()
                        .map_err(|e| command_error("overlay_window_gtk_window_failed", e.to_string()))?;

                    let width = region.width.max(0);
                    let height = region.height.max(0);
                    let rectangle = RectangleInt::new(region.x, region.y, width, height);
                    let cairo_region = Region::create_rectangle(&rectangle);

                    let empty_region = Region::create();
                    gtk_window.input_shape_combine_region(Some(&empty_region));
                    gtk_window.input_shape_combine_region(Some(&cairo_region));
                    Ok(())
                })();

                let _ = sender.send(result);
            })
            .map_err(|e| command_error("overlay_window_main_thread_failed", e.to_string()))?;

        receiver
            .recv()
            .map_err(|e| command_error("overlay_window_main_thread_channel_failed", e.to_string()))??;
    }

    Ok(())
}

#[tauri::command]
pub fn set_overlay_click_through(app: tauri::AppHandle, enabled: bool) -> Result<(), CommandError> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| command_error("overlay_window_not_found", "overlay window not created"))?;
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn set_overlay_interactive(app: tauri::AppHandle, interactive: bool) -> Result<(), CommandError> {
    let window = ensure_overlay_panel_window(&app)?;

    #[cfg(linux_bsd_target_os)]
    {
        let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
        let window_for_closure = window.clone();

        window
            .run_on_main_thread(move || {
                let result = (|| {
                    if gtk_layer_shell::is_supported() {
                        let gtk_window = window_for_closure
                            .gtk_window()
                            .map_err(|e| {
                                command_error("overlay_window_gtk_window_failed", e.to_string())
                            })?;

                        gtk_window.set_keyboard_mode(if interactive {
                            KeyboardMode::OnDemand
                        } else {
                            KeyboardMode::None
                        });
                    }

                    Ok(())
                })();

                let _ = sender.send(result);
            })
            .map_err(|e| command_error("overlay_window_main_thread_failed", e.to_string()))?;

        receiver
            .recv()
            .map_err(|e| command_error("overlay_window_main_thread_channel_failed", e.to_string()))??;
    }

    window
        .set_focusable(interactive)
        .map_err(|e| command_error("overlay_window_set_focusable_failed", e.to_string()))?;

    #[cfg(linux_bsd_target_os)]
    {
        window
            .set_ignore_cursor_events(false)
            .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    }

    #[cfg(not(linux_bsd_target_os))]
    {
        window
            .set_ignore_cursor_events(!interactive)
            .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    }

    if interactive {
        window
            .set_focus()
            .map_err(|e| command_error("overlay_window_focus_failed", e.to_string()))?;
    }

    Ok(())
}
