use std::sync::mpsc;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{command_error, CommandError};
use crate::window::identifiers::{HINT_TOOLTIP_VIEW_QUERY_VALUE, HINT_TOOLTIP_WINDOW_LABEL};

#[cfg(linux_bsd_target_os)]
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

fn configure_hint_tooltip_layer_shell(window: &tauri::WebviewWindow) -> Result<bool, CommandError> {
    #[cfg(linux_bsd_target_os)]
    {
        let (sender, receiver) = mpsc::channel::<Result<bool, CommandError>>();
        let window = window.clone();
        let window_for_closure = window.clone();

        window
            .run_on_main_thread(move || {
                let result = (|| {
                    if !gtk_layer_shell::is_supported() {
                        return Ok(false);
                    }

                    let gtk_window = window_for_closure
                        .gtk_window()
                        .map_err(|e| command_error("hint_tooltip_window_gtk_window_failed", e.to_string()))?;

                    gtk_window.init_layer_shell();
                    gtk_window.set_namespace("poe-tauri-hint-tooltip");
                    gtk_window.set_layer(Layer::Overlay);
                    gtk_window.set_keyboard_mode(KeyboardMode::None);
                    gtk_window.set_exclusive_zone(0);

                    gtk_window.set_anchor(Edge::Top, true);
                    gtk_window.set_anchor(Edge::Left, true);
                    gtk_window.set_anchor(Edge::Bottom, false);
                    gtk_window.set_anchor(Edge::Right, false);

                    gtk_window.set_layer_shell_margin(Edge::Top, 0);
                    gtk_window.set_layer_shell_margin(Edge::Left, 0);
                    gtk_window.set_layer_shell_margin(Edge::Bottom, 0);
                    gtk_window.set_layer_shell_margin(Edge::Right, 0);

                    Ok(true)
                })();

                let _ = sender.send(result);
            })
            .map_err(|e| command_error("hint_tooltip_window_main_thread_failed", e.to_string()))?;

        return receiver
            .recv()
            .map_err(|e| command_error("hint_tooltip_window_main_thread_channel_failed", e.to_string()))?;
    }

    #[cfg(not(linux_bsd_target_os))]
    {
        let _ = window;
        Ok(false)
    }
}

pub fn ensure_hint_tooltip_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, CommandError> {
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
        let _ = configure_hint_tooltip_layer_shell(&window)?;
        window
            .set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize { width: 1, height: 1 })))
            .map_err(|e| command_error("hint_tooltip_window_set_min_size_failed", e.to_string()))?;
        Ok(window)
    })
}
