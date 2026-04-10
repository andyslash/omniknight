use uuid::Uuid;

use crate::terminal::scrollback::Scrollback;

#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub title: String,
    pub scrollback: Scrollback,
    pub is_focused: bool,
}

impl TerminalSession {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id: None,
            agent_id: None,
            title,
            scrollback: Scrollback::default(),
            is_focused: false,
        }
    }

    pub fn new_for_agent(agent_id: Uuid, title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id: None,
            agent_id: Some(agent_id),
            title,
            scrollback: Scrollback::default(),
            is_focused: false,
        }
    }

    pub fn push_output(&mut self, line: String) {
        self.scrollback.push(line);
    }
}
