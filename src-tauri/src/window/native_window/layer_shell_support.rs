use std::sync::OnceLock;

static LAYER_SHELL_SUPPORTED: OnceLock<bool> = OnceLock::new();

pub fn probe_layer_shell_support() -> bool {
    *LAYER_SHELL_SUPPORTED.get_or_init(|| gtk_layer_shell::is_supported())
}

pub fn init_on_main_thread() {
    LAYER_SHELL_SUPPORTED.get_or_init(|| gtk_layer_shell::is_supported());
}
