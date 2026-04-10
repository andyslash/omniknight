use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::ui::theme::Theme;
use crate::workspace::model::WorkspaceStatus;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_workspace_list(frame, chunks[0], state);
    render_info_panel(frame, chunks[1], state);
}

fn render_workspace_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let workspaces = state.workspaces.list();

    if workspaces.is_empty() {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No workspaces yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'n' to create one",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(" Workspaces ")
                .borders(Borders::ALL)
                .border_style(Theme::active_border()),
        );
        frame.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = workspaces
        .iter()
        .enumerate()
        .map(|(i, ws)| {
            let status_icon = match ws.status {
                WorkspaceStatus::Active => "●",
                WorkspaceStatus::Archived => "○",
            };
            let style = if i == state.selected_workspace {
                Theme::selected()
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {status_icon} "),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(&ws.name, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Workspaces ")
            .borders(Borders::ALL)
            .border_style(Theme::active_border()),
    );

    frame.render_widget(list, area);
}

fn render_info_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let welcome = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("  Welcome to Omniknight", Theme::title())),
        Line::from(""),
        Line::from(Span::styled(
            "  A keyboard-first TUI cockpit for orchestrating AI agents.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Workspaces: {}", state.workspaces.list().len()),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!("  Agents running: {}", state.agents.active_count()),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!("  Missions: {}", state.missions.list().len()),
            Style::default().fg(Color::White),
        )),
    ])
    .block(
        Block::default()
            .title(" Overview ")
            .borders(Borders::ALL)
            .border_style(Theme::border()),
    );

    frame.render_widget(welcome, chunks[0]);

    let stats = Paragraph::new(Line::from(vec![Span::styled(
        format!(
            "  Agents: {}/{}",
            state.agents.active_count(),
            state.agents.list().len()
        ),
        Style::default().fg(Color::Cyan),
    )]))
    .block(
        Block::default()
            .title(" System ")
            .borders(Borders::ALL)
            .border_style(Theme::border()),
    );

    frame.render_widget(stats, chunks[1]);
}
