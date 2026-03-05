use std::sync::{Mutex, OnceLock};

use crate::error::{command_error, CommandError};
use crate::persistence::settings::LevelingGuideSettings;
use crate::persistence::store;
use crate::window::identifiers::{OVERLAY_DEFAULT_MARGIN_BOTTOM_PX, OVERLAY_DEFAULT_MARGIN_LEFT_PX, OVERLAY_WINDOW_LABEL};
use crate::window::overlay_window::{ensure_always_on_top, ensure_overlay_window};
use tauri::Manager;

const OVERLAY_POSITION_STORE_KEY: &str = "overlay_position";

const OVERLAY_PANEL_SIZE_STORE_KEY: &str = "overlay_panel_size";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct OverlayPanelSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OverlayPosition {
    Absolute { x: i32, y: i32 },
    LayerShellMargins { left: i32, bottom: i32 },
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

static OVERLAY_SCREEN_RECT: OnceLock<Mutex<Option<OverlayScreenRect>>> = OnceLock::new();

fn overlay_screen_rect_store() -> &'static Mutex<Option<OverlayScreenRect>> {
    OVERLAY_SCREEN_RECT.get_or_init(|| Mutex::new(None))
}

pub fn get_overlay_screen_rect() -> Option<OverlayScreenRect> {
    overlay_screen_rect_store()
        .lock()
        .ok()
        .and_then(|guard| *guard)
}

fn set_overlay_screen_rect(rect: OverlayScreenRect) {
    if let Ok(mut guard) = overlay_screen_rect_store().lock() {
        *guard = Some(rect);
    }
}

pub fn clear_overlay_screen_rect() {
    if let Ok(mut guard) = overlay_screen_rect_store().lock() {
        *guard = None;
    }
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

fn margins_to_absolute_with_height(
    window: &tauri::WebviewWindow,
    left: i32,
    bottom: i32,
    window_height: i32,
) -> Result<(i32, i32), CommandError> {
    let monitor = get_overlay_monitor(window)?;
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();

    let monitor_height = i32::try_from(monitor_size.height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

    let left = left.max(0);
    let bottom = bottom.max(0);

    let x = monitor_position.x + left;
    let y_in_monitor = (monitor_height - bottom - window_height).max(0);
    let y = monitor_position.y + y_in_monitor;
    Ok((x, y))
}

fn refresh_overlay_screen_rect(
    window: &tauri::WebviewWindow,
    position: &OverlayPosition,
    width: i32,
    height: i32,
) {
    let origin = match position {
        OverlayPosition::Absolute { x, y } => Some((*x, *y)),
        OverlayPosition::LayerShellMargins { left, bottom } => {
            margins_to_absolute_with_height(window, *left, *bottom, height).ok()
        }
    };
    let (x, y) = match origin {
        Some(pos) => pos,
        None => return,
    };
    set_overlay_screen_rect(OverlayScreenRect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    });
}

fn apply_overlay_position(
    app: &tauri::AppHandle,
    position: &OverlayPosition,
) -> Result<(), CommandError> {
    let native_window = crate::window::native_window();
    let window = ensure_overlay_window(app)?;

    if native_window.uses_layer_shell_margins() {
        let (left, bottom) = match position {
            OverlayPosition::LayerShellMargins { left, bottom } => (*left, *bottom),
            OverlayPosition::Absolute { x, y } => absolute_to_margins(&window, *x, *y)?,
        };

        native_window.set_layer_shell_margins(&window, left, bottom)?;
        let size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 0, height: 0 });
        let w = i32::try_from(size.width).unwrap_or(0);
        let h = i32::try_from(size.height).unwrap_or(0);
        refresh_overlay_screen_rect(&window, position, w, h);
        return Ok(());
    }

    let (x, y) = match position {
        OverlayPosition::Absolute { x, y } => (*x, *y),
        OverlayPosition::LayerShellMargins { left, bottom } => {
            margins_to_absolute(&window, *left, *bottom)?
        }
    };

    native_window.set_position(&window, x, y)?;
    let size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 0, height: 0 });
    let w = i32::try_from(size.width).unwrap_or(0);
    let h = i32::try_from(size.height).unwrap_or(0);
    refresh_overlay_screen_rect(&window, position, w, h);
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

fn persist_overlay_shown(app: &tauri::AppHandle, shown: bool) -> Result<(), CommandError> {
    let mut settings =
        store::get_optional::<LevelingGuideSettings>(app, LevelingGuideSettings::STORE_KEY)?
            .unwrap_or_default();
    settings.overlay_shown = shown;
    store::set_value(app, LevelingGuideSettings::STORE_KEY, &settings)
}

#[tauri::command(async)]
pub fn show_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    let overlay_window = ensure_overlay_window(&app)?;

    let applied = apply_saved_overlay_position_if_any(&app)?;
    if !applied {
        let default_pos = OverlayPosition::LayerShellMargins {
            left: OVERLAY_DEFAULT_MARGIN_LEFT_PX,
            bottom: OVERLAY_DEFAULT_MARGIN_BOTTOM_PX,
        };
        let size = overlay_window.outer_size().unwrap_or(tauri::PhysicalSize { width: 0, height: 0 });
        let w = i32::try_from(size.width).unwrap_or(0);
        let h = i32::try_from(size.height).unwrap_or(0);
        refresh_overlay_screen_rect(&overlay_window, &default_pos, w, h);
    }

    overlay_window
        .show()
        .map_err(|e| command_error("overlay_panel_window_show_failed", e.to_string()))?;

    ensure_always_on_top(&overlay_window)?;

    persist_overlay_shown(&app, true)?;

    Ok(())
}

