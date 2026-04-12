use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use crossbeam_channel::Sender;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use uuid::Uuid;

use crate::app::event::AppEvent;
use crate::workspace::model::{Workspace, WorkspaceStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Running,
    Idle,
    Done,
    Error,
}

pub struct Session {
    pub id: Uuid,
    pub title: String,
    pub is_agent: bool,
    pub pty_writer: Option<Box<dyn Write + Send>>,
    pub output: Arc<Mutex<Vec<u8>>>,
    pub last_output_at: Arc<Mutex<Instant>>,
    pub child: Option<Box<dyn portable_pty::Child + Send>>,
    _reader_handle: Option<std::thread::JoinHandle<()>>,
}

impl Session {
    /// Quick status check based on output activity (no mutable borrow needed).
    pub fn status(&self) -> SessionStatus {
        let last = *self.last_output_at.lock().unwrap();
        if last.elapsed().as_secs() < 3 {
            SessionStatus::Running
        } else {
            SessionStatus::Idle
        }
    }
}

pub struct ManagedWorkspace {
    pub workspace: Workspace,
    pub sessions: Vec<Session>,
    pub active_session: usize,
    pub collapsed: bool,
}

/// Entry in the flattened session tree for rendering.
pub enum SessionTreeEntry {
    WorkspaceHeader {
        workspace_id: Uuid,
        name: String,
        collapsed: bool,
        session_count: usize,
    },
    SessionItem {
        workspace_id: Uuid,
        session_idx: usize,
        title: String,
        status: SessionStatus,
        is_agent: bool,
    },
}

