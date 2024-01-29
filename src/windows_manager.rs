use crate::keys::Keyboard;
use crate::windows_defer_pos_handle::WindowsDeferPosHandle;
use crate::windows_window::WindowsWindow;
use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, error, info, trace};
use std::collections::{BTreeMap, HashMap};
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

#[derive(Debug, Default)]
pub struct WindowsManager {
    pub windows: BTreeMap<isize, WindowsWindow>,
    pub floating: HashMap<isize, bool>,
    mouse_move_lock: Mutex<()>,
    mouse_move_window: Option<isize>,
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

        let mut windows: Vec<HWND> = vec![];

        unsafe {
            let _ = EnumWindows(
                Some(Self::enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<HWND> as isize),
            );
        };

        for hwnd in windows.into_iter() {
            if crate::win32_helpers::is_app_window(hwnd) {
                self.register_window(hwnd.0);
            }
        }

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

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, userdata: LPARAM) -> BOOL {
        let windows = &mut *(userdata.0 as *mut Vec<HWND>);
        windows.push(hwnd);

        TRUE
    }

    unsafe fn defer_windows_pos(count: i32) -> Result<WindowsDeferPosHandle> {
        let info = BeginDeferWindowPos(count)?;

        Ok(WindowsDeferPosHandle::new(info))
    }

    pub fn toggle_focused_window_tiling(&mut self) {
        let hwnd_option = self
            .windows
            .values()
            .find(|w| w.is_focused())
            .map(|window| window.handle());

        if let Some(hwnd) = hwnd_option {
            if self.floating.contains_key(&hwnd) {
                self.floating.remove(&hwnd);
                self.handle_window_add(hwnd, false);
            } else {
                if let Some(floating) = self.floating.get_mut(&hwnd) {
                    *floating = true; // TODO: Check this
                }

                self.handle_window_remove(hwnd);

                if let Some(window) = self.windows.values().find(|w| w.handle() == hwnd) {
                    window.bring_to_top();
                }
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
        hwnd: HWND,
        id_object: i32,
        id_child: i32,
        _id_event_thread: u32,
        _dwms_event_time: u32,
    ) {
        let hwnd = hwnd.0;

        if Self::event_window_is_valid(id_child, id_object, hwnd) {
            let mut instance = INSTANCE.lock().unwrap();

            match event_type {
                EVENT_OBJECT_SHOW => instance.register_window(hwnd),
                EVENT_OBJECT_DESTROY => instance.unregister_window(hwnd),
                EVENT_OBJECT_CLOAKED => instance.update_window(hwnd, WindowUpdateType::Hide),
                EVENT_OBJECT_UNCLOAKED => instance.update_window(hwnd, WindowUpdateType::Show),
                EVENT_SYSTEM_MINIMIZESTART => {
                    instance.update_window(hwnd, WindowUpdateType::MinimizeStart)
                }
                EVENT_SYSTEM_MINIMIZEEND => {
                    instance.update_window(hwnd, WindowUpdateType::MinimizeEnd)
                }
                EVENT_SYSTEM_FOREGROUND => {
                    instance.update_window(hwnd, WindowUpdateType::Foreground)
                }
                EVENT_SYSTEM_MOVESIZESTART => instance.start_move_window(hwnd),
                EVENT_SYSTEM_MOVESIZEEND => instance.end_move_window(hwnd),
                EVENT_OBJECT_LOCATIONCHANGE => instance.window_move(hwnd),
                _ => error!("event_callback | event_type: UNKNOWN({:?})", event_type),
            }
        }
    }

    fn event_window_is_valid(id_child: i32, id_object: i32, hwnd: isize) -> bool {
        id_child == 0 && id_object == 0 && hwnd != 0
    }

    fn register_window(&mut self, hwnd: isize) {
        if self.windows.contains_key(&hwnd) {
            debug!("register_window | handle: 0x{:X} already registered", &hwnd);
            return;
        }

        debug!("register_window | handle: 0x{:X} not registered", &hwnd);

        match WindowsWindow::new(hwnd) {
            Ok(window) => self.windows.insert(hwnd, window),
            Err(e) => {
                error!("register_window | Failed to register window: {:?}", e);
                None
            }
        };
    }

    fn unregister_window(&mut self, hwnd: isize) {
        if !self.windows.contains_key(&hwnd) {
            debug!("unregister_window | handle: 0x{:X} not registered", &hwnd);
            return;
        }

        debug!("unregister_window | handle: 0x{:X} registered", &hwnd);

        self.windows.remove(&hwnd);
        // TODO: HandleWindowRemove(window);
    }

    fn update_window(&mut self, hwnd: isize, update_type: WindowUpdateType) {
        if update_type == WindowUpdateType::Show && self.windows.contains_key(&hwnd) {
            if let Some(_window) = self.windows.get(&hwnd) {};
            // TODO: WindowUpdated?.Invoke(window, update_type);
        } else if update_type == WindowUpdateType::Show {
            self.register_window(hwnd);
        } else if update_type == WindowUpdateType::Hide && self.windows.contains_key(&hwnd) {
            if let Some(window) = self.windows.get(&hwnd) {
                if !window.did_manual_hide() {
                    self.unregister_window(hwnd);
                } else {
                    // TODO: WindowUpdated?.Invoke(window, update_type);
                }
            };
        } else if self.windows.contains_key(&hwnd) {
            if let Some(_window) = self.windows.get(&hwnd) {};
            // TODO: WindowUpdated?.Invoke(window, update_type);
        }
    }

    fn start_move_window(&mut self, hwnd: isize) {
        if self.windows.contains_key(&hwnd) {
            self.handle_window_move_start(hwnd);
            // TODO: WindowUpdated?.Invoke(window, WindowUpdateType.MoveStart);
            debug!("start_move_window | handle: 0x{:X}", &hwnd);
        }
    }

    fn end_move_window(&mut self, hwnd: isize) {
        if self.windows.contains_key(&hwnd) {
            self.handle_window_move_end();
            // TODO: WindowUpdated?.Invoke(window, WindowUpdateType.MoveEnd);
            debug!("end_move_window | handle: 0x{:X}", &hwnd);
        }
    }

    fn window_move(&self, hwnd: isize) {
        if let Some(window) = self.windows.get(&hwnd) {
            if window.can_layout() {
                // TODO: WindowUpdated?.Invoke(_windows[handle], WindowUpdateType.Move);
            }
        }
    }

    fn handle_window_move_start(&mut self, handle: isize) {
        if let Some(current_handle) = self.mouse_move_window {
            if let Some(window) = self.windows.get_mut(&current_handle) {
                window.is_mouse_moving = false;
            }
        }

        self.mouse_move_window = Some(handle);

        if let Some(window) = self.windows.get_mut(&handle) {
            window.is_mouse_moving = true;
        }
    }

    fn handle_window_move_end(&mut self) {
        let _lock = self.mouse_move_lock.lock().unwrap();
        if let Some(ref mut handle) = self.mouse_move_window {
            if let Some(window) = self.windows.get_mut(handle) {
                window.is_mouse_moving = false;
            }
            self.mouse_move_window = None;
        }
    }

    fn handle_window_add(&mut self, _handle: isize, _first_create: bool) {
        // TODO: WindowCreated?.Invoke(window, firstCreate);
    }

    fn handle_window_remove(&mut self, _handle: isize) {
        // TODO: WindowDestroyed?.Invoke(window);
    }
}

#[derive(Debug, PartialEq)]
pub enum WindowUpdateType {
    Show,
    Hide,
    MinimizeStart,
    MinimizeEnd,
    Foreground,
    MoveStart,
    MoveEnd,
    Move,
}
