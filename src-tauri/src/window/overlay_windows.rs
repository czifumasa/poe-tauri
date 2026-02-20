use std::sync::mpsc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use crate::error::{command_error, CommandError};

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use gtk::prelude::WidgetExt;

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use gtk::cairo::Region;

fn configure_overlay_layer_shell(
    window: &tauri::WebviewWindow,
) -> Result<bool, CommandError> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
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
                        .map_err(|e| command_error("overlay_window_gtk_window_failed", e.to_string()))?;

                    gtk_window.init_layer_shell();
                    gtk_window.set_namespace("poe-tauri-overlay");
                    gtk_window.set_layer(Layer::Overlay);
                    gtk_window.set_keyboard_mode(KeyboardMode::None);
                    gtk_window.set_exclusive_zone(0);

                    gtk_window.set_anchor(Edge::Top, true);
                    gtk_window.set_anchor(Edge::Bottom, true);
                    gtk_window.set_anchor(Edge::Left, true);
                    gtk_window.set_anchor(Edge::Right, true);

                    gtk_window.set_layer_shell_margin(Edge::Top, 0);
                    gtk_window.set_layer_shell_margin(Edge::Bottom, 0);
                    gtk_window.set_layer_shell_margin(Edge::Left, 0);
                    gtk_window.set_layer_shell_margin(Edge::Right, 0);

                    let empty_region = Region::create();
                    gtk_window.input_shape_combine_region(Some(&empty_region));

                    Ok(true)
                })();

                let _ = sender.send(result);
            })
            .map_err(|e| command_error("overlay_window_main_thread_failed", e.to_string()))?;

        return receiver
            .recv()
            .map_err(|e| command_error("overlay_window_main_thread_channel_failed", e.to_string()))?;
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        let _ = window;
        Ok(false)
    }
}

pub fn ensure_overlay_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, CommandError> {
    if let Some(window) = app.get_webview_window("overlay") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(
        app,
        "overlay",
        WebviewUrl::App("index.html?view=overlay".into()),
    )
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .skip_taskbar(true)
    .focusable(false)
    .visible(false)
    .resizable(true)
    .maximized(true)
    .build()
    .map_err(|e| command_error("overlay_window_create_failed", e.to_string()))
    .and_then(|window| {
        let _ = configure_overlay_layer_shell(&window)?;
        Ok(window)
    })
}

fn configure_overlay_panel_layer_shell(
    window: &tauri::WebviewWindow,
) -> Result<bool, CommandError> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
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
                        .map_err(|e| {
                            command_error("overlay_panel_window_gtk_window_failed", e.to_string())
                        })?;

                    gtk_window.init_layer_shell();
                    gtk_window.set_namespace("poe-tauri-overlay-panel");
                    gtk_window.set_layer(Layer::Overlay);
                    gtk_window.set_keyboard_mode(KeyboardMode::None);
                    gtk_window.set_exclusive_zone(0);

                    gtk_window.set_anchor(Edge::Top, true);
                    gtk_window.set_anchor(Edge::Left, true);
                    gtk_window.set_anchor(Edge::Bottom, false);
                    gtk_window.set_anchor(Edge::Right, false);

                    gtk_window.set_layer_shell_margin(Edge::Top, 24);
                    gtk_window.set_layer_shell_margin(Edge::Left, 24);
                    gtk_window.set_layer_shell_margin(Edge::Bottom, 0);
                    gtk_window.set_layer_shell_margin(Edge::Right, 0);

                    Ok(true)
                })();

                let _ = sender.send(result);
            })
            .map_err(|e| command_error("overlay_panel_window_main_thread_failed", e.to_string()))?;

        return receiver.recv().map_err(|e| {
            command_error("overlay_panel_window_main_thread_channel_failed", e.to_string())
        })?;
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        let _ = window;
        Ok(false)
    }
}

pub fn ensure_overlay_panel_window(
    app: &tauri::AppHandle,
) -> Result<tauri::WebviewWindow, CommandError> {
    if let Some(window) = app.get_webview_window("overlay_panel") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(
        app,
        "overlay_panel",
        WebviewUrl::App("index.html?view=overlay-panel".into()),
    )
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .skip_taskbar(true)
    .focusable(true)
    .visible(false)
    .resizable(false)
    .inner_size(10.0, 10.0)
    .build()
    .map_err(|e| command_error("overlay_panel_window_create_failed", e.to_string()))
    .and_then(|window| {
        let _ = configure_overlay_panel_layer_shell(&window)?;
        Ok(window)
    })
}
