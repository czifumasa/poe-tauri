use crate::error::{command_error, CommandError};
use super::trait_def::{LayerShellConfig, NativeWindow};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
    SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub struct Win32Backend;

fn extract_hwnd(window: &tauri::WebviewWindow) -> Result<HWND, CommandError> {
    let handle = window
        .window_handle()
        .map_err(|e| command_error("win32_window_handle_failed", e.to_string()))?;

    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Ok(h.hwnd.get() as HWND),
        _ => Err(command_error(
            "win32_unexpected_handle_type",
            "expected Win32 window handle",
        )),
    }
}

fn apply_tool_window_style(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
    let hwnd = extract_hwnd(window)?;

    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_style = ex_style | (WS_EX_NOACTIVATE as isize) | (WS_EX_TOOLWINDOW as isize);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);

        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED,
        );
    }

    Ok(())
}

impl NativeWindow for Win32Backend {
    fn configure_window(
        &self,
        window: &tauri::WebviewWindow,
        _layer_shell_config: &LayerShellConfig,
    ) -> Result<(), CommandError> {
        apply_tool_window_style(window)
    }

    fn ensure_always_on_top(&self, window: &tauri::WebviewWindow) -> Result<(), CommandError> {
        window.set_always_on_top(true).map_err(|e| {
            command_error("window_set_always_on_top_failed", e.to_string())
        })?;

        let hwnd = extract_hwnd(window)?;

        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
        }

        Ok(())
    }

    fn set_position(
        &self,
        window: &tauri::WebviewWindow,
        x: i32,
        y: i32,
    ) -> Result<(), CommandError> {
        let hwnd = extract_hwnd(window)?;

        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_FRAMECHANGED,
            );
        }

        Ok(())
    }
}
