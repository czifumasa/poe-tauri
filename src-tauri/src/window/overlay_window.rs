use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{command_error, CommandError};
use crate::window::identifiers::{
    OVERLAY_DEFAULT_MARGIN_PX, OVERLAY_VIEW_QUERY_VALUE, OVERLAY_WINDOW_LABEL,
};
use crate::window::native_window::LayerShellConfig;

const OVERLAY_LAYER_SHELL_CONFIG: LayerShellConfig = LayerShellConfig {
    namespace: "poe-tauri-overlay",
    keyboard_interactive: true,
    anchor_left: true,
    anchor_bottom: true,
    anchor_top: false,
    anchor_right: false,
    default_margin_left: OVERLAY_DEFAULT_MARGIN_PX,
    default_margin_bottom: OVERLAY_DEFAULT_MARGIN_PX,
    default_margin_top: 0,
    default_margin_right: 0,
};

pub fn ensure_always_on_top(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
    crate::window::native_backend().ensure_always_on_top(window)
}

pub fn ensure_overlay_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, CommandError> {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW_LABEL) {
        return Ok(window);
    }

    WebviewWindowBuilder::new(
        app,
        OVERLAY_WINDOW_LABEL,
        WebviewUrl::App(format!("index.html?view={OVERLAY_VIEW_QUERY_VALUE}").into()),
    )
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .skip_taskbar(true)
    .focusable(true)
    .visible(false)
    .resizable(false)
    .inner_size(340.0, 130.0)
    .build()
    .map_err(|e| command_error("overlay_panel_window_create_failed", e.to_string()))
    .and_then(|window| {
        crate::window::native_backend()
            .configure_overlay_window(&window, &OVERLAY_LAYER_SHELL_CONFIG)?;

        window
            .set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize {
                width: 1,
                height: 1,
            })))
            .map_err(|e| {
                command_error("overlay_panel_window_set_min_size_failed", e.to_string())
            })?;
        Ok(window)
    })
}
