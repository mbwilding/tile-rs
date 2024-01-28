use crate::monitor::Monitor;
use std::collections::HashMap;

#[derive(Debug)]
pub struct native_monitor_container {
    monitors: Vec<Monitor>,
    monitor_map: HashMap<Monitor, u32>,
}

impl native_monitor_container {
    pub fn new() -> Self {
        let screens = 0; // TODO
        let monitors = Vec::with_capacity(screens as usize); // TODO
        let monitor_map = HashMap::with_capacity(screens as usize); // TODO

        let primary_monitor = Monitor::new(0);

        Self {
            monitors,
            monitor_map,
        }
    }

    pub fn get_monitor(&self, index: u32) -> Option<&Monitor> {
        self.monitors.get(index as usize)
    }

    pub fn get_monitor_count(&self) -> u32 {
        self.monitors.len() as u32
    }
}
