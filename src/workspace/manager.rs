use std::collections::HashMap;
use std::path::PathBuf;

use chrono::Utc;
use uuid::Uuid;

use crate::workspace::model::{Workspace, WorkspaceStatus};

pub struct WorkspaceManager {
    workspaces: HashMap<Uuid, Workspace>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
        }
    }

    pub fn create(&mut self, name: String, root_dir: PathBuf) -> Uuid {
        let id = Uuid::new_v4();
        let workspace = Workspace {
            id,
            name,
            root_dir,
            env_vars: HashMap::new(),
            status: WorkspaceStatus::Active,
            created_at: Utc::now(),
        };
        self.workspaces.insert(id, workspace);
        id
    }

    pub fn get(&self, id: Uuid) -> Option<&Workspace> {
        self.workspaces.get(&id)
    }

    pub fn list(&self) -> Vec<&Workspace> {
        self.workspaces.values().collect()
    }

    pub fn archive(&mut self, id: Uuid) {
        if let Some(ws) = self.workspaces.get_mut(&id) {
            ws.status = WorkspaceStatus::Archived;
        }
    }

    pub fn delete(&mut self, id: Uuid) {
        self.workspaces.remove(&id);
    }
}
