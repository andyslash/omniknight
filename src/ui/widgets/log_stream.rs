use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, lines: &[String], title: &str) {
    let text_lines: Vec<Line> = lines
        .iter()
        .map(|line| Line::from(Span::styled(line.as_str(), Theme::log_text())))
        .collect();

    let visible_height = area.height.saturating_sub(2) as usize;
    let start = text_lines.len().saturating_sub(visible_height);
    let visible: Vec<Line> = text_lines.into_iter().skip(start).collect();

    let paragraph = Paragraph::new(visible)
        .block(
            Block::default()
                .title(format!(" {title} "))
                .borders(Borders::ALL)
                .border_style(Theme::border()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
