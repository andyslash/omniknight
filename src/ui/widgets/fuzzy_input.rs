use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::action::Action;
use crate::app::state::View;
use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct PaletteItem {
    pub category: String,
    pub label: String,
    pub action: Action,
}

pub struct CommandPaletteState {
    pub input: String,
    pub items: Vec<PaletteItem>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub is_open: bool,
}

impl CommandPaletteState {
    pub fn new() -> Self {
        let items = vec![
            PaletteItem {
                category: "view".into(),
                label: "Home Dashboard".into(),
                action: Action::SwitchView(View::Home),
            },
            PaletteItem {
                category: "view".into(),
                label: "Mission Control".into(),
                action: Action::SwitchView(View::MissionControl),
            },
            PaletteItem {
                category: "view".into(),
                label: "Terminal".into(),
                action: Action::SwitchView(View::Terminal),
            },
            PaletteItem {
                category: "view".into(),
                label: "Workspace".into(),
                action: Action::SwitchView(View::Workspace),
            },
            PaletteItem {
                category: "command".into(),
                label: "Quit".into(),
                action: Action::Quit,
            },
            PaletteItem {
                category: "terminal".into(),
                label: "New Terminal Tab".into(),
                action: Action::NewTerminalTab,
            },
        ];
        let filtered = (0..items.len()).collect();
        Self {
            input: String::new(),
            items,
            filtered,
            selected: 0,
            is_open: false,
        }
    }

    pub fn filter(&mut self) {
        let query = self.input.to_lowercase();
        if query.is_empty() {
            self.filtered = (0..self.items.len()).collect();
        } else {
            self.filtered = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    item.label.to_lowercase().contains(&query)
                        || item.category.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.selected = 0;
    }

    pub fn selected_action(&self) -> Option<&Action> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.items.get(idx))
            .map(|item| &item.action)
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &CommandPaletteState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Input field
    let input = Paragraph::new(Line::from(vec![
        Span::raw("> "),
        Span::styled(&state.input, Theme::palette_input()),
        Span::raw("_"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::active_border()),
    );
    frame.render_widget(input, chunks[0]);

    // Results list
    let items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let item = &state.items[idx];
            let style = if i == state.selected {
                Theme::palette_selected()
            } else {
                Theme::log_text()
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}: ", item.category),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&item.label, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border()),
    );
    frame.render_widget(list, chunks[1]);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
