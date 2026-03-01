#[cfg(linux_bsd_target_os)]
use std::sync::mpsc;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{command_error, CommandError};
use crate::window::identifiers::{OVERLAY_VIEW_QUERY_VALUE, OVERLAY_WINDOW_LABEL};

#[cfg(linux_bsd_target_os)]
use crate::window::identifiers::OVERLAY_DEFAULT_MARGIN_PX;

#[cfg(linux_bsd_target_os)]
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

#[cfg(linux_bsd_target_os)]
use gtk::prelude::GtkWindowExt;

#[cfg(linux_bsd_target_os)]
fn configure_x11_window_hints(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
    let (sender, receiver) = mpsc::channel::<Result<(), CommandError>>();
    let window_for_closure = window.clone();

    window
        .run_on_main_thread(move || {
            let result = (|| {
                let gtk_window = window_for_closure.gtk_window().map_err(|e| {
                    command_error("overlay_panel_window_gtk_window_failed", e.to_string())
                })?;

                gtk_window.set_keep_above(true);
                gtk_window.set_accept_focus(false);
                gtk_window.set_skip_taskbar_hint(true);
                gtk_window.set_skip_pager_hint(true);

                Ok(())
            })();

            let _ = sender.send(result);
        })
        .map_err(|e| command_error("overlay_panel_window_main_thread_failed", e.to_string()))?;

    receiver.recv().map_err(|e| {
        command_error(
            "overlay_panel_window_main_thread_channel_failed",
            e.to_string(),
        )
    })?
}

#[cfg(linux_bsd_target_os)]
fn configure_overlay_layer_shell(window: &tauri::WebviewWindow) -> Result<bool, CommandError> {
    let (sender, receiver) = mpsc::channel::<Result<bool, CommandError>>();
    let window = window.clone();
    let window_for_closure = window.clone();

    window
        .run_on_main_thread(move || {
            let result = (|| {
                if !crate::window::layer_shell_support::is_supported() {
                    return Ok(false);
                }

                let gtk_window = window_for_closure.gtk_window().map_err(|e| {
                    command_error("overlay_panel_window_gtk_window_failed", e.to_string())
                })?;

                gtk_window.init_layer_shell();
                gtk_window.set_namespace("poe-tauri-overlay");
                gtk_window.set_layer(Layer::Overlay);
                gtk_window.set_keyboard_mode(KeyboardMode::OnDemand);
                gtk_window.set_exclusive_zone(0);

                gtk_window.set_anchor(Edge::Top, false);
                gtk_window.set_anchor(Edge::Left, true);
                gtk_window.set_anchor(Edge::Bottom, true);
                gtk_window.set_anchor(Edge::Right, false);

                gtk_window.set_layer_shell_margin(Edge::Top, 0);
                gtk_window.set_layer_shell_margin(Edge::Left, OVERLAY_DEFAULT_MARGIN_PX);
                gtk_window.set_layer_shell_margin(Edge::Bottom, OVERLAY_DEFAULT_MARGIN_PX);
                gtk_window.set_layer_shell_margin(Edge::Right, 0);

                Ok(true)
            })();

            let _ = sender.send(result);
        })
        .map_err(|e| command_error("overlay_panel_window_main_thread_failed", e.to_string()))?;

    receiver.recv().map_err(|e| {
        command_error(
            "overlay_panel_window_main_thread_channel_failed",
            e.to_string(),
        )
    })?
}

#[cfg(windows_target_os)]
fn configure_win32_overlay_window(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
    crate::window::win32::configure_overlay_style(window)
}

pub fn ensure_always_on_top(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
    window.set_always_on_top(true).map_err(|e| {
        command_error(
            "overlay_panel_window_set_always_on_top_failed",
            e.to_string(),
        )
    })?;

    #[cfg(linux_bsd_target_os)]
    {
        if !crate::window::layer_shell_support::is_supported() {
            configure_x11_window_hints(window)?;
        }
    }

    #[cfg(windows_target_os)]
    {
        crate::window::win32::set_topmost(window)?;
    }

    Ok(())
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
        #[cfg(linux_bsd_target_os)]
        {
            let is_layer_shell = configure_overlay_layer_shell(&window)?;
            if !is_layer_shell {
                configure_x11_window_hints(&window)?;
            }
        }

        #[cfg(windows_target_os)]
        {
            configure_win32_overlay_window(&window)?;
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
