use std::sync::OnceLock;

static LAYER_SHELL_SUPPORTED: OnceLock<bool> = OnceLock::new();

pub fn init_on_main_thread() {
    LAYER_SHELL_SUPPORTED.get_or_init(|| gtk_layer_shell::is_supported());
}

pub fn is_supported() -> bool {
    *LAYER_SHELL_SUPPORTED.get().unwrap_or(&false)
}
