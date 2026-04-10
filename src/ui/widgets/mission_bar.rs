use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::mission::model::{Mission, MissionStatus, StepStatus};

pub fn render(frame: &mut Frame, area: Rect, missions: &[&Mission]) {
    if missions.is_empty() {
        let p = Paragraph::new(Span::styled(
            " No missions",
            Style::default().fg(Color::DarkGray),
        ))
        .block(Block::default().title(" Missions ").borders(Borders::ALL));
        frame.render_widget(p, area);
        return;
    }

    let spans: Vec<Span> = missions
        .iter()
        .enumerate()
        .flat_map(|(i, m)| {
            let completed = m
                .steps
                .iter()
                .filter(|s| s.status == StepStatus::Completed)
                .count();
            let total = m.steps.len();
            let status_color = match m.status {
                MissionStatus::InProgress => Color::Green,
                MissionStatus::Completed => Color::Cyan,
                MissionStatus::Failed => Color::Red,
                MissionStatus::Planning => Color::Yellow,
                MissionStatus::Cancelled => Color::DarkGray,
            };
            let mut spans = vec![Span::styled(
                format!("{} [{:?} {}/{}]", m.title, m.status, completed, total),
                Style::default().fg(status_color),
            )];
            if i < missions.len() - 1 {
                spans.push(Span::raw("  |  "));
            }
            spans
        })
        .collect();

    let paragraph = Paragraph::new(Line::from(spans))
        .block(Block::default().title(" Missions ").borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
