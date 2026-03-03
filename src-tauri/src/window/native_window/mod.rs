mod trait_def;
mod default_native_window;

#[cfg(linux_bsd_target_os)]
mod gtk_util;

#[cfg(linux_bsd_target_os)]
mod layer_shell_support;

#[cfg(linux_bsd_target_os)]
mod layer_shell_native_window;

#[cfg(linux_bsd_target_os)]
mod x11_native_window;

#[cfg(windows_target_os)]
mod win32_native_window;

pub use trait_def::NativeWindow;

use std::sync::OnceLock;

static NATIVE_WINDOW: OnceLock<Box<dyn NativeWindow>> = OnceLock::new();

pub fn init_native_window() {
    NATIVE_WINDOW.get_or_init(|| {
        #[cfg(linux_bsd_target_os)]
        {
            let native_window: Box<dyn NativeWindow> =
                if layer_shell_support::probe_layer_shell_support() {
                    Box::new(layer_shell_native_window::LayerShellNativeWindow)
                } else {
                    Box::new(x11_native_window::X11NativeWindow)
                };
            native_window.init_window_manager();
            return native_window;
        }

        #[cfg(windows_target_os)]
        {
            let native_window: Box<dyn NativeWindow> =
                Box::new(win32_native_window::Win32NativeWindow);
            native_window.init_window_manager();
            return native_window;
        }

        #[allow(unreachable_code)]
        {
            let native_window: Box<dyn NativeWindow> =
                Box::new(default_native_window::DefaultNativeWindow);
            native_window.init_window_manager();
            native_window
        }
    });
}

pub fn native_window() -> &'static dyn NativeWindow {
    NATIVE_WINDOW
        .get()
        .expect("native window not initialized; call init_native_window() during setup")
        .as_ref()
}
