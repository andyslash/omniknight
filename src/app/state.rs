use uuid::Uuid;

use crate::keybinds::mode::InputMode;
use crate::keybinds::vim::VimState;
use crate::ui::widgets::fuzzy_input::CommandPaletteState;
use crate::ui::widgets::input_dialog::InputDialogState;
use crate::workspace::manager::WorkspaceManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Workspaces,
    Terminal,
}

pub struct AppState {
    pub focused_pane: Pane,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub workspaces: WorkspaceManager,
    pub command_palette: CommandPaletteState,
    pub dialog: Option<InputDialogState>,
    pub active_workspace_id: Option<Uuid>,
    pub selected_workspace: usize,
    pub terminal_scroll_offset: usize,
    pub vim: VimState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            focused_pane: Pane::Workspaces,
            input_mode: InputMode::Normal,
            should_quit: false,
            workspaces: WorkspaceManager::new(),
            command_palette: CommandPaletteState::new(),
            dialog: None,
            active_workspace_id: None,
            selected_workspace: 0,
            terminal_scroll_offset: 0,
            vim: VimState::new(),
        }
    }

    pub fn active_workspace(&self) -> Option<&crate::workspace::model::Workspace> {
        self.active_workspace_id
            .and_then(|id| self.workspaces.get(id))
    }
}
