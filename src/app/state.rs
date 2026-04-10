use uuid::Uuid;

use crate::agent::manager::AgentManager;
use crate::keybinds::mode::InputMode;
use crate::mission::manager::MissionManager;
use crate::terminal::manager::TerminalManager;
use crate::ui::widgets::fuzzy_input::CommandPaletteState;
use crate::workspace::manager::WorkspaceManager;
use crate::workspace::model::Workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    MissionControl,
    Terminal,
    Workspace,
}

pub struct AppState {
    pub active_view: View,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub agents: AgentManager,
    pub workspaces: WorkspaceManager,
    pub terminals: TerminalManager,
    pub missions: MissionManager,
    pub command_palette: CommandPaletteState,
    pub active_workspace_id: Option<Uuid>,
    pub selected_agent: usize,
    pub selected_workspace: usize,
    pub selected_mission: usize,
    pub selected_terminal_tab: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_view: View::Home,
            input_mode: InputMode::Normal,
            should_quit: false,
            agents: AgentManager::new(),
            workspaces: WorkspaceManager::new(),
            terminals: TerminalManager::new(),
            missions: MissionManager::new(),
            command_palette: CommandPaletteState::new(),
            active_workspace_id: None,
            selected_agent: 0,
            selected_workspace: 0,
            selected_mission: 0,
            selected_terminal_tab: 0,
        }
    }

    pub fn switch_view(&mut self, view: View) {
        self.active_view = view;
    }

    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.active_workspace_id
            .and_then(|id| self.workspaces.get(id))
    }
}