#[tauri::command(async)]
pub fn hide_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW_LABEL) {
        window
            .hide()
            .map_err(|e| command_error("overlay_panel_window_hide_failed", e.to_string()))?;
    }
    persist_overlay_shown(&app, false)?;
    Ok(())
}

#[tauri::command(async)]
pub fn overlay_get_position(app: tauri::AppHandle) -> Result<OverlayPosition, CommandError> {
    let native_window = crate::window::native_window();
    let window = ensure_overlay_window(&app)?;

    if native_window.uses_layer_shell_margins() {
        let saved = get_saved_overlay_position(&app)?;
        if let Some(position) = saved {
            let (left, bottom) = match position {
                OverlayPosition::LayerShellMargins { left, bottom } => (left, bottom),
                OverlayPosition::Absolute { x, y } => absolute_to_margins(&window, x, y)?,
            };

            return Ok(OverlayPosition::LayerShellMargins { left, bottom });
        }

        return Ok(OverlayPosition::LayerShellMargins {
            left: OVERLAY_DEFAULT_MARGIN_LEFT_PX,
            bottom: OVERLAY_DEFAULT_MARGIN_BOTTOM_PX,
        });
    }

    let (x, y) = native_window.get_position(&window)?;
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

    let left_margin = i32::try_from(OVERLAY_DEFAULT_MARGIN_LEFT_PX)
        .map_err(|e| command_error("overlay_panel_window_margin_overflow", e.to_string()))?;
    let bottom_margin = i32::try_from(OVERLAY_DEFAULT_MARGIN_BOTTOM_PX)
        .map_err(|e| command_error("overlay_panel_window_margin_overflow", e.to_string()))?;

    let monitor_height = i32::try_from(monitor_size.height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

    let y_in_monitor = (monitor_height - bottom_margin - new_height).max(0);
    let x = monitor_position.x + left_margin;
    let y = monitor_position.y + y_in_monitor;
    Ok((x, y))
}

#[tauri::command(async)]
pub fn set_overlay_panel_size(
    app: tauri::AppHandle,
    width: u32,
    height: u32,
) -> Result<(), CommandError> {
    let native_window = crate::window::native_window();
    let window = ensure_overlay_window(&app)?;
    let width = width.max(1);
    let height = height.max(1);

    let old_panel_size =
        store::get_optional::<OverlayPanelSize>(&app, OVERLAY_PANEL_SIZE_STORE_KEY)?;

    store::set_value(
        &app,
        OVERLAY_PANEL_SIZE_STORE_KEY,
        &OverlayPanelSize { width, height },
    )?;

    let new_height_i32 = i32::try_from(height)
        .map_err(|e| command_error("overlay_panel_window_height_overflow", e.to_string()))?;

    if native_window.uses_layer_shell_margins() {
        native_window.set_size_with_gtk_refresh(&window, width, height)?;
        ensure_always_on_top(&window)?;

        let saved_pos = get_saved_overlay_position(&app)?;
        let position = saved_pos.unwrap_or(OverlayPosition::LayerShellMargins {
            left: OVERLAY_DEFAULT_MARGIN_LEFT_PX,
            bottom: OVERLAY_DEFAULT_MARGIN_BOTTOM_PX,
        });
        let w_i32 = i32::try_from(width).unwrap_or(0);
        let h_i32 = i32::try_from(height).unwrap_or(0);
        refresh_overlay_screen_rect(&window, &position, w_i32, h_i32);

        return Ok(());
    }

    let saved_pos = get_saved_overlay_position(&app)?;

    native_window.set_size_with_gtk_refresh(&window, width, height)?;

    match (&saved_pos, &old_panel_size) {
        (Some(pos), Some(old_size)) => {
            let old_height_i32 = i32::try_from(old_size.height).unwrap_or(0);
            let height_delta = new_height_i32 - old_height_i32;

            if height_delta == 0 {
                apply_overlay_position(&app, pos)?;
            } else {
                let (old_x, old_y) = match pos {
                    OverlayPosition::Absolute { x, y } => (*x, *y),
                    OverlayPosition::LayerShellMargins { left, bottom } => {
                        margins_to_absolute(&window, *left, *bottom)?
                    }
                };
                let new_y = old_y - height_delta;
                let adjusted = OverlayPosition::Absolute { x: old_x, y: new_y };
                apply_overlay_position(&app, &adjusted)?;
                save_overlay_position(&app, &adjusted)?;
            }
        }
        (Some(pos), None) => {
            apply_overlay_position(&app, pos)?;
        }
        (None, _) => {
            let (x, y) = compute_default_absolute_position(&window, new_height_i32)?;
            native_window.set_position(&window, x, y)?;
        }
    }

    ensure_always_on_top(&window)?;

    let final_pos = get_saved_overlay_position(&app)?;
    if let Some(ref pos) = final_pos {
        let size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 0, height: 0 });
        let w = i32::try_from(size.width).unwrap_or(0);
        let h = i32::try_from(size.height).unwrap_or(0);
        refresh_overlay_screen_rect(&window, pos, w, h);
    }

    Ok(())
}
