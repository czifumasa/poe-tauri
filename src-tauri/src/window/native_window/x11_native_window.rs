use gtk::prelude::{GtkWindowExt, WidgetExt};

use crate::error::{command_error, CommandError};
use super::gtk_util::run_on_main_thread;
use super::trait_def::{LayerShellConfig, NativeWindow};

pub struct X11NativeWindow;

fn apply_x11_window_hints(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
    run_on_main_thread(window, |w| {
        let gtk_window = w
            .gtk_window()
            .map_err(|e| command_error("x11_gtk_window_failed", e.to_string()))?;

        gtk_window.set_keep_above(true);
        gtk_window.set_accept_focus(false);
        gtk_window.set_skip_taskbar_hint(true);
        gtk_window.set_skip_pager_hint(true);

        Ok(())
    })
}

impl NativeWindow for X11NativeWindow {
    fn configure_window(
        &self,
        window: &tauri::WebviewWindow,
        _layer_shell_config: &LayerShellConfig,
    ) -> Result<(), CommandError> {
        apply_x11_window_hints(window)
    }

    fn ensure_always_on_top(&self, window: &tauri::WebviewWindow) -> Result<(), CommandError> {
        window.set_always_on_top(true).map_err(|e| {
            command_error("window_set_always_on_top_failed", e.to_string())
        })?;
        apply_x11_window_hints(window)
    }

    fn get_position(
        &self,
        window: &tauri::WebviewWindow,
    ) -> Result<(i32, i32), CommandError> {
        run_on_main_thread(window, |w| {
            let gtk_window = w
                .gtk_window()
                .map_err(|e| command_error("x11_gtk_window_failed", e.to_string()))?;

            let (x, y) = gtk_window.position();
            Ok((x, y))
        })
    }

    fn set_size_with_gtk_refresh(
        &self,
        window: &tauri::WebviewWindow,
        width: u32,
        height: u32,
    ) -> Result<(), CommandError> {
        let width_i32 = i32::try_from(width)
            .map_err(|e| command_error("window_width_overflow", e.to_string()))?;
        let height_i32 = i32::try_from(height)
            .map_err(|e| command_error("window_height_overflow", e.to_string()))?;

        run_on_main_thread(window, move |w| {
            let gtk_window = w
                .gtk_window()
                .map_err(|e| command_error("x11_gtk_window_failed", e.to_string()))?;

            gtk_window.set_size_request(width_i32, height_i32);

            w.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
                .map_err(|e| command_error("window_set_size_failed", e.to_string()))?;

            Ok(())
        })?;

        run_on_main_thread(window, |w| {
            let gtk_window = w
                .gtk_window()
                .map_err(|e| command_error("x11_gtk_window_failed", e.to_string()))?;

            gtk_window.queue_draw();

            w.hide()
                .map_err(|e| command_error("window_hide_failed", e.to_string()))?;

            std::thread::sleep(std::time::Duration::from_millis(1));

            w.show()
                .map_err(|e| command_error("window_show_failed", e.to_string()))?;

            Ok(())
        })
    }
}
