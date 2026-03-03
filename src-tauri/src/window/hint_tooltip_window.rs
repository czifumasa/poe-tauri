use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{command_error, CommandError};
use crate::window::identifiers::{HINT_TOOLTIP_VIEW_QUERY_VALUE, HINT_TOOLTIP_WINDOW_LABEL};
use crate::window::native_window::LayerShellConfig;

const TOOLTIP_LAYER_SHELL_CONFIG: LayerShellConfig = LayerShellConfig {
    namespace: "poe-tauri-hint-tooltip",
    keyboard_interactive: false,
    anchor_left: true,
    anchor_bottom: false,
    anchor_top: true,
    anchor_right: false,
    default_margin_left: 0,
    default_margin_bottom: 0,
    default_margin_top: 0,
    default_margin_right: 0,
};

pub fn ensure_hint_tooltip_always_on_top(
    window: &tauri::WebviewWindow,
) -> Result<(), CommandError> {
    crate::window::native_backend().ensure_always_on_top(window)
}

pub fn ensure_hint_tooltip_window(
    app: &tauri::AppHandle,
) -> Result<tauri::WebviewWindow, CommandError> {
    if let Some(window) = app.get_webview_window(HINT_TOOLTIP_WINDOW_LABEL) {
        return Ok(window);
    }

    WebviewWindowBuilder::new(
        app,
        HINT_TOOLTIP_WINDOW_LABEL,
        WebviewUrl::App(format!("index.html?view={HINT_TOOLTIP_VIEW_QUERY_VALUE}").into()),
    )
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .skip_taskbar(true)
    .focusable(false)
    .visible(false)
    .resizable(false)
    .inner_size(320.0, 240.0)
    .build()
    .map_err(|e| command_error("hint_tooltip_window_create_failed", e.to_string()))
    .and_then(|window| {
        crate::window::native_backend()
            .configure_tooltip_window(&window, &TOOLTIP_LAYER_SHELL_CONFIG)?;

        window
            .set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize {
                width: 1,
                height: 1,
            })))
            .map_err(|e| command_error("hint_tooltip_window_set_min_size_failed", e.to_string()))?;
        Ok(window)
    })
}
