use crate::classes::window_location::WindowLocation;
use crate::classes::window_state::WindowState;
use crate::csharp::structs::Rectangle;
use crate::helpers::win32_helpers;
use anyhow::{bail, Result};
use log::{debug, error, trace, warn};
use std::ffi::c_void;
use std::fmt::Display;
use std::mem::size_of;
use std::path::Path;
use windows::Win32::Foundation::{HWND, MAX_PATH, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsZoomed, SendNotifyMessageW,
    SetForegroundWindow, ShowWindow, SC_CLOSE, SW_HIDE, SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED,
    SW_SHOWNOACTIVATE, WM_SYSCOMMAND,
};

const IGNORE_WINDOW_CLASSES: [&str; 10] = [
    "TaskManagerWindow",
    "MSCTFIME UI",
    "SHELLDLL_DefView",
    "LockScreenBackstopFrame",
    "Progman",
    "Shell_TrayWnd",
    "WorkerW",
    "Shell_SecondaryTrayWnd",
    "Windows.UI.Composition.DesktopWindowContentBridge",
    "IME",
];

const IGNORE_PROGRAMS: [&str; 3] = [
    "StartMenuExperienceHost.exe",
    "SearchHost.exe",
    "ApplicationFrameHost.exe",
];

const IGNORE_WINDOW_TITLES: [&str; 1] = ["Windows Input Experience"];

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Window {
    // Private
    process_id: u32,
    process_name: String,
    process_file_name: String,
    did_manual_hide: bool,

    // Public
    pub handle: isize,
    pub is_mouse_moving: bool,
}

impl Window {
    pub fn new(handle: isize) -> Result<Self> {
        let hwnd = HWND(handle);

        // Filtering
        {
            let class = Self::class_from_hwnd(hwnd);
            if IGNORE_WINDOW_CLASSES.contains(&class.as_str()) {
                let msg = format!("Filtered class: {}", &class);
                trace!("{}", &msg);
                bail!("{}", &msg);
            }

            let title = Self::title_from_hwnd(hwnd);
            if title.is_empty() {
                let msg = "Filtered empty title";
                trace!("{}", &msg);
                bail!("{}", &msg);
            }
            if IGNORE_WINDOW_TITLES.contains(&title.as_str()) {
                let msg = format!("Filtered title: {}", &title);
                trace!("{}", &msg);
                bail!("{}", &msg);
            }
        }

        let mut process_id = 0;

        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };

        if process_id == 0 {
            error!("Failed to get process id");
            bail!("Failed to get process id");
        }

