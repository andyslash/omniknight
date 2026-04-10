use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::{agent_list, log_stream, mission_bar};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let main_and_bar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_and_bar[0]);

    let agents = state.agents.list();
    agent_list::render(frame, main_chunks[0], &agents, state.selected_agent);

    if let Some(agent) = agents.get(state.selected_agent) {
        let lines: Vec<String> = agent.output.iter().cloned().collect();
        log_stream::render(
            frame,
            main_chunks[1],
            &lines,
            &format!("{} — {}", agent.name, agent.task_description),
        );
    } else {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No agents running",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'a' to launch an agent",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(" Agent Output ")
                .borders(Borders::ALL)
                .border_style(Theme::border()),
        );
        frame.render_widget(empty, main_chunks[1]);
    }

    let missions = state.missions.list();
    mission_bar::render(frame, main_and_bar[1], &missions);
}
