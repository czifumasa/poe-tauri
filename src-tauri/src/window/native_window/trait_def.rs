use crate::error::CommandError;

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
    fn init_window_manager(&self);

    fn configure_overlay_window(
        &self,
        window: &tauri::WebviewWindow,
        layer_shell_config: &LayerShellConfig,
    ) -> Result<(), CommandError>;

    fn configure_tooltip_window(
        &self,
        window: &tauri::WebviewWindow,
        layer_shell_config: &LayerShellConfig,
    ) -> Result<(), CommandError>;

    fn ensure_always_on_top(&self, window: &tauri::WebviewWindow) -> Result<(), CommandError>;

    fn set_position(
        &self,
        window: &tauri::WebviewWindow,
        x: i32,
        y: i32,
    ) -> Result<(), CommandError>;

    fn set_layer_shell_margins(
        &self,
        window: &tauri::WebviewWindow,
        left: i32,
        bottom: i32,
    ) -> Result<(), CommandError>;

    fn set_size_with_gtk_refresh(
        &self,
        window: &tauri::WebviewWindow,
        width: u32,
        height: u32,
    ) -> Result<(), CommandError>;

    fn get_position(
        &self,
        window: &tauri::WebviewWindow,
    ) -> Result<(i32, i32), CommandError>;

    fn uses_layer_shell_margins(&self) -> bool;
}
