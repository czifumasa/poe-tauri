use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{command_error, CommandError};
use crate::window::identifiers::{OVERLAY_VIEW_QUERY_VALUE, OVERLAY_WINDOW_LABEL};

pub fn ensure_always_on_top(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
    crate::window::native_window().ensure_always_on_top(window)
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
        if let Some(config) = crate::window::native_window().create_overlay_config() {
            crate::window::native_window()
                .configure_window(&window, &config)?;
        }

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
