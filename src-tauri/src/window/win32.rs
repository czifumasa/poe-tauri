use crate::error::{command_error, CommandError};

use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
    GWL_STYLE, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW,
};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

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

pub fn configure_overlay_style(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
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

pub fn configure_tooltip_style(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
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

pub fn set_topmost(window: &tauri::WebviewWindow) -> Result<(), CommandError> {
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

pub fn set_position(
    window: &tauri::WebviewWindow,
    x: i32,
    y: i32,
) -> Result<(), CommandError> {
    let hwnd = extract_hwnd(window)?;

    unsafe {
        SetWindowPos(hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_FRAMECHANGED);
    }

    Ok(())
}

pub fn set_position_and_size(
    window: &tauri::WebviewWindow,
    x: i32,
    y: i32,
    client_width: i32,
    client_height: i32,
) -> Result<(), CommandError> {
    let hwnd = extract_hwnd(window)?;

    let (outer_width, outer_height) = unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: client_width,
            bottom: client_height,
        };

        AdjustWindowRectEx(&mut rect, style, 0, ex_style);

        (rect.right - rect.left, rect.bottom - rect.top)
    };

    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            x,
            y,
            outer_width,
            outer_height,
            SWP_FRAMECHANGED,
        );
    }

    Ok(())
}
