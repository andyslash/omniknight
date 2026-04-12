use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::action::Action;
use crate::app::state::{AppState, Pane};
use crate::keybinds::mode::InputMode;
use crate::ui::widgets::input_dialog::DialogIntent;

pub fn handle_key(key: KeyEvent, state: &mut AppState) -> Action {
    match state.input_mode {
        InputMode::Command => handle_command_mode(key),
        InputMode::Insert => handle_insert_mode(key),
        InputMode::Normal => handle_normal_mode(key, state),
        InputMode::Dialog => handle_dialog_mode(key),
    }
}

fn handle_command_mode(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::CloseCommandPalette,
        KeyCode::Enter => Action::CommandPaletteSelect,
        KeyCode::Backspace => Action::CommandPaletteBackspace,
        KeyCode::Up => Action::CommandPaletteNavigateUp,
        KeyCode::Down => Action::CommandPaletteNavigateDown,
        KeyCode::Char(c) => Action::CommandPaletteInput(c),
        _ => Action::Noop,
    }
}

fn handle_insert_mode(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::SetInputMode(InputMode::Normal),
        _ => Action::TerminalInput(key),
    }
}

fn handle_dialog_mode(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::DialogCancel,
        KeyCode::Enter => Action::DialogConfirm,
        KeyCode::Tab => Action::DialogNextField,
        KeyCode::BackTab => Action::DialogPrevField,
        KeyCode::Backspace => Action::DialogBackspace,
        KeyCode::Char(c) => Action::DialogInput(c),
        _ => Action::Noop,
    }
}

fn handle_normal_mode(key: KeyEvent, state: &mut AppState) -> Action {
    // Vim: g prefix
    if state.vim.pending_g {
        state.vim.pending_g = false;
        if key.code == KeyCode::Char('g') {
            state.vim.reset();
            return Action::GotoTop;
        }
        state.vim.reset();
        return Action::Noop;
    }

    // Vim: digit accumulation
    if let KeyCode::Char(c @ '0'..='9') = key.code {
        let d = c as u8 - b'0';
        if state.vim.count.is_some() || d >= 5 {
            state.vim.feed_digit(d);
            return Action::Noop;
        }
    }

    // Vim motions
    match key.code {
        KeyCode::Char('G') => {
            state.vim.reset();
            return Action::GotoBottom;
        }
        KeyCode::Char('g') => {
            state.vim.pending_g = true;
            return Action::Noop;
        }
        _ => {}
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('d') => {
                state.vim.reset();
                return Action::PageDown;
            }
            KeyCode::Char('u') => {
                state.vim.reset();
                return Action::PageUp;
            }
            _ => {}
        }
    }

    let count = state.vim.take_count();

    // Global keybinds
    match key.code {
        KeyCode::Char('q') => return Action::Quit,
        KeyCode::Char(':') => return Action::OpenCommandPalette,
        KeyCode::Char('j') | KeyCode::Down => {
            return if count > 1 {
                Action::NavigateN {
                    direction: 1,
                    count,
                }
            } else {
                Action::NavigateDown
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            return if count > 1 {
                Action::NavigateN {
                    direction: -1,
                    count,
                }
            } else {
                Action::NavigateUp
            }
        }
        // Session cycling (global — works from any pane)
        KeyCode::Char(']') => return Action::NextSession,
        KeyCode::Char('[') => return Action::PrevSession,
        _ => {}
    }

    // Pane-specific
    match state.focused_pane {
        Pane::Workspaces => match key.code {
            KeyCode::Char('l') | KeyCode::Enter | KeyCode::Tab => Action::FocusTerminal,
            KeyCode::Char('n') => Action::OpenDialog(DialogIntent::CreateWorkspace),
            _ => Action::Noop,
        },
        Pane::Terminal => match key.code {
            KeyCode::Char('h') | KeyCode::Esc | KeyCode::BackTab => Action::FocusWorkspaces,
            KeyCode::Char('i') => Action::SetInputMode(InputMode::Insert),
            KeyCode::Char('t') => Action::NewShellSession,
            KeyCode::Char('a') => Action::OpenDialog(DialogIntent::LaunchAgent),
            _ => Action::Noop,
        },
    }
}
