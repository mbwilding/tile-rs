use crate::keys::Keyboard;
use crate::windows_defer_pos_handle::WindowsDeferPosHandle;
use crate::windows_window::WindowsWindow;
use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, error, info, trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HMODULE, HWND, LPARAM, LRESULT, TRUE, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    BeginDeferWindowPos, CallNextHookEx, DispatchMessageW, EnumWindows, GetMessageW,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, EVENT_OBJECT_CLOAKED,
    EVENT_OBJECT_DESTROY, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
    EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WINEVENT_OUTOFCONTEXT, WM_LBUTTONUP,
};

lazy_static! {
    pub static ref INSTANCE: Arc<Mutex<WindowsManager>> =
        Arc::new(Mutex::new(WindowsManager::default()));
}

lazy_static::lazy_static! {
    static ref WINDOW_COLLECTOR: Mutex<Vec<HWND>> = Mutex::new(Vec::new());
}

#[derive(Debug, Default)]
pub struct WindowsManager {
    pub windows: HashMap<isize, WindowsWindow>,
    pub floating: HashMap<WindowsWindow, bool>,
}

impl WindowsManager {
    pub fn init(&mut self) {
        info!("Initializing hooks");

        let module_handle = unsafe {
            GetModuleHandleW(PCWSTR::null()).unwrap_or_else(|e| {
                error!("Failed GetModuleHandleW: {:?}", e);
                std::process::exit(69);
            })
        };

        let win_event_hooks = [
            unsafe {
                Self::register_event_hook(EVENT_OBJECT_DESTROY, EVENT_OBJECT_SHOW, module_handle)
            },
            unsafe {
                Self::register_event_hook(
                    EVENT_OBJECT_CLOAKED,
                    EVENT_OBJECT_UNCLOAKED,
                    module_handle,
                )
            },
            unsafe {
                Self::register_event_hook(
                    EVENT_SYSTEM_MINIMIZESTART,
                    EVENT_SYSTEM_MINIMIZEEND,
                    module_handle,
                )
            },
            unsafe {
                Self::register_event_hook(
                    EVENT_SYSTEM_MOVESIZESTART,
                    EVENT_SYSTEM_MOVESIZEEND,
                    module_handle,
                )
            },
            unsafe {
                Self::register_event_hook(
                    EVENT_SYSTEM_FOREGROUND,
                    EVENT_SYSTEM_FOREGROUND,
                    module_handle,
                )
            },
            unsafe {
                Self::register_event_hook(
                    EVENT_OBJECT_LOCATIONCHANGE,
                    EVENT_OBJECT_LOCATIONCHANGE,
                    module_handle,
                )
            },
        ];

        let _event_mouse = unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(Self::mouse_callback), module_handle, 0)
                .unwrap_or_else(|e| {
                    error!("Failed SetWindowsHookExW[Mouse]: {:?}", e);
                    std::process::exit(69);
                })
        };

        let _event_keyboard = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::keyboard_callback),
                module_handle,
                0,
            )
            .unwrap_or_else(|e| {
                error!("Failed SetWindowsHookExW[Keyboard]: {:?}", e);
                std::process::exit(69);
            })
        };

        info!("Initialized hooks");

        let mut windows: Vec<isize> = vec![];

        let _ = unsafe {
            EnumWindows(
                Some(Self::enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<isize> as isize),
            )
            .ok()
        };

        for hwnd in WINDOW_COLLECTOR.lock().unwrap().iter() {
            if crate::win32_helpers::is_app_window(hwnd.to_owned()) {
                self.register_window(hwnd.0);
            }
        }
        WINDOW_COLLECTOR.lock().unwrap().clear();

        let mut message = MSG::default();

        // Handle close
        std::thread::spawn(move || {
            loop {
                unsafe {
                    if GetMessageW(&mut message, HWND(0), 0, 0).as_bool() {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    } else {
                        break;
                    }
                }
            }

            unsafe {
                UnhookWindowsHookEx(_event_mouse).unwrap_or_else(|e| {
                    error!("Failed UnhookWindowsHookEx: {:?}", e);
                })
            };

            for hooks in win_event_hooks.into_iter() {
                unsafe {
                    if !UnhookWinEvent(hooks).as_bool() {
                        error!("Failed UnhookWinEvent");
                    }
                }
            }
        });
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut collector = WINDOW_COLLECTOR.lock().unwrap();
        collector.push(hwnd);

        TRUE
    }

    unsafe fn defer_windows_pos(count: i32) -> Result<WindowsDeferPosHandle> {
        let info = BeginDeferWindowPos(count)?;

        Ok(WindowsDeferPosHandle::new(info))
    }

    pub fn toggle_focused_window_tiling(&mut self) {
        let window = self.windows.values().find(|w| w.is_focused());

        if let Some(window) = window {
            if self.floating.contains_key(window) {
                self.floating.remove(window);
                // TODO: HandleWindowAdd(window, false);
            } else {
                *self.floating.get_mut(window).unwrap() = true; // TODO: Fix unwrap
                                                                // TODO: HandleWindowRemove(window);
                window.bring_to_top();
            }
        }
    }

    unsafe fn register_event_hook(
        event_min: u32,
        event_max: u32,
        module_handle: HMODULE,
    ) -> HWINEVENTHOOK {
        SetWinEventHook(
            event_min,
            event_max,
            module_handle,
            Some(Self::event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    }

    unsafe extern "system" fn mouse_callback(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if w_param.0 == WM_LBUTTONUP as usize {
            trace!("mouse_callback | WM_LBUTTONUP");
            // TODO: HandleWindowMoveEnd();
        }

        CallNextHookEx(None, n_code, w_param, l_param)
    }

    unsafe extern "system" fn keyboard_callback(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if let Some(_keyboard) = Keyboard::new(n_code, w_param, l_param) {
            // TODO: Do something, perhaps crossbeam_channel
        };

        CallNextHookEx(None, n_code, w_param, l_param)
    }

    unsafe extern "system" fn event_callback(
        _h_win_event_hook: HWINEVENTHOOK,
        event_type: u32,
        window_handle: HWND,
        id_object: i32,
        id_child: i32,
        _id_event_thread: u32,
        _dwms_event_time: u32,
    ) {
        if Self::event_window_is_valid(id_child, id_object, window_handle) {
            let hwnd = window_handle.0;
            let mut instance = INSTANCE.lock().unwrap();

            match event_type {
                EVENT_OBJECT_SHOW => instance.register_window(hwnd),
                EVENT_OBJECT_DESTROY => instance.unregister_window(hwnd),
                EVENT_OBJECT_CLOAKED => trace!("event_callback | EVENT_OBJECT_CLOAKED"),
                EVENT_OBJECT_UNCLOAKED => trace!("event_callback | EVENT_OBJECT_UNCLOAKED"),
                EVENT_SYSTEM_MINIMIZESTART => trace!("event_callback | EVENT_SYSTEM_MINIMIZESTART"),
                EVENT_SYSTEM_MINIMIZEEND => trace!("event_callback | EVENT_SYSTEM_MINIMIZEEND"),
                EVENT_SYSTEM_FOREGROUND => trace!("event_callback | EVENT_SYSTEM_FOREGROUND"),
                EVENT_SYSTEM_MOVESIZESTART => trace!("event_callback | EVENT_SYSTEM_MOVESIZESTART"),
                EVENT_SYSTEM_MOVESIZEEND => trace!("event_callback | EVENT_SYSTEM_MOVESIZEEND"),
                EVENT_OBJECT_LOCATIONCHANGE => {
                    trace!("event_callback | EVENT_OBJECT_LOCATIONCHANGE")
                }
                _ => trace!("event_callback | event_type: UNKNOWN({:?})", event_type),
            }
        }
    }

    fn event_window_is_valid(id_child: i32, id_object: i32, window_handle: HWND) -> bool {
        id_child == 0 && id_object == 0 && window_handle.0 != 0
    }

    fn register_window(&mut self, handle: isize) {
        if self.windows.contains_key(&handle) {
            debug!(
                "register_window | handle: 0x{:X} already registered",
                handle
            );
            return;
        }

        debug!("register_window | handle: 0x{:X} not registered", handle);

        match WindowsWindow::new(handle) {
            Ok(window) => self.windows.insert(handle, window),
            Err(e) => {
                error!("register_window | Failed to register window: {:?}", e);
                None
            }
        };
    }

    fn unregister_window(&mut self, handle: isize) {
        if !self.windows.contains_key(&handle) {
            debug!("unregister_window | handle: 0x{:X} not registered", handle);
            return;
        }

        debug!("unregister_window | handle: 0x{:X} registered", handle);

        self.windows.remove(&handle);
        // TODO: HandleWindowRemove(window);
    }
}
