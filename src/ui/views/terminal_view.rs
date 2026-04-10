use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let tabs = state.terminals.tabs();
    if tabs.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No terminal sessions",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Press Ctrl+n to create one",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(" Terminal ")
                .borders(Borders::ALL)
                .border_style(Theme::border()),
        );
        frame.render_widget(empty, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let tab_titles: Vec<Line> = tabs
        .iter()
        .map(|t| Line::from(Span::raw(&t.title)))
        .collect();

    let tab_bar = Tabs::new(tab_titles)
        .block(
            Block::default()
                .title(" Terminals ")
                .borders(Borders::ALL)
                .border_style(Theme::border()),
        )
        .select(state.terminals.active_tab_index())
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" | "));

    frame.render_widget(tab_bar, chunks[0]);

    if let Some(session) = state.terminals.active() {
        let height = chunks[1].height.saturating_sub(2) as usize;
        let visible = session.scrollback.visible_lines(height);
        let lines: Vec<Line> = visible
            .iter()
            .map(|l| Line::from(Span::styled(*l, Theme::log_text())))
            .collect();

        let output = Paragraph::new(lines).block(
            Block::default()
                .title(format!(" {} ", session.title))
                .borders(Borders::ALL)
                .border_style(Theme::active_border()),
        );

        frame.render_widget(output, chunks[1]);
    }
}
