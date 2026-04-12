use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub enum DialogIntent {
    CreateWorkspace,
    LaunchAgent,
}

#[derive(Debug, Clone)]
pub struct DialogField {
    pub label: String,
    pub value: String,
    pub placeholder: String,
}

#[derive(Debug, Clone)]
pub struct InputDialogState {
    pub title: String,
    pub fields: Vec<DialogField>,
    pub focused_field: usize,
    pub intent: DialogIntent,
}

impl InputDialogState {
    pub fn workspace_dialog() -> Self {
        Self {
            title: "New Workspace".to_string(),
            fields: vec![
                DialogField {
                    label: "Name".to_string(),
                    value: String::new(),
                    placeholder: "my-project".to_string(),
                },
                DialogField {
                    label: "Root Dir".to_string(),
                    value: std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    placeholder: "/path/to/project".to_string(),
                },
            ],
            focused_field: 0,
            intent: DialogIntent::CreateWorkspace,
        }
    }

    pub fn agent_dialog() -> Self {
        Self {
            title: "Launch Agent".to_string(),
            fields: vec![
                DialogField {
                    label: "Command".to_string(),
                    value: String::new(),
                    placeholder: "claude".to_string(),
                },
                DialogField {
                    label: "Task".to_string(),
                    value: String::new(),
                    placeholder: "Describe what the agent should do".to_string(),
                },
            ],
            focused_field: 0,
            intent: DialogIntent::LaunchAgent,
        }
    }

    pub fn input_char(&mut self, ch: char) {
        if let Some(field) = self.fields.get_mut(self.focused_field) {
            field.value.push(ch);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(field) = self.fields.get_mut(self.focused_field) {
            field.value.pop();
        }
    }

    pub fn next_field(&mut self) {
        if !self.fields.is_empty() {
            self.focused_field = (self.focused_field + 1) % self.fields.len();
        }
    }

    pub fn prev_field(&mut self) {
        if !self.fields.is_empty() {
            self.focused_field = (self.focused_field + self.fields.len() - 1) % self.fields.len();
        }
    }
}

pub fn render_overlay(frame: &mut Frame, dialog: &InputDialogState) {
    let area = frame.area();
    let height = (dialog.fields.len() as u16 * 3) + 4;
    let popup_area = centered_rect(50, height.min(60), area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" {} ", dialog.title))
        .borders(Borders::ALL)
        .border_style(Theme::active_border());

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let constraints: Vec<Constraint> = dialog
        .fields
        .iter()
        .map(|_| Constraint::Length(3))
        .chain(std::iter::once(Constraint::Length(1)))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, field) in dialog.fields.iter().enumerate() {
        let is_focused = i == dialog.focused_field;
        let display_value = if field.value.is_empty() {
            Span::styled(&field.placeholder, Style::default().fg(Color::DarkGray))
        } else {
            Span::styled(&field.value, Style::default().fg(Color::White))
        };

        let cursor = if is_focused { "_" } else { "" };

        let field_border = if is_focused {
            Theme::active_border()
        } else {
            Theme::border()
        };

        let label_style = if is_focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let p = Paragraph::new(Line::from(vec![
            display_value,
            Span::styled(cursor, Style::default().fg(Color::Cyan)),
        ]))
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", field.label), label_style))
                .borders(Borders::ALL)
                .border_style(field_border),
        );

        frame.render_widget(p, chunks[i]);
    }

    // Hint line at bottom
    if let Some(&last_chunk) = chunks.last() {
        let hint = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": confirm  "),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": next field  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": cancel"),
        ]));
        frame.render_widget(hint, last_chunk);
    }
}

/// centered_rect using absolute height instead of percentage
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical_padding = area.height.saturating_sub(height) / 2;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(height),
            Constraint::Min(0),
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
