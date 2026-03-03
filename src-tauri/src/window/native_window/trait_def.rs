use crate::error::{command_error, CommandError};

pub struct LayerShellConfig {
    pub namespace: &'static str,
    pub keyboard_interactive: bool,
    pub anchor_left: bool,
    pub anchor_bottom: bool,
    pub anchor_top: bool,
    pub anchor_right: bool,
    pub default_margin_left: i32,
    pub default_margin_bottom: i32,
    pub default_margin_top: i32,
    pub default_margin_right: i32,
}

pub trait NativeWindow: Send + Sync {
    fn init_window_manager(&self) {}

    fn configure_window(
        &self,
        _window: &tauri::WebviewWindow,
        _layer_shell_config: &LayerShellConfig,
    ) -> Result<(), CommandError> {
        Ok(())
    }

    fn ensure_always_on_top(&self, window: &tauri::WebviewWindow) -> Result<(), CommandError> {
        window.set_always_on_top(true).map_err(|e| {
            command_error("window_set_always_on_top_failed", e.to_string())
        })
    }

    fn set_position(
        &self,
        window: &tauri::WebviewWindow,
        x: i32,
        y: i32,
    ) -> Result<(), CommandError> {
        window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
            .map_err(|e| command_error("window_set_position_failed", e.to_string()))
    }

    fn set_layer_shell_margins(
        &self,
        _window: &tauri::WebviewWindow,
        _left: i32,
        _bottom: i32,
    ) -> Result<(), CommandError> {
        Ok(())
    }

    fn set_size_with_gtk_refresh(
        &self,
        window: &tauri::WebviewWindow,
        width: u32,
        height: u32,
    ) -> Result<(), CommandError> {
        window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
            .map_err(|e| command_error("window_set_size_failed", e.to_string()))
    }

    fn get_position(
        &self,
        window: &tauri::WebviewWindow,
    ) -> Result<(i32, i32), CommandError> {
        let pos = window
            .outer_position()
            .map_err(|e| command_error("window_get_position_failed", e.to_string()))?;
        Ok((pos.x, pos.y))
    }

    fn uses_layer_shell_margins(&self) -> bool {
        false
    }
}
