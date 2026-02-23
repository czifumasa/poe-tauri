use tauri::Manager;
use crate::error::{command_error, CommandError};
use crate::persistence::store;
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

const OVERLAY_POSITION_STORE_KEY: &str = "overlay_position";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OverlayPosition {
    Absolute { x: i32, y: i32 },
    LayerShellMargins { left: i32, bottom: i32 },
}

fn get_overlay_monitor(window: &tauri::WebviewWindow) -> Result<tauri::Monitor, CommandError> {
    window
        .current_monitor()
        .map_err(|e| command_error("overlay_panel_window_get_monitor_failed", e.to_string()))?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| command_error("overlay_panel_window_get_monitor_failed", "no monitor available"))
}

fn margins_to_absolute(
    window: &tauri::WebviewWindow,
    left: i32,
    bottom: i32,
) -> Result<(i32, i32), CommandError> {
    let monitor = get_overlay_monitor(window)?;
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();

    let outer_size = window
        .outer_size()
        .map_err(|e| command_error("overlay_panel_window_get_size_failed", e.to_string()))?;

    let monitor_height = i32::try_from(monitor_size.height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;
    let window_height = i32::try_from(outer_size.height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

    let left = left.max(0);
    let bottom = bottom.max(0);

    let x = monitor_position.x + left;
    let y_in_monitor = (monitor_height - bottom - window_height).max(0);
    let y = monitor_position.y + y_in_monitor;
    Ok((x, y))
}

fn absolute_to_margins(
    window: &tauri::WebviewWindow,
    x: i32,
    y: i32,
) -> Result<(i32, i32), CommandError> {
    let monitor = get_overlay_monitor(window)?;
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();

    let outer_size = window
        .outer_size()
        .map_err(|e| command_error("overlay_panel_window_get_size_failed", e.to_string()))?;

    let monitor_height = i32::try_from(monitor_size.height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;
    let window_height = i32::try_from(outer_size.height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

    let left = (x - monitor_position.x).max(0);
    let bottom = (monitor_position.y + monitor_height - y - window_height).max(0);
    Ok((left, bottom))
}

fn get_saved_overlay_position(app: &tauri::AppHandle) -> Result<Option<OverlayPosition>, CommandError> {
    store::get_optional::<OverlayPosition>(app, OVERLAY_POSITION_STORE_KEY)
}

fn save_overlay_position(app: &tauri::AppHandle, position: &OverlayPosition) -> Result<(), CommandError> {
    store::set_value(app, OVERLAY_POSITION_STORE_KEY, position)
}

fn apply_overlay_position(app: &tauri::AppHandle, position: &OverlayPosition) -> Result<(), CommandError> {
    let window = ensure_overlay_window(app)?;

    #[cfg(linux_bsd_target_os)]
    {
        let is_layer_shell_supported = gtk_layer_shell::is_supported();
        if is_layer_shell_supported {
            let (left, bottom) = match position {
                OverlayPosition::LayerShellMargins { left, bottom } => (*left, *bottom),
                OverlayPosition::Absolute { x, y } => absolute_to_margins(&window, *x, *y)?,
            };

            let left = left.max(0);
            let bottom = bottom.max(0);

            let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
            let window_for_closure = window.clone();
            window
                .run_on_main_thread(move || {
                    let result = (|| {
                        let gtk_window = window_for_closure
                            .gtk_window()
                            .map_err(|e| {
                                command_error("overlay_panel_window_gtk_window_failed", e.to_string())
                            })?;
                        gtk_window.set_layer_shell_margin(Edge::Left, left);
                        gtk_window.set_layer_shell_margin(Edge::Bottom, bottom);
                        Ok(())
                    })();

                    let _ = sender.send(result);
                })
                .map_err(|e| command_error("overlay_panel_window_main_thread_failed", e.to_string()))?;

            receiver
                .recv()
                .map_err(|e| command_error("overlay_panel_window_main_thread_channel_failed", e.to_string()))??;

            return Ok(());
        }
    }

    let (x, y) = match position {
        OverlayPosition::Absolute { x, y } => (*x, *y),
        OverlayPosition::LayerShellMargins { left, bottom } => margins_to_absolute(&window, *left, *bottom)?,
    };

    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| command_error("overlay_panel_window_set_position_failed", e.to_string()))?;

    Ok(())
}

fn apply_saved_overlay_position_if_any(app: &tauri::AppHandle) -> Result<bool, CommandError> {
    let saved = get_saved_overlay_position(app)?;
    if let Some(position) = saved {
        apply_overlay_position(app, &position)?;
        return Ok(true);
    }
    Ok(false)
}

#[tauri::command]
pub fn show_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    let overlay_window = ensure_overlay_window(&app)?;

    let _ = apply_saved_overlay_position_if_any(&app)?;

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
pub fn overlay_get_position(app: tauri::AppHandle) -> Result<OverlayPosition, CommandError> {
    let window = ensure_overlay_window(&app)?;

    #[cfg(linux_bsd_target_os)]
    {
        if gtk_layer_shell::is_supported() {
            let saved = get_saved_overlay_position(&app)?;
            if let Some(position) = saved {
                let (left, bottom) = match position {
                    OverlayPosition::LayerShellMargins { left, bottom } => (left, bottom),
                    OverlayPosition::Absolute { x, y } => {
                        let (left, bottom) = absolute_to_margins(&window, x, y)?;
                        (left, bottom)
                    }
                };

                return Ok(OverlayPosition::LayerShellMargins { left, bottom });
            }

            return Ok(OverlayPosition::LayerShellMargins {
                left: OVERLAY_DEFAULT_MARGIN_PX,
                bottom: OVERLAY_DEFAULT_MARGIN_PX,
            });
        }
    }

    let position = window
        .outer_position()
        .map_err(|e| command_error("overlay_panel_window_get_position_failed", e.to_string()))?;
    Ok(OverlayPosition::Absolute {
        x: position.x,
        y: position.y,
    })
}

#[tauri::command]
pub fn overlay_set_position(app: tauri::AppHandle, position: OverlayPosition) -> Result<(), CommandError> {
    apply_overlay_position(&app, &position)?;
    save_overlay_position(&app, &position)?;
    Ok(())
}

#[tauri::command]
pub fn overlay_apply_position(app: tauri::AppHandle, position: OverlayPosition) -> Result<(), CommandError> {
    apply_overlay_position(&app, &position)?;
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
        let layer_shell_supported = gtk_layer_shell::is_supported();
        let width_i32 = i32::try_from(width)
            .map_err(|e| command_error("overlay_panel_window_width_overflow", e.to_string()))?;
        let height_i32 = i32::try_from(height)
            .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

        let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
        let window_for_closure = window.clone();
        window
            .run_on_main_thread(move || {
                let result = (|| {
                    if layer_shell_supported {
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

        if !layer_shell_supported {
            let has_saved_position = get_saved_overlay_position(&app)?.is_some();
            if has_saved_position {
                let _ = apply_saved_overlay_position_if_any(&app)?;
            } else {
                let monitor = get_overlay_monitor(&window)?;
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
                    .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
                    .map_err(|e| command_error("overlay_panel_window_set_position_failed", e.to_string()))?;
            }
        }

    }

    #[cfg(not(linux_bsd_target_os))]
    {
        let has_saved_position = get_saved_overlay_position(&app)?.is_some();

        if has_saved_position {
            let _ = apply_saved_overlay_position_if_any(&app)?;
        } else {
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
        }

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
            .set_ignore_cursor_events(false)
            .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    }

    if interactive {
        window
            .set_focus()
            .map_err(|e| command_error("overlay_window_focus_failed", e.to_string()))?;
    }

    Ok(())
}
