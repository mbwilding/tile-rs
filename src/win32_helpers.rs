use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindow, GetWindowLongPtrW, IsWindowVisible, GWL_EXSTYLE, GWL_STYLE, GW_OWNER, WS_CHILD,
    WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;
    let _ = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut c_void,
            size_of::<u32>() as u32,
        )
    };

    cloaked != 0
}

// TODO: Check implementation
pub fn is_app_window(hwnd: HWND) -> bool {
    unsafe {
        IsWindowVisible(hwnd).as_bool()
            && GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_NOACTIVATE.0 as isize == 0
            && GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_CHILD.0 as isize == 0
    }
}

// TODO: Check implementation
pub fn is_alt_tab_window(hwnd: HWND) -> bool {
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);

        if ex_style & (WS_EX_TOOLWINDOW.0 as isize) != 0 || GetWindow(hwnd, GW_OWNER) != HWND(0) {
            return false;
        }

        if ex_style & (WS_EX_APPWINDOW.0 as isize) != 0 {
            return true;
        }

        true
    }
}
