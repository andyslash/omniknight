use ratatui::style::{Color, Modifier, Style};

use crate::agent::state::AgentState;

pub struct Theme;

impl Theme {
    pub fn status_bar() -> Style {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    }
    pub fn hint_bar() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn selected() -> Style {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
    pub fn title() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    pub fn agent_state(state: AgentState) -> Style {
        match state {
            AgentState::Running => Style::default().fg(Color::Green),
            AgentState::Paused => Style::default().fg(Color::Yellow),
            AgentState::Idle => Style::default().fg(Color::Gray),
            AgentState::Completed => Style::default().fg(Color::Cyan),
            AgentState::Failed => Style::default().fg(Color::Red),
            AgentState::Killed => Style::default().fg(Color::DarkGray),
        }
    }
    pub fn border() -> Style {
        Style::default().fg(Color::Gray)
    }
    pub fn active_border() -> Style {
        Style::default().fg(Color::Cyan)
    }
    pub fn log_text() -> Style {
        Style::default().fg(Color::White)
    }
    pub fn palette_input() -> Style {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    }
    pub fn palette_selected() -> Style {
        Style::default().bg(Color::Blue).fg(Color::White)
    }
}
