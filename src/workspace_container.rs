use crate::classes::monitor::Monitor;
use crate::workspace::Workspace;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct WorkspaceContainer {
    workspaces: Vec<Workspace>,
    workspace_map: HashMap<Workspace, usize>,
    monitor_to_workspace_map: HashMap<Monitor, usize>,
    last_monitor: HashMap<usize, Monitor>,
}

impl WorkspaceContainer {
    pub fn create_workspaces(&mut self, names: Vec<&str>) {
        for name in names {
            self.create_workspace(name);
        }
    }

    pub fn create_workspace(&mut self, _name: &str) {
        // TODO
    }
}
