use crate::structs::{Point, Rectangle};
use crate::system_information;
use crate::system_information::multi_monitor_support;
use std::ffi::OsStr;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, FALSE, LPARAM, POINT, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    CreateDCW, DeleteDC, EnumDisplayMonitors, GetDeviceCaps, GetMonitorInfoW, MonitorFromPoint,
    MonitorFromRect, BITSPIXEL, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
    MONITOR_DEFAULTTONEAREST, PLANES,
};
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;

const PRIMARY_MONITOR: isize = 0xBAADF00D;

struct MonitorData {
    pub hmonitor: HMONITOR,
    pub monitor_info: MONITORINFOEXW,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Screen {
    pub bounds: Rectangle,
    pub primary: bool,
    pub device_name: String,
    pub hmonitor: isize,
    pub bit_depth: i32,
}

impl Screen {
    pub fn new(monitor: isize, hdc: Option<HDC>) -> Screen {
        let mut screen_dc = hdc;

        let mut bounds = Rectangle::default();
        let mut primary = false;
        let mut device_name = String::new();
        let hmonitor = HMONITOR(monitor);
        let mut bit_depth = 0;

        if !multi_monitor_support() || monitor == PRIMARY_MONITOR {
            // Single monitor system
            bounds = system_information::virtual_screen();
            primary = true;
            device_name.push_str("DISPLAY");
        } else {
            // Multi monitor system

            // TODO: [Wait for update in Windows crate for ExW]
            //device_name.push_str(
            //    &Self::all_screens()
            //        .iter()
            //        .find(|s| s.hmonitor == monitor)
            //        .unwrap()
            //        .device_name,
            //);

            let mut info = MONITORINFOEXW {
                monitorInfo: MONITORINFO {
                    cbSize: size_of::<MONITORINFOEXW>() as u32,
                    ..Default::default()
                },
                szDevice: Default::default(),
            };

            unsafe { GetMonitorInfoW(HMONITOR(monitor), ptr::addr_of!(info) as *mut MONITORINFO) };

            device_name.push_str(&String::from_utf16_lossy(&info.szDevice));
            bounds = Rectangle::from(info.monitorInfo.rcMonitor);
            primary = (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;

            let pwsz_driver = OsStr::new(&device_name)
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();

            let pwsz_driver_ptr = PCWSTR(pwsz_driver.as_ptr() as *mut _);

            if hdc.is_none() {
                screen_dc = Some(unsafe { CreateDCW(pwsz_driver_ptr, None, None, None) });
            }
        }

        if let Some(hdc) = hdc {
            bit_depth = unsafe { GetDeviceCaps(hdc, BITSPIXEL) };
            bit_depth *= unsafe { GetDeviceCaps(hdc, PLANES) };
        }

        if hdc != screen_dc {
            if let Some(screen_dc) = screen_dc {
                unsafe { DeleteDC(screen_dc) };
            }
        }

        Screen {
            bounds,
            primary,
            device_name,
            hmonitor: hmonitor.0,
            bit_depth,
        }
    }

    pub fn all_screens() -> Vec<Screen> {
        unsafe {
            if multi_monitor_support() {
                let mut monitor_datas: Vec<MonitorData> = Vec::new();
                let userdata = &mut monitor_datas as *mut _ as isize;

                EnumDisplayMonitors(
                    None,
                    None,
                    Some(Self::enumerate_monitors_callback),
                    LPARAM(userdata),
                );

                return monitor_datas
                    .iter()
                    .map(|monitor_data| Screen::new(monitor_data.hmonitor.0, None))
                    .collect();
            }

            vec![Screen::new(PRIMARY_MONITOR, None)]
        }
    }

    unsafe extern "system" fn enumerate_monitors_callback(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        userdata: LPARAM,
    ) -> BOOL {
        let monitors = &mut *(userdata.0 as *mut Vec<MonitorData>);
        let info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            szDevice: Default::default(),
        };

        if GetMonitorInfoW(hmonitor, ptr::addr_of!(info) as *mut MONITORINFO).as_bool() {
            monitors.push(MonitorData {
                hmonitor,
                monitor_info: info,
            });

            return TRUE;
        }

        FALSE
    }

    pub fn working_area(&self) -> Rectangle {
        if !multi_monitor_support() || self.hmonitor == PRIMARY_MONITOR {
            system_information::working_area()
        } else {
            let mut monitor_info = MONITORINFO {
                cbSize: size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            unsafe {
                let _ = GetMonitorInfoW(HMONITOR(self.hmonitor), &mut monitor_info);
            };

            Rectangle::from(monitor_info.rcWork)
        }
    }

    pub fn primary_screen() -> Screen {
        if multi_monitor_support() {
            // TODO: unwrap
            Screen::all_screens()
                .into_iter()
                .find(|s| s.primary)
                .unwrap()
        } else {
            Screen::new(PRIMARY_MONITOR, None)
        }
    }

    pub fn from_point(point: Point) -> Screen {
        if multi_monitor_support() {
            let hmonitor = unsafe {
                MonitorFromPoint(
                    POINT {
                        x: point.x,
                        y: point.y,
                    },
                    MONITOR_DEFAULTTONEAREST,
                )
            };
            Screen::new(hmonitor.0, None)
        } else {
            Screen::new(PRIMARY_MONITOR, None)
        }
    }

    pub fn from_rectangle(rect: Rectangle) -> Screen {
        if multi_monitor_support() {
            let rect = RECT {
                left: rect.x,
                top: rect.y,
                right: rect.right(),
                bottom: rect.bottom(),
            };
            let hmonitor = unsafe { MonitorFromRect(&rect as *const _, MONITOR_DEFAULTTONEAREST) };
            Screen::new(hmonitor.0, None)
        } else {
            Screen::new(PRIMARY_MONITOR, None)
        }
    }
}
