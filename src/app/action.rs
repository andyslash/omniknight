use crossterm::event::KeyEvent;
use std::path::PathBuf;

use crate::keybinds::mode::InputMode;
use crate::ui::widgets::input_dialog::DialogIntent;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    SetInputMode(InputMode),
    FocusSessionList,
    FocusTerminal,
    NavigateUp,
    NavigateDown,
    GotoTop,
    GotoBottom,
    PageUp,
    PageDown,
    NavigateN {
        direction: i32,
        count: usize,
    },
    Select,
    Back,
    ToggleCollapse,
    // Workspace
    CreateWorkspace {
        name: String,
        root: PathBuf,
    },
    // Sessions
    NewShellSession,
    SpawnAgent {
        command: String,
        args: Vec<String>,
        title: String,
    },
    NextSession,
    PrevSession,
    // Terminal input
    TerminalInput(KeyEvent),
    // Command palette
    OpenCommandPalette,
    CloseCommandPalette,
    CommandPaletteInput(char),
    CommandPaletteBackspace,
    CommandPaletteSelect,
    CommandPaletteNavigateUp,
    CommandPaletteNavigateDown,
    // Dialog
    OpenDialog(DialogIntent),
    DialogInput(char),
    DialogBackspace,
    DialogConfirm,
    DialogCancel,
    DialogNextField,
    DialogPrevField,
    Noop,
}
