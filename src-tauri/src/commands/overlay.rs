use crate::error::{command_error, CommandError};
use crate::persistence::store;
use crate::window::identifiers::{OVERLAY_DEFAULT_MARGIN_PX, OVERLAY_WINDOW_LABEL};
use crate::window::overlay_window::{ensure_always_on_top, ensure_overlay_window};
use tauri::Manager;

const OVERLAY_POSITION_STORE_KEY: &str = "overlay_position";

const OVERLAY_PANEL_SIZE_STORE_KEY: &str = "overlay_panel_size";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct OverlayPanelSize {
    pub width: u32,
    pub height: u32,
}

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
        .ok_or_else(|| {
            command_error(
                "overlay_panel_window_get_monitor_failed",
                "no monitor available",
            )
        })
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

fn get_saved_overlay_position(
    app: &tauri::AppHandle,
) -> Result<Option<OverlayPosition>, CommandError> {
    store::get_optional::<OverlayPosition>(app, OVERLAY_POSITION_STORE_KEY)
}

fn save_overlay_position(
    app: &tauri::AppHandle,
    position: &OverlayPosition,
) -> Result<(), CommandError> {
    store::set_value(app, OVERLAY_POSITION_STORE_KEY, position)
}

fn apply_overlay_position(
    app: &tauri::AppHandle,
    position: &OverlayPosition,
) -> Result<(), CommandError> {
    let backend = crate::window::native_backend();
    let window = ensure_overlay_window(app)?;

    if backend.uses_layer_shell_margins() {
        let (left, bottom) = match position {
            OverlayPosition::LayerShellMargins { left, bottom } => (*left, *bottom),
            OverlayPosition::Absolute { x, y } => absolute_to_margins(&window, *x, *y)?,
        };

        return backend.set_layer_shell_margins(&window, left, bottom);
    }

    let (x, y) = match position {
        OverlayPosition::Absolute { x, y } => (*x, *y),
        OverlayPosition::LayerShellMargins { left, bottom } => {
            margins_to_absolute(&window, *left, *bottom)?
        }
    };

    backend.set_position(&window, x, y)
}

fn apply_saved_overlay_position_if_any(app: &tauri::AppHandle) -> Result<bool, CommandError> {
    let saved = get_saved_overlay_position(app)?;
    if let Some(position) = saved {
        apply_overlay_position(app, &position)?;
        return Ok(true);
    }
    Ok(false)
}

#[tauri::command(async)]
pub fn show_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    let overlay_window = ensure_overlay_window(&app)?;

    let _ = apply_saved_overlay_position_if_any(&app)?;

    overlay_window
        .show()
        .map_err(|e| command_error("overlay_panel_window_show_failed", e.to_string()))?;

    ensure_always_on_top(&overlay_window)?;

    Ok(())
}

