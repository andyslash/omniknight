use crossterm::event::{KeyCode, KeyEvent};

use crate::app::action::Action;
use crate::app::state::{AppState, View};
use crate::keybinds::mode::InputMode;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Action {
    match state.input_mode {
        InputMode::Command => handle_command_mode(key),
        InputMode::Insert => handle_insert_mode(key),
        InputMode::Normal => handle_normal_mode(key, state),
    }
}

fn handle_command_mode(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::CloseCommandPalette,
        KeyCode::Enter => Action::CommandPaletteSelect,
        KeyCode::Backspace => Action::CommandPaletteBackspace,
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

fn handle_normal_mode(key: KeyEvent, state: &AppState) -> Action {
    // Global keybinds
    match key.code {
        KeyCode::Char('q') => return Action::Quit,
        KeyCode::Char(':') => return Action::OpenCommandPalette,
        KeyCode::Char('1') => return Action::SwitchView(View::Home),
        KeyCode::Char('2') => return Action::SwitchView(View::MissionControl),
        KeyCode::Char('3') => return Action::SwitchView(View::Terminal),
        KeyCode::Char('4') => return Action::SwitchView(View::Workspace),
        KeyCode::Char('j') => return Action::NavigateDown,
        KeyCode::Char('k') => return Action::NavigateUp,
        KeyCode::Enter => return Action::Select,
        KeyCode::Esc => return Action::Back,
        _ => {}
    }

    // View-specific keybinds
    match state.active_view {
        View::MissionControl => handle_mission_control_keys(key, state),
        View::Terminal => handle_terminal_keys(key),
        View::Home => Action::Noop,
        View::Workspace => Action::Noop,
    }
}

fn handle_mission_control_keys(key: KeyEvent, state: &AppState) -> Action {
    match key.code {
        KeyCode::Char('x') => {
            let agents = state.agents.list();
            agents
                .get(state.selected_agent)
                .map(|a| Action::KillAgent(a.id))
                .unwrap_or(Action::Noop)
        }
        KeyCode::Char('p') => {
            let agents = state.agents.list();
            agents
                .get(state.selected_agent)
                .map(|a| Action::PauseAgent(a.id))
                .unwrap_or(Action::Noop)
        }
        KeyCode::Char('r') => {
            let agents = state.agents.list();
            agents
                .get(state.selected_agent)
                .map(|a| Action::ResumeAgent(a.id))
                .unwrap_or(Action::Noop)
        }
        KeyCode::Char('t') => Action::SwitchView(View::Terminal),
        _ => Action::Noop,
    }
}

fn handle_terminal_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('i') => Action::SetInputMode(InputMode::Insert),
        _ => Action::Noop,
    }
}
