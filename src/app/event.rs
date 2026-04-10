use crossterm::event::KeyEvent;
use uuid::Uuid;

use crate::agent::state::AgentState;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
    AgentOutput { agent_id: Uuid, line: String },
    AgentStateChange { agent_id: Uuid, state: AgentState },
}