pub struct WorkspaceManager {
    workspaces: Vec<ManagedWorkspace>,
    event_sender: Option<Sender<AppEvent>>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: Vec::new(),
            event_sender: None,
        }
    }

    pub fn set_event_sender(&mut self, sender: Sender<AppEvent>) {
        self.event_sender = Some(sender);
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
            branch: None,
        };
        let managed = ManagedWorkspace {
            workspace,
            sessions: Vec::new(),
            active_session: 0,
            collapsed: false,
        };
        self.workspaces.push(managed);
        id
    }

    pub fn spawn_shell(&mut self, workspace_id: Uuid) -> Result<usize> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        self.spawn_session(workspace_id, "shell".to_string(), &shell, &[], false)
    }

    pub fn spawn_agent(
        &mut self,
        workspace_id: Uuid,
        title: String,
        command: &str,
        args: &[String],
    ) -> Result<usize> {
        self.spawn_session(workspace_id, title, command, args, true)
    }

    fn spawn_session(
        &mut self,
        workspace_id: Uuid,
        title: String,
        command: &str,
        args: &[String],
        is_agent: bool,
    ) -> Result<usize> {
        let mw = self
            .workspaces
            .iter_mut()
            .find(|mw| mw.workspace.id == workspace_id)
            .ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;

        let pty_system = NativePtySystem::default();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);
        cmd.cwd(&mw.workspace.root_dir);
        for (k, v) in &mw.workspace.env_vars {
            cmd.env(k, v);
        }

        let child = pty_pair.slave.spawn_command(cmd)?;
        let reader = pty_pair.master.try_clone_reader()?;
        let writer = pty_pair.master.take_writer()?;

        let output_buf = Arc::new(Mutex::new(Vec::new()));
        let buf_clone = Arc::clone(&output_buf);
        let last_output = Arc::new(Mutex::new(Instant::now()));
        let last_output_clone = Arc::clone(&last_output);
        let sender = self.event_sender.clone();
        let ws_id = workspace_id;

        let reader_handle = std::thread::spawn(move || {
            let mut reader = reader;
            let mut chunk = [0u8; 4096];
            loop {
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        {
                            let mut buf = buf_clone.lock().unwrap();
                            buf.extend_from_slice(&chunk[..n]);
                        }
                        {
                            let mut ts = last_output_clone.lock().unwrap();
                            *ts = Instant::now();
                        }
                        if let Some(ref tx) = sender {
                            let _ = tx.send(AppEvent::WorkspaceOutput {
                                workspace_id: ws_id,
                            });
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let session = Session {
            id: Uuid::new_v4(),
            title,
            is_agent,
            pty_writer: Some(writer),
            output: output_buf,
            last_output_at: last_output,
            child: Some(child),
            _reader_handle: Some(reader_handle),
        };

        mw.sessions.push(session);
        let idx = mw.sessions.len() - 1;
        mw.active_session = idx;
        Ok(idx)
    }

    /// Build flattened session tree for rendering in the left pane.
    pub fn session_tree(&self) -> Vec<SessionTreeEntry> {
        let mut entries = Vec::new();
        for mw in &self.workspaces {
            entries.push(SessionTreeEntry::WorkspaceHeader {
                workspace_id: mw.workspace.id,
                name: mw.workspace.name.clone(),
                collapsed: mw.collapsed,
                session_count: mw.sessions.len(),
            });
            if !mw.collapsed {
                for (idx, session) in mw.sessions.iter().enumerate() {
                    entries.push(SessionTreeEntry::SessionItem {
                        workspace_id: mw.workspace.id,
                        session_idx: idx,
                        title: session.title.clone(),
                        status: session.status(),
                        is_agent: session.is_agent,
                    });
                }
            }
        }
        entries
    }

    pub fn toggle_collapse(&mut self, workspace_id: Uuid) {
        if let Some(mw) = self
            .workspaces
            .iter_mut()
            .find(|mw| mw.workspace.id == workspace_id)
        {
            mw.collapsed = !mw.collapsed;
        }
    }

    /// Get output for a specific session (by workspace + index).
    pub fn session_output(&self, workspace_id: Uuid, session_idx: usize) -> Option<Vec<u8>> {
        let mw = self
            .workspaces
            .iter()
            .find(|mw| mw.workspace.id == workspace_id)?;
        let session = mw.sessions.get(session_idx)?;
        let buf = session.output.lock().unwrap();
        Some(buf.clone())
    }

    /// Write to a specific session (by workspace + index).
    pub fn write_to_session(
        &mut self,
        workspace_id: Uuid,
        session_idx: usize,
        data: &[u8],
    ) -> Result<()> {
        let mw = self
            .workspaces
            .iter_mut()
            .find(|mw| mw.workspace.id == workspace_id)
            .ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;
        let session = mw
            .sessions
            .get_mut(session_idx)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        if let Some(ref mut writer) = session.pty_writer {
            writer.write_all(data)?;
            writer.flush()?;
        }
        Ok(())
    }

    /// Get active session output (for backward compat).
    pub fn active_session_output(&self, workspace_id: Uuid) -> Option<Vec<u8>> {
        let mw = self
            .workspaces
            .iter()
            .find(|mw| mw.workspace.id == workspace_id)?;
        self.session_output(workspace_id, mw.active_session)
    }

    /// Write to active session (for backward compat).
    pub fn write_to_active_session(&mut self, workspace_id: Uuid, data: &[u8]) -> Result<()> {
        let idx = self
            .workspaces
            .iter()
            .find(|mw| mw.workspace.id == workspace_id)
            .map(|mw| mw.active_session)
            .ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;
        self.write_to_session(workspace_id, idx, data)
    }

    pub fn session_titles(&self, workspace_id: Uuid) -> Vec<(&str, bool)> {
        self.workspaces
            .iter()
            .find(|mw| mw.workspace.id == workspace_id)
            .map(|mw| {
                mw.sessions
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (s.title.as_str(), i == mw.active_session))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn session_count(&self, workspace_id: Uuid) -> usize {
        self.workspaces
            .iter()
            .find(|mw| mw.workspace.id == workspace_id)
            .map(|mw| mw.sessions.len())
            .unwrap_or(0)
    }

    pub fn switch_session(&mut self, workspace_id: Uuid, idx: usize) {
        if let Some(mw) = self
            .workspaces
            .iter_mut()
            .find(|mw| mw.workspace.id == workspace_id)
        {
            if idx < mw.sessions.len() {
                mw.active_session = idx;
            }
        }
    }

    pub fn next_session(&mut self, workspace_id: Uuid) {
        if let Some(mw) = self
            .workspaces
            .iter_mut()
            .find(|mw| mw.workspace.id == workspace_id)
        {
            if !mw.sessions.is_empty() {
                mw.active_session = (mw.active_session + 1) % mw.sessions.len();
            }
        }
    }

    pub fn prev_session(&mut self, workspace_id: Uuid) {
        if let Some(mw) = self
            .workspaces
            .iter_mut()
            .find(|mw| mw.workspace.id == workspace_id)
        {
            if !mw.sessions.is_empty() {
                mw.active_session = (mw.active_session + mw.sessions.len() - 1) % mw.sessions.len();
            }
        }
    }

    pub fn has_sessions(&self, workspace_id: Uuid) -> bool {
        self.session_count(workspace_id) > 0
    }

    /// Resolve a tree index to the workspace it belongs to.
    pub fn workspace_for_tree_index(&self, idx: usize) -> Option<Uuid> {
        let tree = self.session_tree();
        match tree.get(idx) {
            Some(SessionTreeEntry::WorkspaceHeader { workspace_id, .. }) => Some(*workspace_id),
            Some(SessionTreeEntry::SessionItem { workspace_id, .. }) => Some(*workspace_id),
            None => None,
        }
    }

    pub fn get(&self, id: Uuid) -> Option<&Workspace> {
        self.workspaces
            .iter()
            .find(|mw| mw.workspace.id == id)
            .map(|mw| &mw.workspace)
    }

    pub fn list(&self) -> Vec<&Workspace> {
        self.workspaces.iter().map(|mw| &mw.workspace).collect()
    }

    pub fn delete(&mut self, id: Uuid) {
        self.workspaces.retain(|mw| mw.workspace.id != id);
    }
}
