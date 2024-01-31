// TODO

use crate::classes::window_order::WindowOrder;
use crate::layout_engines::dwindle_layout_engine::DwindleLayoutEngine;
use crate::layout_engines::focus_layout_engine::FocusLayoutEngine;
use crate::layout_engines::full_layout_engine::FullLayoutEngine;
use crate::layout_engines::grid_layout_engine::GridLayoutEngine;
use crate::layout_engines::*;
use crate::manager::WindowUpdateType;
use crate::window::Window;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug)]
pub struct Workspace {
    pub name: String,
    pub windows: Vec<Window>,
    pub managed_windows: Vec<Window>,
    pub layout_engine: LayoutEngineType,
    last_focused: Option<Window>,

    layout_engines: HashMap<LayoutEngineType, Box<dyn LayoutEngine>>,
}

impl Workspace {
    pub fn new(name: &str) -> Self {
        let mut layout_engines: HashMap<LayoutEngineType, Box<dyn LayoutEngine>> = HashMap::new();
        layout_engines.insert(
            LayoutEngineType::Dwindle,
            Box::new(DwindleLayoutEngine::new()),
        );
        layout_engines.insert(LayoutEngineType::Focus, Box::new(FocusLayoutEngine::new()));
        layout_engines.insert(LayoutEngineType::Full, Box::new(FullLayoutEngine::new()));
        layout_engines.insert(LayoutEngineType::Grid, Box::new(GridLayoutEngine::new()));
        //layout_engines.insert(LayoutEngineType::Focus, Box::new(PanelLayoutEngine::new()));
        //layout_engines.insert(LayoutEngineType::Focus, Box::new(TallLayoutEngine::new()));

        Self {
            name: name.to_string(),
            windows: Vec::new(),
            managed_windows: Vec::new(),
            layout_engine: LayoutEngineType::default(),
            last_focused: None,
            layout_engines,
        }
    }

    pub fn layout_name(&self) -> String {
        format!("{:?}", self.layout_engine)
    }

    pub fn focused_window(&mut self) -> Option<&Window> {
        self.windows.iter().find(|w| w.is_focused())
    }

    pub fn add_window(&mut self, window: &Window, window_order: WindowOrder, layout: bool) {
        {
            if self.last_focused.is_none() || window.is_focused() {
                self.last_focused = Some(window.clone());
            }

            match window_order {
                WindowOrder::NewWindowsLast => self.windows.push(window.clone()),
                WindowOrder::NewWindowsFirst => self.windows.insert(0, window.clone()),
            }
        }

        if layout {
            self.do_layout();
        }
    }

    pub fn remove_window(&mut self, window: &Window, layout: bool) {
        let length = self.managed_windows.len();

        if let Some(last_focused) = &self.last_focused {
            if last_focused == window {
                let next_index = self
                    .managed_windows
                    .iter()
                    .position(|w| w == window)
                    .and_then(|i| {
                        if length > 1 {
                            Some((i + 1) % length)
                        } else {
                            None
                        }
                    });

                self.last_focused = next_index.and_then(|ni| self.managed_windows.get(ni).cloned());
            }
        }

        self.windows.retain(|w| w != window);

        if layout {
            self.do_layout();
        }
    }

    pub fn update_window(
        &mut self,
        window: &Window,
        window_update_type: WindowUpdateType,
        layout: bool,
    ) {
        // DEFAULT layout: true
        if window_update_type == WindowUpdateType::Foreground {
            self.last_focused = Some(window.clone());
        }

        if layout {
            self.do_layout();
        }
    }

    pub fn close_focus_window(&mut self) {
        if let Some(window) = self.focused_window() {
            window.close();
        }
    }

    pub fn previous_layout_engine(&mut self) {
        self.layout_engine = self.layout_engine.previous();
    }

    pub fn next_layout_engine(&mut self) {
        self.layout_engine = self.layout_engine.next();
    }

    pub fn reset_layout(&mut self) {
        self.get_layout_engine().reset_primary_area();
        self.do_layout();
    }

    fn swap_windows(&mut self, left: &Window, right: &Window) {
        {
            let left_idx = self.windows.iter().position(|w| w == left);
            let right_idx = self.windows.iter().position(|w| w == right);

            if let (Some(l_idx), Some(r_idx)) = (left_idx, right_idx) {
                self.windows.swap(l_idx, r_idx);
                self.windows[r_idx].notify_updated();
                self.windows[l_idx].notify_updated();
            }
        }

        self.do_layout();
    }

    pub fn do_layout(&mut self) {
        if let Some(_window) = &self.focused_window() {
            // TODO: OnLayoutCompleted?.Invoke(this);
            return;
        }

        // let _windows = self.managed_windows.clone();

        // TODO: _context.Enabled
        if true {}
    }

    fn get_layout_engine(&mut self) -> &mut Box<dyn LayoutEngine> {
        self.layout_engines.get_mut(&self.layout_engine).unwrap()
    }
}

impl Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
