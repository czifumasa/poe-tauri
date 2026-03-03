use gtk::prelude::WidgetExt;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::error::{command_error, CommandError};
use super::gtk_util::run_on_main_thread;
use super::trait_def::{LayerShellConfig, NativeWindow};

pub struct LayerShellBackend;

fn apply_layer_shell_config(
    window: &tauri::WebviewWindow,
    config: &LayerShellConfig,
) -> Result<(), CommandError> {
    let namespace = config.namespace;
    let keyboard_interactive = config.keyboard_interactive;
    let anchor_left = config.anchor_left;
    let anchor_bottom = config.anchor_bottom;
    let anchor_top = config.anchor_top;
    let anchor_right = config.anchor_right;
    let margin_left = config.default_margin_left;
    let margin_bottom = config.default_margin_bottom;
    let margin_top = config.default_margin_top;
    let margin_right = config.default_margin_right;

    run_on_main_thread(window, move |w| {
        let gtk_window = w
            .gtk_window()
            .map_err(|e| command_error("layer_shell_gtk_window_failed", e.to_string()))?;

        gtk_window.init_layer_shell();
        gtk_window.set_namespace(namespace);
        gtk_window.set_layer(Layer::Overlay);
        gtk_window.set_keyboard_mode(if keyboard_interactive {
            KeyboardMode::OnDemand
        } else {
            KeyboardMode::None
        });
        gtk_window.set_exclusive_zone(0);

        gtk_window.set_anchor(Edge::Top, anchor_top);
        gtk_window.set_anchor(Edge::Left, anchor_left);
        gtk_window.set_anchor(Edge::Bottom, anchor_bottom);
        gtk_window.set_anchor(Edge::Right, anchor_right);

        gtk_window.set_layer_shell_margin(Edge::Top, margin_top);
        gtk_window.set_layer_shell_margin(Edge::Left, margin_left);
        gtk_window.set_layer_shell_margin(Edge::Bottom, margin_bottom);
        gtk_window.set_layer_shell_margin(Edge::Right, margin_right);

        Ok(())
    })
}

impl NativeWindow for LayerShellBackend {
    fn init_window_manager(&self) {
        super::layer_shell_support::init_on_main_thread();
    }

    fn configure_window(
        &self,
        window: &tauri::WebviewWindow,
        layer_shell_config: &LayerShellConfig,
    ) -> Result<(), CommandError> {
        apply_layer_shell_config(window, layer_shell_config)
    }

    fn set_position(
        &self,
        _window: &tauri::WebviewWindow,
        _x: i32,
        _y: i32,
    ) -> Result<(), CommandError> {
        Ok(())
    }

    fn set_layer_shell_margins(
        &self,
        window: &tauri::WebviewWindow,
        left: i32,
        bottom: i32,
    ) -> Result<(), CommandError> {
        let left = left.max(0);
        let bottom = bottom.max(0);

        run_on_main_thread(window, move |w| {
            let gtk_window = w
                .gtk_window()
                .map_err(|e| command_error("layer_shell_gtk_window_failed", e.to_string()))?;

            gtk_window.set_layer_shell_margin(Edge::Left, left);
            gtk_window.set_layer_shell_margin(Edge::Bottom, bottom);

            Ok(())
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
                .map_err(|e| command_error("layer_shell_gtk_window_failed", e.to_string()))?;

            gtk_window.set_size_request(width_i32, height_i32);

            w.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
                .map_err(|e| command_error("window_set_size_failed", e.to_string()))?;

            Ok(())
        })?;

        run_on_main_thread(window, |w| {
            let gtk_window = w
                .gtk_window()
                .map_err(|e| command_error("layer_shell_gtk_window_failed", e.to_string()))?;

            gtk_window.queue_draw();

            w.hide()
                .map_err(|e| command_error("window_hide_failed", e.to_string()))?;

            std::thread::sleep(std::time::Duration::from_millis(1));

            w.show()
                .map_err(|e| command_error("window_show_failed", e.to_string()))?;

            Ok(())
        })
    }

    fn get_position(
        &self,
        _window: &tauri::WebviewWindow,
    ) -> Result<(i32, i32), CommandError> {
        Err(command_error(
            "layer_shell_get_position_unsupported",
            "layer shell windows do not have absolute positions; use saved margins instead",
        ))
    }

    fn uses_layer_shell_margins(&self) -> bool {
        true
    }
}
