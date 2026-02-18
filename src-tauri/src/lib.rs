use std::sync::mpsc;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

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
use gtk::cairo::{RectangleInt, Region};

#[derive(Clone, serde::Serialize)]
struct CommandError {
    kind: String,
    message: String,
}

fn command_error(kind: &str, message: impl Into<String>) -> CommandError {
    CommandError {
        kind: kind.to_string(),
        message: message.into(),
    }
}

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

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn ensure_overlay_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, CommandError> {
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

fn ensure_overlay_panel_window(
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

#[tauri::command]
fn show_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    let window = ensure_overlay_window(&app)?;
    let panel_window = ensure_overlay_panel_window(&app)?;
    window
        .show()
        .map_err(|e| command_error("overlay_window_show_failed", e.to_string()))?;
    panel_window
        .show()
        .map_err(|e| command_error("overlay_panel_window_show_failed", e.to_string()))?;
    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        window
            .set_always_on_top(true)
            .map_err(|e| command_error("overlay_window_always_on_top_failed", e.to_string()))?;
        window
            .set_visible_on_all_workspaces(true)
            .map_err(|e| command_error("overlay_window_sticky_failed", e.to_string()))?;
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        window
            .set_focusable(false)
            .map_err(|e| command_error("overlay_window_set_focusable_failed", e.to_string()))?;
        window
            .set_ignore_cursor_events(true)
            .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        window
            .set_ignore_cursor_events(true)
            .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
fn set_overlay_panel_size(app: tauri::AppHandle, width: f64, height: f64) -> Result<(), CommandError> {
    let window = ensure_overlay_panel_window(&app)?;
    let width = width.max(1.0);
    let height = height.max(1.0);
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|e| command_error("overlay_panel_window_set_size_failed", e.to_string()))?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct OverlayInputRegion {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[tauri::command]
fn set_overlay_input_region(
    app: tauri::AppHandle,
    region: OverlayInputRegion,
) -> Result<(), CommandError> {
    let window = ensure_overlay_window(&app)?;

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
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
fn hide_overlay(app: tauri::AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window("overlay") {
        window
            .hide()
            .map_err(|e| command_error("overlay_window_hide_failed", e.to_string()))?;
    }
    if let Some(window) = app.get_webview_window("overlay_panel") {
        window
            .hide()
            .map_err(|e| command_error("overlay_panel_window_hide_failed", e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
fn set_overlay_click_through(app: tauri::AppHandle, enabled: bool) -> Result<(), CommandError> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| command_error("overlay_window_not_found", "overlay window not created"))?;
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    Ok(())
}

#[tauri::command]
fn set_overlay_interactive(app: tauri::AppHandle, interactive: bool) -> Result<(), CommandError> {
    let window = ensure_overlay_panel_window(&app)?;

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
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

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        window
            .set_ignore_cursor_events(false)
            .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        window
            .set_ignore_cursor_events(!interactive)
            .map_err(|e| command_error("overlay_window_set_click_through_failed", e.to_string()))?;
    }

    if interactive {
        window
            .set_focus()
            .map_err(|e| command_error("overlay_window_focus_failed", e.to_string()))?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            show_overlay,
            hide_overlay,
            set_overlay_click_through,
            set_overlay_interactive,
            set_overlay_input_region,
            set_overlay_panel_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
