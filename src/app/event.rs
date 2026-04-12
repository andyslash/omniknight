use crossterm::event::KeyEvent;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
    WorkspaceOutput { workspace_id: Uuid },
}
