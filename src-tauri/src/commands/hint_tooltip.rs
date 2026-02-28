use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{command_error, CommandError};
use crate::persistence::store;
use crate::window::hint_tooltip_window::{
    ensure_hint_tooltip_always_on_top, ensure_hint_tooltip_window,
};
use crate::window::identifiers::{OVERLAY_DEFAULT_MARGIN_PX, OVERLAY_WINDOW_LABEL};

use crate::commands::overlay::{OverlayPanelSize, OverlayPosition};

#[cfg(linux_bsd_target_os)]
use gtk::prelude::WidgetExt;

#[cfg(linux_bsd_target_os)]
use gtk_layer_shell::{Edge, LayerShell};

#[cfg(linux_bsd_target_os)]
use std::sync::mpsc;

const HINT_TOOLTIP_CONTENT_EVENT: &str = "hint_tooltip_content";

const TOOLTIP_WIDTH_PX: u32 = 360;
const TOOLTIP_HEIGHT_PX: u32 = 260;
const TOOLTIP_GAP_PX: i32 = 6;

const OVERLAY_POSITION_STORE_KEY: &str = "overlay_position";
const OVERLAY_PANEL_SIZE_STORE_KEY: &str = "overlay_panel_size";

static LAST_TOOLTIP_CONTENT: OnceLock<Mutex<Option<HintTooltipContentPayload>>> = OnceLock::new();

