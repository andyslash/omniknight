use ratatui::style::{Color, Modifier, Style};

use crate::workspace::manager::SessionStatus;

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
    pub fn session_status(status: SessionStatus) -> Style {
        match status {
            SessionStatus::Running => Style::default().fg(Color::Green),
            SessionStatus::Idle => Style::default().fg(Color::Gray),
            SessionStatus::Done => Style::default().fg(Color::Cyan),
            SessionStatus::Error => Style::default().fg(Color::Red),
        }
    }
    pub fn session_status_icon(status: SessionStatus) -> &'static str {
        match status {
            SessionStatus::Running => "●",
            SessionStatus::Idle => "○",
            SessionStatus::Done => "✓",
            SessionStatus::Error => "✕",
        }
    }
    pub fn border() -> Style {
        Style::default().fg(Color::Gray)
    }
    pub fn active_border() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    pub fn focused_title() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    pub fn unfocused_title() -> Style {
        Style::default().fg(Color::DarkGray)
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
