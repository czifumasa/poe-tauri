mod trait_def;
mod default_native_window;

#[cfg(linux_bsd_target_os)]
mod layer_shell_support;

#[cfg(linux_bsd_target_os)]
mod layer_shell_native_window;

#[cfg(linux_bsd_target_os)]
mod x11_native_window;

#[cfg(windows_target_os)]
mod win32_native_window;

pub use trait_def::{LayerShellConfig, NativeWindow};

use std::sync::OnceLock;

static NATIVE_BACKEND: OnceLock<Box<dyn NativeWindow>> = OnceLock::new();

pub fn init_native_backend() {
    NATIVE_BACKEND.get_or_init(|| {
        #[cfg(linux_bsd_target_os)]
        {
            let backend: Box<dyn NativeWindow> =
                if layer_shell_support::probe_layer_shell_support() {
                    Box::new(layer_shell_native_window::LayerShellBackend)
                } else {
                    Box::new(x11_native_window::X11Backend)
                };
            backend.init_window_manager();
            return backend;
        }

        #[cfg(windows_target_os)]
        {
            let backend: Box<dyn NativeWindow> =
                Box::new(win32_native_window::Win32Backend);
            backend.init_window_manager();
            return backend;
        }

        #[allow(unreachable_code)]
        {
            let backend: Box<dyn NativeWindow> =
                Box::new(default_native_window::DefaultBackend);
            backend.init_window_manager();
            backend
        }
    });
}

pub fn native_backend() -> &'static dyn NativeWindow {
    NATIVE_BACKEND
        .get()
        .expect("native backend not initialized; call init_native_backend() during setup")
        .as_ref()
}