fn last_tooltip_content() -> &'static Mutex<Option<HintTooltipContentPayload>> {
    LAST_TOOLTIP_CONTENT.get_or_init(|| Mutex::new(None))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HintTooltipContentPayload {
    pub key: String,
    pub data_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HintTooltipShowArgs {
    pub key: String,
    pub data_uri: String,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

fn monitor_height(monitor: Rect) -> i32 {
    monitor.bottom - monitor.top
}

fn load_saved_overlay_position(app: &AppHandle) -> Result<Option<OverlayPosition>, CommandError> {
    store::get_optional::<OverlayPosition>(app, OVERLAY_POSITION_STORE_KEY)
}

fn load_saved_overlay_panel_size(
    app: &AppHandle,
) -> Result<Option<OverlayPanelSize>, CommandError> {
    store::get_optional::<OverlayPanelSize>(app, OVERLAY_PANEL_SIZE_STORE_KEY)
}

fn overlay_origin_physical(
    app: &AppHandle,
    overlay: &tauri::WebviewWindow,
    monitor: Rect,
) -> Result<(i32, i32), CommandError> {
    #[cfg(linux_bsd_target_os)]
    {
        if gtk_layer_shell::is_supported() {
            let saved = load_saved_overlay_position(app)?;
            let (left, bottom) = match saved {
                Some(OverlayPosition::LayerShellMargins { left, bottom }) => (left, bottom),
                Some(OverlayPosition::Absolute { .. }) | None => {
                    (OVERLAY_DEFAULT_MARGIN_PX, OVERLAY_DEFAULT_MARGIN_PX)
                }
            };

            let window_height = match load_saved_overlay_panel_size(app)? {
                Some(size) => i32::try_from(size.height).map_err(|e| {
                    command_error("hint_tooltip_overlay_height_overflow", e.to_string())
                })?,
                None => {
                    let outer_size = overlay.outer_size().map_err(|e| {
                        command_error("hint_tooltip_overlay_size_failed", e.to_string())
                    })?;
                    i32::try_from(outer_size.height).map_err(|e| {
                        command_error("hint_tooltip_overlay_height_overflow", e.to_string())
                    })?
                }
            };

            let left = left.max(0);
            let bottom = bottom.max(0);

            let x = monitor.left + left;
            let y_in_monitor = (monitor_height(monitor) - bottom - window_height).max(0);
            let y = monitor.top + y_in_monitor;
            return Ok((x, y));
        }
    }

    let pos = overlay
        .outer_position()
        .map_err(|e| command_error("hint_tooltip_overlay_position_failed", e.to_string()))?;
    Ok((pos.x, pos.y))
}

fn monitor_rect_for_window(window: &tauri::WebviewWindow) -> Result<Rect, CommandError> {
    let monitor = window
        .current_monitor()
        .map_err(|e| command_error("hint_tooltip_get_monitor_failed", e.to_string()))?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| command_error("hint_tooltip_get_monitor_failed", "no monitor available"))?;

    let pos = monitor.position();
    let size = monitor.size();

    let width = i32::try_from(size.width)
        .map_err(|e| command_error("hint_tooltip_monitor_width_overflow", e.to_string()))?;
    let height = i32::try_from(size.height)
        .map_err(|e| command_error("hint_tooltip_monitor_height_overflow", e.to_string()))?;

    Ok(Rect {
        left: pos.x,
        top: pos.y,
        right: pos.x + width,
        bottom: pos.y + height,
    })
}

fn pick_tooltip_position(overlay: Rect, monitor: Rect) -> Option<(i32, i32)> {
    let width_i32 = i32::try_from(TOOLTIP_WIDTH_PX).ok()?;
    let height_i32 = i32::try_from(TOOLTIP_HEIGHT_PX).ok()?;

    let space_above = overlay.top - monitor.top - TOOLTIP_GAP_PX;
    let space_below = monitor.bottom - overlay.bottom - TOOLTIP_GAP_PX;

    let can_above = space_above >= height_i32;
    let can_below = space_below >= height_i32;

    let place_above = match (can_above, can_below) {
        (true, true) => space_above >= space_below,
        (true, false) => true,
        (false, true) => false,
        (false, false) => return None,
    };

    let overlay_center_x = overlay.left + (overlay.right - overlay.left) / 2;
    let raw_x = overlay_center_x - width_i32 / 2;
    let x = clamp_i32(raw_x, monitor.left, monitor.right - width_i32);
    let y = if place_above {
        overlay.top - TOOLTIP_GAP_PX - height_i32
    } else {
        overlay.bottom + TOOLTIP_GAP_PX
    };

    Some((x, y))
}

fn position_tooltip_window(app: &AppHandle) -> Result<(), CommandError> {
    let tooltip = ensure_hint_tooltip_window(app)?;

    let overlay = app
        .get_webview_window(OVERLAY_WINDOW_LABEL)
        .ok_or_else(|| {
            command_error(
                "hint_tooltip_overlay_missing",
                "overlay window not available",
            )
        })?;

    let monitor_rect = monitor_rect_for_window(&overlay)?;
    let overlay_origin = overlay_origin_physical(app, &overlay, monitor_rect)?;

    let (overlay_width, overlay_height) = match load_saved_overlay_panel_size(app)? {
        Some(size) => (
            i32::try_from(size.width)
                .map_err(|e| command_error("hint_tooltip_overlay_width_overflow", e.to_string()))?,
            i32::try_from(size.height).map_err(|e| {
                command_error("hint_tooltip_overlay_height_overflow", e.to_string())
            })?,
        ),
        None => {
            let overlay_size = overlay
                .outer_size()
                .map_err(|e| command_error("hint_tooltip_overlay_size_failed", e.to_string()))?;
            (
                i32::try_from(overlay_size.width).map_err(|e| {
                    command_error("hint_tooltip_overlay_width_overflow", e.to_string())
                })?,
                i32::try_from(overlay_size.height).map_err(|e| {
                    command_error("hint_tooltip_overlay_height_overflow", e.to_string())
                })?,
            )
        }
    };

    let overlay_rect = Rect {
        left: overlay_origin.0,
        top: overlay_origin.1,
        right: overlay_origin.0 + overlay_width,
        bottom: overlay_origin.1 + overlay_height,
    };

    let Some((x, y)) = pick_tooltip_position(overlay_rect, monitor_rect) else {
        return Err(command_error(
            "hint_tooltip_no_space",
            "no space around overlay window for tooltip",
        ));
    };

    let width = TOOLTIP_WIDTH_PX;
    let height = TOOLTIP_HEIGHT_PX;

    tooltip
        .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
        .map_err(|e| command_error("hint_tooltip_set_size_failed", e.to_string()))?;

    #[cfg(linux_bsd_target_os)]
    {
        if gtk_layer_shell::is_supported() {
            let left_margin = (x - monitor_rect.left).max(0);
            let top_margin = (y - monitor_rect.top).max(0);

            let width_i32 = i32::try_from(width)
                .map_err(|e| command_error("hint_tooltip_width_overflow", e.to_string()))?;
            let height_i32 = i32::try_from(height)
                .map_err(|e| command_error("hint_tooltip_height_overflow", e.to_string()))?;

            let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
            let tooltip_for_closure = tooltip.clone();
            tooltip
                .run_on_main_thread(move || {
                    let result = (|| {
                        let gtk_window = tooltip_for_closure.gtk_window().map_err(|e| {
                            command_error("hint_tooltip_window_gtk_window_failed", e.to_string())
                        })?;

                        gtk_window.set_layer_shell_margin(Edge::Left, left_margin);
                        gtk_window.set_layer_shell_margin(Edge::Top, top_margin);
                        gtk_window.set_size_request(width_i32, height_i32);
                        Ok(())
                    })();
                    let _ = sender.send(result);
                })
                .map_err(|e| {
                    command_error("hint_tooltip_window_main_thread_failed", e.to_string())
                })?;

            receiver.recv().map_err(|e| {
                command_error(
                    "hint_tooltip_window_main_thread_channel_failed",
                    e.to_string(),
                )
            })??;

            return Ok(());
        }
    }

    tooltip
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| command_error("hint_tooltip_set_position_failed", e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn hint_tooltip_show(
    app: tauri::AppHandle,
    args: HintTooltipShowArgs,
) -> Result<(), CommandError> {
    let window = ensure_hint_tooltip_window(&app)?;

    position_tooltip_window(&app)?;

    let payload = HintTooltipContentPayload {
        key: args.key,
        data_uri: args.data_uri,
    };

    if let Ok(mut guard) = last_tooltip_content().lock() {
        *guard = Some(payload.clone());
    }

    window
        .show()
        .map_err(|e| command_error("hint_tooltip_show_failed", e.to_string()))?;

    ensure_hint_tooltip_always_on_top(&window)?;

    window
        .emit(HINT_TOOLTIP_CONTENT_EVENT, payload)
        .map_err(|e| command_error("hint_tooltip_emit_failed", e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn hint_tooltip_get_last_content(_app: tauri::AppHandle) -> Option<HintTooltipContentPayload> {
    last_tooltip_content()
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

#[tauri::command]
pub fn hint_tooltip_hide(app: tauri::AppHandle) -> Result<(), CommandError> {
    if let Some(window) =
        app.get_webview_window(crate::window::identifiers::HINT_TOOLTIP_WINDOW_LABEL)
    {
        window
            .hide()
            .map_err(|e| command_error("hint_tooltip_hide_failed", e.to_string()))?;
    }
    Ok(())
}
