use tauri::Manager;
use crate::error::{command_error, CommandError};
use crate::window::identifiers::{INPUT_MASK_WINDOW_LABEL, OVERLAY_DEFAULT_MARGIN_PX, OVERLAY_WINDOW_LABEL};
use crate::window::input_mask_window::ensure_input_mask_window;
use crate::window::overlay_window::ensure_overlay_window;

#[cfg(linux_bsd_target_os)]
use gtk_layer_shell::{Edge, KeyboardMode, LayerShell};

#[cfg(linux_bsd_target_os)]
use gtk::prelude::WidgetExt;

#[cfg(linux_bsd_target_os)]
use gtk::cairo::{RectangleInt, Region};

#[cfg(linux_bsd_target_os)]
use std::sync::mpsc;

#[tauri::command]
pub fn show_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    let overlay_window = ensure_overlay_window(&app)?;

    #[cfg(linux_bsd_target_os)]
    {
        let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
        let window_for_closure = overlay_window.clone();
        overlay_window
            .run_on_main_thread(move || {
                let result = (|| {
                    if gtk_layer_shell::is_supported() {
                        let gtk_window = window_for_closure
                            .gtk_window()
                            .map_err(|e| command_error("overlay_panel_window_gtk_window_failed", e.to_string()))?;
                        gtk_window.set_layer_shell_margin(Edge::Bottom, OVERLAY_DEFAULT_MARGIN_PX);
                        gtk_window.set_layer_shell_margin(Edge::Left, OVERLAY_DEFAULT_MARGIN_PX);
                    }
                    Ok(())
                })();
                let _ = sender.send(result);
            })
            .map_err(|e| command_error("overlay_panel_window_main_thread_failed", e.to_string()))?;

        receiver
            .recv()
            .map_err(|e| command_error("overlay_panel_window_main_thread_channel_failed", e.to_string()))??;
    }

    overlay_window
        .show()
        .map_err(|e| command_error("overlay_panel_window_show_failed", e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn hide_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW_LABEL) {
        window
            .hide()
            .map_err(|e| command_error("overlay_panel_window_hide_failed", e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_overlay_panel_size(
    app: tauri::AppHandle,
    width: u32,
    height: u32,
) -> Result<(), CommandError> {
    let window = ensure_overlay_window(&app)?;
    let width = width.max(1);
    let height = height.max(1);

    #[cfg(linux_bsd_target_os)]
    {
        let width_i32 = i32::try_from(width)
            .map_err(|e| command_error("overlay_panel_window_width_overflow", e.to_string()))?;
        let height_i32 = i32::try_from(height)
            .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

        let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
        let window_for_closure = window.clone();
        window
            .run_on_main_thread(move || {
                let result = (|| {
                    if gtk_layer_shell::is_supported() {
                        let gtk_window = window_for_closure
                            .gtk_window()
                            .map_err(|e| command_error("overlay_panel_window_gtk_window_failed", e.to_string()))?;
                        gtk_window.set_size_request(width_i32, height_i32);
                    }

                    window_for_closure
                        .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
                        .map_err(|e| command_error("overlay_panel_window_set_size_failed", e.to_string()))?;

                    Ok(())
                })();
                let _ = sender.send(result);
            })
            .map_err(|e| command_error("overlay_panel_window_main_thread_failed", e.to_string()))?;

        receiver
            .recv()
            .map_err(|e| command_error("overlay_panel_window_main_thread_channel_failed", e.to_string()))??;

    }

    #[cfg(not(linux_bsd_target_os))]
    {
        let monitor = window
            .current_monitor()
            .map_err(|e| command_error("overlay_panel_window_get_monitor_failed", e.to_string()))?
            .or_else(|| {
                window
                    .primary_monitor()
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| command_error("overlay_panel_window_get_monitor_failed", "no monitor available"))?;

        let monitor_size = monitor.size();
        let monitor_position = monitor.position();
        let margin = i32::try_from(OVERLAY_DEFAULT_MARGIN_PX)
            .map_err(|e| command_error("overlay_panel_window_margin_overflow", e.to_string()))?;
        let monitor_height = i32::try_from(monitor_size.height)
            .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;
        let new_height = i32::try_from(height)
            .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;
        let y_in_monitor = (monitor_height - margin - new_height).max(0);
        let x = monitor_position.x + margin;
        let y = monitor_position.y + y_in_monitor;

        window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x,
                y,
            }))
            .map_err(|e| command_error("overlay_panel_window_set_position_failed", e.to_string()))?;

        window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
            .map_err(|e| command_error("overlay_panel_window_set_size_failed", e.to_string()))?;
    }

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
    let window = ensure_input_mask_window(&app)?;

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
        .get_webview_window(INPUT_MASK_WINDOW_LABEL)
        .ok_or_else(|| command_error("overlay_window_not_found", "input_mask window not created"))?;
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn set_overlay_interactive(app: tauri::AppHandle, interactive: bool) -> Result<(), CommandError> {
    let window = ensure_overlay_window(&app)?;

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
