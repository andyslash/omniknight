use crossterm::event::KeyEvent;
use std::path::PathBuf;
use uuid::Uuid;

use crate::app::state::View;
use crate::keybinds::mode::InputMode;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    SwitchView(View),
    SetInputMode(InputMode),
    LaunchAgent {
        command: String,
        args: Vec<String>,
        task_description: String,
    },
    KillAgent(Uuid),
    PauseAgent(Uuid),
    ResumeAgent(Uuid),
    FocusAgent(Uuid),
    NavigateUp,
    NavigateDown,
    Select,
    Back,
    SwitchWorkspace(Uuid),
    CreateWorkspace {
        name: String,
        root: PathBuf,
    },
    NewTerminalTab,
    CloseTerminalTab(usize),
    SwitchTerminalTab(usize),
    TerminalInput(KeyEvent),
    OpenCommandPalette,
    CloseCommandPalette,
    CommandPaletteInput(char),
    CommandPaletteBackspace,
    CommandPaletteSelect,
    CreateMission {
        title: String,
    },
    Noop,
}