        let process_handle = match unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                process_id,
            )
        } {
            Ok(handle) => handle,
            Err(e) => {
                let msg = format!(
                    "Failed to open process [hwnd: 0x{:X} | pid: {}]: {:?}",
                    handle,
                    process_id,
                    e.message(),
                );
                warn!("{}", msg);
                bail!("{}", msg);
            }
        };

        trace!("process_handle: {:?}", process_handle);

        let mut exe_path_bytes: Vec<u16> = vec![0; MAX_PATH as usize];
        let exe_path_length =
            unsafe { K32GetModuleFileNameExW(process_handle, None, &mut exe_path_bytes) };
        let process_name = String::from_utf16_lossy(&exe_path_bytes[..exe_path_length as usize]);

        trace!("process_name: {:?}", process_name);

        // Get file name
        let process_file_name = if let Some(file_name) = Path::new(&process_name).file_name() {
            file_name.to_string_lossy().to_string()
        } else {
            "--NA--".to_string()
        };

        trace!("process_file_name: {:?}", process_file_name);

        // Filtering
        {
            if IGNORE_PROGRAMS.contains(&process_file_name.as_str()) {
                let msg = format!("Filtered program: {}", &process_file_name);
                trace!("{}", &msg);
                bail!("{}", &msg);
            }
        }

        Ok(Self {
            handle,
            process_id,
            process_name,
            process_file_name,
            did_manual_hide: false,
            is_mouse_moving: false,
        })
    }

    pub fn did_manual_hide(&self) -> bool {
        self.did_manual_hide
    }

    pub fn title(&self) -> String {
        Self::title_from_hwnd(self.hwnd())
    }

    pub fn title_from_hwnd(hwnd: HWND) -> String {
        let length = unsafe { GetWindowTextLengthW(hwnd) };
        let mut bytes: Vec<u16> = vec![0; length as usize + 1];
        let _ = unsafe { GetWindowTextW(hwnd, bytes.as_mut_slice()) };

        String::from_utf16_lossy(&bytes[..length as usize])
    }

    pub fn hwnd(&self) -> HWND {
        HWND(self.handle)
    }

    pub fn class(&self) -> String {
        Self::class_from_hwnd(self.hwnd())
    }

    pub fn class_from_hwnd(hwnd: HWND) -> String {
        let mut class: Vec<u16> = vec![0; MAX_PATH as usize];
        unsafe {
            GetClassNameW(hwnd, class.as_mut_slice());
        }

        let null_pos = class.iter().position(|&c| c == 0).unwrap_or(class.len());

        String::from_utf16_lossy(&class[..null_pos])
    }

    pub fn location(&self) -> WindowLocation {
        let mut rect: RECT = RECT::default();
        unsafe {
            let _ = GetWindowRect(self.hwnd(), &mut rect); // TODO: Look into this
        }

        let mut state = WindowState::Normal;
        if self.is_minimized() {
            state = WindowState::Minimized;
        } else if self.is_maximized() {
            state = WindowState::Maximized;
        }

        WindowLocation {
            x: rect.left,
            y: rect.right,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
            state,
        }
    }

    #[allow(dead_code)]
    pub fn offset(&self) -> Rectangle {
        let handle = HWND(self.handle);

        let mut rect1: RECT = RECT::default();
        unsafe {
            let _ = GetWindowRect(handle, &mut rect1);
        }

        let x1 = rect1.left;
        let y1 = rect1.top;
        let width1 = rect1.right - rect1.left;
        let height1 = rect1.bottom - rect1.top;

        let mut rect2 = RECT::default();
        let size = size_of::<RECT>() as u32;
        unsafe {
            let rect_ptr = &mut rect2 as *mut _ as *mut c_void;
            let _ = DwmGetWindowAttribute(handle, DWMWA_EXTENDED_FRAME_BOUNDS, rect_ptr, size);
        }

        let x2 = rect2.left;
        let y2 = rect2.top;
        let width2 = rect2.right - rect2.left;
        let height2 = rect2.bottom - rect2.top;

        let x = x1 - x2;
        let y = y1 - y2;
        let width = width1 - width2;
        let height = height1 - height2;

        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    #[allow(dead_code)]
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn process_file_name(&self) -> &str {
        &self.process_file_name
    }

    pub fn process_name(&self) -> &str {
        &self.process_name
    }

    pub fn can_layout(&self) -> bool {
        let hwnd = self.hwnd();

        self.did_manual_hide
            || win32_helpers::is_cloaked(hwnd)
                && win32_helpers::is_app_window(hwnd)
                && win32_helpers::is_alt_tab_window(hwnd)
    }

    pub fn is_focused(&self) -> bool {
        let foreground_window = unsafe { GetForegroundWindow() };
        self.hwnd() == foreground_window
    }

    pub fn is_minimized(&self) -> bool {
        unsafe { IsIconic(self.hwnd()).as_bool() }
    }

    pub fn is_maximized(&self) -> bool {
        unsafe { IsZoomed(self.hwnd()).as_bool() }
    }

    #[allow(dead_code)]
    pub fn is_fullscreen(&self) -> bool {
        unsafe {
            let mut window_rect = RECT::default();
            if GetWindowRect(self.hwnd(), &mut window_rect).is_ok() {
                let monitor = MonitorFromWindow(self.hwnd(), MONITOR_DEFAULTTONEAREST);
                let mut monitor_info = MONITORINFO {
                    cbSize: size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };

                if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
                    let screen_rect = monitor_info.rcMonitor;
                    return screen_rect.left == window_rect.left
                        && screen_rect.right == window_rect.right
                        && screen_rect.top == window_rect.top
                        && screen_rect.bottom == window_rect.bottom;
                }
            }
        }

        false
    }

    #[allow(dead_code)]
    pub fn focus(&self) {
        if !self.is_focused() {
            unsafe {
                trace!("[{}] :: Focus", self.title());
                SetForegroundWindow(self.hwnd());
            }
        }
    }

    #[allow(dead_code)]
    pub fn hide(&mut self) {
        trace!("[{}] :: Hide", self.title());

        if self.can_layout() {
            self.did_manual_hide = true;
        }

        unsafe {
            ShowWindow(self.hwnd(), SW_HIDE);
        }
    }

    pub fn show_normal(&mut self) {
        self.did_manual_hide = false;

        trace!("[{}] :: ShowNormal", self.title());

        unsafe {
            ShowWindow(self.hwnd(), SW_SHOWNOACTIVATE);
        }
    }

    pub fn show_maximized(&mut self) {
        self.did_manual_hide = false;

        trace!("[{}] :: ShowMaximized", self.title());

        unsafe {
            ShowWindow(self.hwnd(), SW_SHOWMAXIMIZED);
        }
    }

    pub fn show_minimized(&mut self) {
        self.did_manual_hide = false;

        trace!("[{}] :: ShowMinimized", self.title());

        unsafe {
            ShowWindow(self.hwnd(), SW_SHOWMINIMIZED);
        }
    }

    #[allow(dead_code)]
    pub fn show_in_current_state(&mut self) {
        if self.is_minimized() {
            self.show_minimized();
        } else if self.is_maximized() {
            self.show_maximized();
        } else {
            self.show_normal();
        }

        // TODO: WindowUpdated?.Invoke(this);
    }

    #[allow(dead_code)]
    pub fn bring_to_top(&self) {
        unsafe {
            let _ = BringWindowToTop(self.hwnd());
        }

        // TODO: WindowUpdated?.Invoke(this);
    }

    pub fn close(&self) {
        debug!("[{}] :: Close", self.title());

        unsafe {
            let _ = SendNotifyMessageW(self.hwnd(), WM_SYSCOMMAND, WPARAM(SC_CLOSE as usize), None);
        }
    }

    #[allow(dead_code)]
    pub fn notify_updated(&mut self) {
        // TODO: WindowUpdated?.Invoke(this);
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}][{}][{}][{}]",
            self.handle,
            self.title(),
            self.class(),
            self.process_name,
        )
    }
}
