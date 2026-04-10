use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::state::{AppState, View};
use crate::keybinds::mode::InputMode;
use crate::ui::theme::Theme;
use crate::ui::views::{command_palette, home, mission_control, terminal_view, workspace_view};

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_top_bar(frame, chunks[0], state);

    match state.active_view {
        View::Home => home::render(frame, chunks[1], state),
        View::MissionControl => mission_control::render(frame, chunks[1], state),
        View::Terminal => terminal_view::render(frame, chunks[1], state),
        View::Workspace => workspace_view::render(frame, chunks[1], state),
    }

    render_bottom_bar(frame, chunks[2], state);

    if state.command_palette.is_open {
        command_palette::render_overlay(frame, state);
    }
}

fn render_top_bar(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let mode_label = match state.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Insert => "INSERT",
        InputMode::Command => "COMMAND",
    };

    let ws_name = state
        .active_workspace()
        .map(|ws| ws.name.as_str())
        .unwrap_or("none");

    let running = state.agents.active_count();

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(format!(" [{mode_label}]"), Theme::status_bar()),
        Span::styled("  Omniknight v0.1", Theme::status_bar()),
        Span::styled(format!("  |  Workspace: {ws_name}"), Theme::status_bar()),
        Span::styled(
            format!("  |  Agents: {running} running"),
            Theme::status_bar(),
        ),
    ]))
    .style(Theme::status_bar());

    frame.render_widget(bar, area);
}

fn render_bottom_bar(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let hints = match state.active_view {
        View::Home => "q: quit  1-4: views  j/k: navigate  Enter: select  n: new workspace  :: palette",
        View::MissionControl => "q: quit  j/k: navigate  a: new agent  x: kill  p: pause  r: resume  t: terminal  :: palette",
        View::Terminal => "q: quit  i: insert mode  1-9: switch tab  Ctrl+n: new tab  :: palette",
        View::Workspace => "q: quit  j/k: navigate  e: edit  m: new mission  :: palette",
    };

    let bar = Paragraph::new(Span::styled(format!(" {hints}"), Theme::hint_bar()));
    frame.render_widget(bar, area);
}
