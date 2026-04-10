use uuid::Uuid;

use crate::terminal::session::TerminalSession;

pub struct TerminalManager {
    sessions: Vec<TerminalSession>,
    active_tab: usize,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            active_tab: 0,
        }
    }

    pub fn create_tab(&mut self, title: String) -> usize {
        let session = TerminalSession::new(title);
        self.sessions.push(session);
        let idx = self.sessions.len() - 1;
        self.active_tab = idx;
        idx
    }

    pub fn create_agent_tab(&mut self, agent_id: Uuid, title: String) -> usize {
        let session = TerminalSession::new_for_agent(agent_id, title);
        self.sessions.push(session);
        let idx = self.sessions.len() - 1;
        self.active_tab = idx;
        idx
    }

    pub fn close_tab(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.sessions.remove(index);
            if self.active_tab >= self.sessions.len() && !self.sessions.is_empty() {
                self.active_tab = self.sessions.len() - 1;
            }
        }
    }

    pub fn active(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active_tab)
    }

    #[allow(dead_code)]
    pub fn active_mut(&mut self) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(self.active_tab)
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.active_tab = index;
        }
    }

    pub fn tabs(&self) -> &[TerminalSession] {
        &self.sessions
    }

    pub fn active_tab_index(&self) -> usize {
        self.active_tab
    }
}
