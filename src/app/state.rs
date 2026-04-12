use uuid::Uuid;

use crate::keybinds::mode::InputMode;
use crate::keybinds::vim::VimState;
use crate::ui::widgets::fuzzy_input::CommandPaletteState;
use crate::ui::widgets::input_dialog::InputDialogState;
use crate::workspace::manager::WorkspaceManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    SessionList,
    Terminal,
}

pub struct AppState {
    pub focused_pane: Pane,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub workspaces: WorkspaceManager,
    pub command_palette: CommandPaletteState,
    pub dialog: Option<InputDialogState>,
    /// Index into the flattened session_tree()
    pub selected_index: usize,
    /// Active session shown in terminal pane: (workspace_id, session_idx)
    pub active_session: Option<(Uuid, usize)>,
    pub terminal_scroll_offset: usize,
    pub vim: VimState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            focused_pane: Pane::SessionList,
            input_mode: InputMode::Normal,
            should_quit: false,
            workspaces: WorkspaceManager::new(),
            command_palette: CommandPaletteState::new(),
            dialog: None,
            selected_index: 0,
            active_session: None,
            terminal_scroll_offset: 0,
            vim: VimState::new(),
        }
    }

    pub fn active_workspace_id(&self) -> Option<Uuid> {
        self.active_session.map(|(ws_id, _)| ws_id)
    }

    pub fn active_workspace(&self) -> Option<&crate::workspace::model::Workspace> {
        self.active_workspace_id()
            .and_then(|id| self.workspaces.get(id))
    }
}
