use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{command_error, CommandError};
use crate::window::hint_tooltip_window::{
    ensure_hint_tooltip_always_on_top, ensure_hint_tooltip_window,
};
use crate::window::identifiers::OVERLAY_WINDOW_LABEL;

use crate::commands::overlay::{OverlayScreenRect, get_overlay_screen_rect};

const HINT_TOOLTIP_CONTENT_EVENT: &str = "hint_tooltip_content";
const HINT_TOOLTIP_CLEAR_EVENT: &str = "hint_tooltip_clear";

const TOOLTIP_WIDTH_PX: u32 = 360;
const TOOLTIP_HEIGHT_PX: u32 = 260;
const TOOLTIP_GAP_PX: i32 = 6;

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

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

fn monitor_rect_for_window(
    window: &tauri::WebviewWindow,
) -> Result<OverlayScreenRect, CommandError> {
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

    Ok(OverlayScreenRect {
        left: pos.x,
        top: pos.y,
        right: pos.x + width,
        bottom: pos.y + height,
    })
}

fn pick_tooltip_position(
    overlay: OverlayScreenRect,
    monitor: OverlayScreenRect,
) -> Option<(i32, i32)> {
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
    let native_window = crate::window::native_window();
    let tooltip = ensure_hint_tooltip_window(app)?;

    let overlay_window = app
        .get_webview_window(OVERLAY_WINDOW_LABEL)
        .ok_or_else(|| {
            command_error(
                "hint_tooltip_overlay_missing",
                "overlay window not available",
            )
        })?;

    let overlay_rect = get_overlay_screen_rect().ok_or_else(|| {
        command_error(
            "hint_tooltip_overlay_rect_missing",
            "overlay screen rect not available",
        )
    })?;

    let monitor_rect = monitor_rect_for_window(&overlay_window)?;

    let Some((x, y)) = pick_tooltip_position(overlay_rect, monitor_rect) else {
        return Err(command_error(
            "hint_tooltip_no_space",
            "no space around overlay window for tooltip",
        ));
    };

    let width = TOOLTIP_WIDTH_PX;
    let height = TOOLTIP_HEIGHT_PX;

    if native_window.uses_layer_shell_margins() {
        let height_i32 = i32::try_from(height)
            .map_err(|e| command_error("hint_tooltip_height_overflow", e.to_string()))?;

        let left_margin = (x - monitor_rect.left).max(0);
        let bottom_margin = (monitor_rect.bottom - y - height_i32).max(0);

        native_window.set_size_with_gtk_refresh(&tooltip, width, height)?;
        return native_window.set_layer_shell_margins(&tooltip, left_margin, bottom_margin);
    }

    tooltip
        .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
        .map_err(|e| command_error("hint_tooltip_set_size_failed", e.to_string()))?;

    native_window.set_position(&tooltip, x, y)
}

#[tauri::command(async)]
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
        .emit(HINT_TOOLTIP_CLEAR_EVENT, ())
        .map_err(|e| command_error("hint_tooltip_emit_clear_failed", e.to_string()))?;

    window
        .emit(HINT_TOOLTIP_CONTENT_EVENT, payload)
        .map_err(|e| command_error("hint_tooltip_emit_failed", e.to_string()))?;

    Ok(())
}

#[tauri::command(async)]
pub fn hint_tooltip_get_last_content(_app: tauri::AppHandle) -> Option<HintTooltipContentPayload> {
    last_tooltip_content()
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

#[tauri::command(async)]
pub fn hint_tooltip_hide(app: tauri::AppHandle) -> Result<(), CommandError> {
    if let Some(window) =
        app.get_webview_window(crate::window::identifiers::HINT_TOOLTIP_WINDOW_LABEL)
    {
        if let Ok(mut guard) = last_tooltip_content().lock() {
            *guard = None;
        }

        window
            .emit(HINT_TOOLTIP_CLEAR_EVENT, ())
            .map_err(|e| command_error("hint_tooltip_emit_clear_failed", e.to_string()))?;

        window
            .hide()
            .map_err(|e| command_error("hint_tooltip_hide_failed", e.to_string()))?;
    }
    Ok(())
}