#[tauri::command(async)]
pub fn hide_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW_LABEL) {
        window
            .hide()
            .map_err(|e| command_error("overlay_panel_window_hide_failed", e.to_string()))?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn overlay_is_visible(app: tauri::AppHandle) -> Result<bool, CommandError> {
    let Some(window) = app.get_webview_window(OVERLAY_WINDOW_LABEL) else {
        return Ok(false);
    };

    window
        .is_visible()
        .map_err(|e| command_error("overlay_panel_window_get_visibility_failed", e.to_string()))
}

#[tauri::command(async)]
pub fn overlay_get_position(app: tauri::AppHandle) -> Result<OverlayPosition, CommandError> {
    let backend = crate::window::native_backend();
    let window = ensure_overlay_window(&app)?;

    if backend.uses_layer_shell_margins() {
        let saved = get_saved_overlay_position(&app)?;
        if let Some(position) = saved {
            let (left, bottom) = match position {
                OverlayPosition::LayerShellMargins { left, bottom } => (left, bottom),
                OverlayPosition::Absolute { x, y } => absolute_to_margins(&window, x, y)?,
            };

            return Ok(OverlayPosition::LayerShellMargins { left, bottom });
        }

        return Ok(OverlayPosition::LayerShellMargins {
            left: OVERLAY_DEFAULT_MARGIN_PX,
            bottom: OVERLAY_DEFAULT_MARGIN_PX,
        });
    }

    let (x, y) = backend.get_position(&window)?;
    Ok(OverlayPosition::Absolute { x, y })
}

#[tauri::command(async)]
pub fn overlay_set_position(
    app: tauri::AppHandle,
    position: OverlayPosition,
) -> Result<(), CommandError> {
    apply_overlay_position(&app, &position)?;
    save_overlay_position(&app, &position)?;
    Ok(())
}

#[tauri::command(async)]
pub fn overlay_reset_to_default_position(app: tauri::AppHandle) -> Result<(), CommandError> {
    if app.get_webview_window(OVERLAY_WINDOW_LABEL).is_none() {
        return Ok(());
    }

    let default_position = OverlayPosition::LayerShellMargins {
        left: OVERLAY_DEFAULT_MARGIN_PX,
        bottom: OVERLAY_DEFAULT_MARGIN_PX,
    };
    apply_overlay_position(&app, &default_position)?;
    Ok(())
}

#[tauri::command(async)]
pub fn overlay_apply_position(
    app: tauri::AppHandle,
    position: OverlayPosition,
) -> Result<(), CommandError> {
    apply_overlay_position(&app, &position)?;
    Ok(())
}

fn compute_default_absolute_position(
    window: &tauri::WebviewWindow,
    new_height: i32,
) -> Result<(i32, i32), CommandError> {
    let monitor = get_overlay_monitor(window)?;
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();

    let margin = i32::try_from(OVERLAY_DEFAULT_MARGIN_PX)
        .map_err(|e| command_error("overlay_panel_window_margin_overflow", e.to_string()))?;
    let monitor_height = i32::try_from(monitor_size.height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

    let y_in_monitor = (monitor_height - margin - new_height).max(0);
    let x = monitor_position.x + margin;
    let y = monitor_position.y + y_in_monitor;
    Ok((x, y))
}

fn capture_old_position_state(
    window: &tauri::WebviewWindow,
    saved_pos: Option<&OverlayPosition>,
) -> Option<(i32, i32, i32)> {
    let old_height = window
        .inner_size()
        .ok()
        .and_then(|size| i32::try_from(size.height).ok());

    if let (Some(pos), Some(h)) = (saved_pos, old_height) {
        match pos {
            OverlayPosition::Absolute { x, y } => Some((*x, *y, h)),
            OverlayPosition::LayerShellMargins { left, bottom } => {
                margins_to_absolute(window, *left, *bottom)
                    .ok()
                    .map(|(x, y)| (x, y, h))
            }
        }
    } else {
        window.inner_size().ok().and_then(|size| {
            window.outer_position().ok().and_then(|pos| {
                let x = i32::try_from(pos.x).ok()?;
                let y = i32::try_from(pos.y).ok()?;
                let h = i32::try_from(size.height).ok()?;
                Some((x, y, h))
            })
        })
    }
}

#[tauri::command(async)]
pub fn set_overlay_panel_size(
    app: tauri::AppHandle,
    width: u32,
    height: u32,
) -> Result<(), CommandError> {
    let backend = crate::window::native_backend();
    let window = ensure_overlay_window(&app)?;
    let width = width.max(1);
    let height = height.max(1);

    let _old_panel_size =
        store::get_optional::<OverlayPanelSize>(&app, OVERLAY_PANEL_SIZE_STORE_KEY)?;

    store::set_value(
        &app,
        OVERLAY_PANEL_SIZE_STORE_KEY,
        &OverlayPanelSize { width, height },
    )?;

    let new_height_i32 = i32::try_from(height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

    if backend.uses_layer_shell_margins() {
        backend.set_size_with_gtk_refresh(&window, width, height)?;
        ensure_always_on_top(&window)?;
        return Ok(());
    }

    let saved_pos = get_saved_overlay_position(&app)?;
    let old_state = capture_old_position_state(&window, saved_pos.as_ref());

    backend.set_size_with_gtk_refresh(&window, width, height)?;

    if let Some(old_state) = old_state {
        let (old_x, old_y, old_height) = old_state;
        let height_delta = new_height_i32 - old_height;
        let new_y = old_y - height_delta;

        let position_to_apply = OverlayPosition::Absolute { x: old_x, y: new_y };
        apply_overlay_position(&app, &position_to_apply)?;
        save_overlay_position(&app, &position_to_apply)?;
    } else {
        let has_saved_position = saved_pos.is_some();
        if has_saved_position {
            let _ = apply_saved_overlay_position_if_any(&app)?;
        } else {
            let (x, y) = compute_default_absolute_position(&window, new_height_i32)?;
            backend.set_position(&window, x, y)?;
        }
    }

    ensure_always_on_top(&window)?;

    Ok(())
}
