use std::sync::mpsc;

use crate::error::{command_error, CommandError};

pub fn run_on_main_thread<F, R>(window: &tauri::WebviewWindow, f: F) -> Result<R, CommandError>
where
    F: FnOnce(&tauri::WebviewWindow) -> Result<R, CommandError> + Send + 'static,
    R: Send + 'static,
{
    let (sender, receiver) = mpsc::channel::<Result<R, CommandError>>();
    let window_for_closure = window.clone();

    window
        .run_on_main_thread(move || {
            let result = f(&window_for_closure);
            let _ = sender.send(result);
        })
        .map_err(|e| command_error("gtk_main_thread_dispatch_failed", e.to_string()))?;

    receiver
        .recv()
        .map_err(|e| command_error("gtk_main_thread_channel_failed", e.to_string()))?
}
