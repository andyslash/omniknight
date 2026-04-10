use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::mission::model::StepStatus;
use crate::ui::theme::Theme;
use crate::ui::widgets::agent_list;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let ws = match state.active_workspace() {
        Some(ws) => ws.clone(),
        None => {
            let p = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  No workspace selected",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "  Go to Home (1) and select one",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(Block::default().title(" Workspace ").borders(Borders::ALL));
            frame.render_widget(p, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left: Workspace info
    let env_lines: Vec<Line> = ws
        .env_vars
        .iter()
        .take(5)
        .map(|(k, v)| {
            let masked = if v.len() > 8 {
                format!("{}***", &v[..4])
            } else {
                "***".to_string()
            };
            Line::from(Span::styled(
                format!("  {k}={masked}"),
                Style::default().fg(Color::Gray),
            ))
        })
        .collect();

    let mut info_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Name: {}", ws.name),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!("  Root: {}", ws.root_dir.display()),
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!("  Status: {:?}", ws.status),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            format!("  Created: {}", ws.created_at.format("%Y-%m-%d")),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled("  Environment:", Theme::title())),
    ];
    info_lines.extend(env_lines);
    if ws.env_vars.is_empty() {
        info_lines.push(Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let info = Paragraph::new(info_lines).block(
        Block::default()
            .title(format!(" {} ", ws.name))
            .borders(Borders::ALL)
            .border_style(Theme::active_border()),
    );
    frame.render_widget(info, chunks[0]);

    // Right: Agents + Missions
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    let agents = state.agents.list();
    agent_list::render(frame, right_chunks[0], &agents, state.selected_agent);

    let missions = state.missions.list_for_workspace(ws.id);
    if missions.is_empty() {
        let p = Paragraph::new(Span::styled(
            " No missions — press 'm' to create one",
            Style::default().fg(Color::DarkGray),
        ))
        .block(Block::default().title(" Missions ").borders(Borders::ALL));
        frame.render_widget(p, right_chunks[1]);
    } else {
        let items: Vec<ListItem> = missions
            .iter()
            .map(|m| {
                let completed = m
                    .steps
                    .iter()
                    .filter(|s| s.status == StepStatus::Completed)
                    .count();
                ListItem::new(Line::from(Span::styled(
                    format!(
                        "  {} [{:?} {}/{}]",
                        m.title,
                        m.status,
                        completed,
                        m.steps.len()
                    ),
                    Style::default().fg(Color::White),
                )))
            })
            .collect();
        let list = List::new(items).block(
            Block::default()
                .title(" Missions ")
                .borders(Borders::ALL)
                .border_style(Theme::border()),
        );
        frame.render_widget(list, right_chunks[1]);
    }
}
