use crate::csharp::screen::Screen;
use crate::csharp::structs::{Point, Rectangle};
use crate::monitor::Monitor;
use log::debug;

#[derive(Debug, Default)]
pub struct NativeMonitorContainer {
    pub monitors: Vec<Monitor>,
    pub focused_monitor: usize,
}

impl NativeMonitorContainer {
    pub fn new() -> Self {
        let mut screens = Screen::all_screens();
        screens.sort_by_key(|s| !s.primary);

        debug!("screens: {:?}", screens);

        let monitors = screens
            .iter()
            .enumerate()
            .map(|(i, s)| Monitor::new(i, s.clone()))
            .collect::<Vec<_>>();

        debug!("monitors: {:?}", monitors);

        Self {
            monitors,
            focused_monitor: 0,
        }
    }

    pub fn num_monitors(&self) -> i32 {
        self.monitors.len() as i32
    }

    pub fn focused_monitor(&self) -> &Monitor {
        self.get_monitor_at_index(self.focused_monitor).unwrap()
    }

    pub fn get_all_monitors(&self) -> &Vec<Monitor> {
        &self.monitors
    }

    pub fn get_monitor_at_index(&self, index: usize) -> Option<&Monitor> {
        self.monitors.get(index)
    }

    pub fn get_monitor_at_point(&self, x: i32, y: i32) -> &Monitor {
        let screen = Screen::from_point(Point { x, y });
        let monitor = self
            .monitors
            .iter()
            .find(|m| m.screen.device_name == screen.device_name);

        match monitor {
            Some(x) => x,
            None => self.monitors.first().unwrap(),
        }
    }

    pub fn get_monitor_at_rect(&self, x: i32, y: i32, width: i32, height: i32) -> &Monitor {
        let screen = Screen::from_rectangle(Rectangle::new(x, y, width, height));
        let monitor = self
            .monitors
            .iter()
            .find(|m| m.screen.device_name == screen.device_name);

        match monitor {
            Some(x) => x,
            None => self.monitors.first().unwrap(),
        }
    }

    pub fn get_next_monitor(&self) -> &Monitor {
        let next_monitor_index = self.focused_monitor + 1;
        let next_monitor = self.get_monitor_at_index(next_monitor_index);

        match next_monitor {
            Some(x) => x,
            None => self.monitors.first().unwrap(),
        }
    }

    pub fn get_previous_monitor(&self) -> &Monitor {
        let previous_monitor_index = self.focused_monitor - 1;
        let previous_monitor = self.get_monitor_at_index(previous_monitor_index);

        match previous_monitor {
            Some(x) => x,
            None => self.monitors.last().unwrap(),
        }
    }
}
