use crate::csharp::structs::{Rectangle, Size};
use std::ffi::c_void;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SystemParametersInfoW, SM_CMONITORS, SM_CXSCREEN, SM_CXVIRTUALSCREEN,
    SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SPI_GETWORKAREA,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

pub fn multi_monitor_support() -> bool {
    unsafe { GetSystemMetrics(SM_CMONITORS) != 0 }
}

pub fn primary_monitor_size() -> Size {
    Size {
        width: unsafe { GetSystemMetrics(SM_CXSCREEN) },
        height: unsafe { GetSystemMetrics(SM_CYSCREEN) },
    }
}

pub fn virtual_screen() -> Rectangle {
    if multi_monitor_support() {
        Rectangle {
            x: unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) },
            y: unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) },
            width: unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) },
            height: unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) },
        }
    } else {
        let size = primary_monitor_size();

        Rectangle {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
        }
    }
}

pub fn working_area() -> Rectangle {
    let mut rect = RECT::default();
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut rect as *mut _ as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
    }
    Rectangle::from(rect)
}
