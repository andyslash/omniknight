use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs};
use ratatui::Frame;
use vt100::Color as VtColor;

use crate::app::state::{AppState, Pane};
use crate::keybinds::mode::InputMode;
use crate::ui::theme::Theme;
use crate::ui::widgets::{fuzzy_input, input_dialog};
use crate::workspace::model::WorkspaceStatus;

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_status_bar(frame, chunks[0], state);
    render_main(frame, chunks[1], state);
    render_hints_bar(frame, chunks[2], state);

    if state.command_palette.is_open {
        let popup_area = fuzzy_input::centered_rect(50, 40, frame.area());
        frame.render_widget(Clear, popup_area);
        fuzzy_input::render(frame, popup_area, &state.command_palette);
    }
    if let Some(dialog) = &state.dialog {
        input_dialog::render_overlay(frame, dialog);
    }
}

fn render_main(frame: &mut Frame, area: Rect, state: &AppState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    render_workspaces(frame, cols[0], state);
    render_terminal(frame, cols[1], state);
}

fn render_workspaces(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focused_pane == Pane::SessionList;
    let (border_style, title_style) = pane_styles(is_focused);
    let title = pane_title("Workspaces", is_focused);

    let workspaces = state.workspaces.list();
    if workspaces.is_empty() {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                " No workspaces",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                " Press 'n' to create",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(Span::styled(title, title_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = workspaces
        .iter()
        .enumerate()
        .map(|(i, ws)| {
            let icon = match ws.status {
                WorkspaceStatus::Active => "●",
                WorkspaceStatus::Archived => "○",
            };
            let is_active = Some(ws.id) == state.active_workspace_id();
            let has_sessions = state.workspaces.has_sessions(ws.id);
            let style = if i == state.selected_index {
                Theme::selected()
            } else if is_active {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let indicator = if has_sessions { " ◉" } else { "" };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {icon} "), Style::default().fg(Color::Green)),
                Span::styled(&ws.name, style),
                Span::styled(indicator, Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(title, title_style))
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(list, area);
}

fn render_terminal(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focused_pane == Pane::Terminal;
    let (border_style, title_style) = pane_styles(is_focused);
    let ws_id = state.active_workspace_id();

    // No workspace selected
    if ws_id.is_none() {
        let title = pane_title("Terminal", is_focused);
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Select a workspace (Enter) to start",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(Span::styled(title, title_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(p, area);
        return;
    }
    let ws_id = ws_id.unwrap();

    let sessions = state.workspaces.session_titles(ws_id);

    // No sessions yet
    if sessions.is_empty() {
        let ws_name = state
            .active_workspace()
            .map(|w| w.name.as_str())
            .unwrap_or("—");
        let title = pane_title(ws_name, is_focused);
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No sessions — press 't' for shell or 'a' for agent",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(Span::styled(title, title_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(p, area);
        return;
    }

    // Has sessions — render tab bar + output
    let has_multiple = sessions.len() > 1;
    let tab_height = if has_multiple { 1u16 } else { 0u16 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(tab_height), Constraint::Min(0)])
        .split(area);

    // Tab bar (only if multiple sessions)
    if has_multiple {
        let active_idx = sessions.iter().position(|(_, active)| *active).unwrap_or(0);
        let tab_titles: Vec<Line> = sessions.iter().map(|(name, _)| Line::from(*name)).collect();
        let tabs = Tabs::new(tab_titles)
            .select(active_idx)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::raw(" │ "));
        frame.render_widget(tabs, chunks[0]);
    }

    // Terminal output
    let output_area = chunks[if has_multiple { 1 } else { 0 }];
    let rows = output_area.height.saturating_sub(2);
    let cols = output_area.width.saturating_sub(2);

    let mode_indicator = match state.input_mode {
        InputMode::Insert if is_focused => " [INSERT]",
        _ => "",
    };
    let active_title = sessions
        .iter()
        .find(|(_, active)| *active)
        .map(|(name, _)| *name)
        .unwrap_or("—");
    let title = pane_title(&format!("{active_title}{mode_indicator}"), is_focused);

    if let Some(raw_output) = state.workspaces.active_session_output(ws_id) {
        let all_lines = parse_vt100_styled(&raw_output, rows, cols);
        let total = all_lines.len();
        let visible_height = rows as usize;
        let end = total.saturating_sub(state.terminal_scroll_offset);
        let start = end.saturating_sub(visible_height);
        let visible: Vec<Line> = all_lines[start..end].to_vec();

        let scroll_info = if state.terminal_scroll_offset > 0 {
            format!(" [↑{}]", state.terminal_scroll_offset)
        } else {
            String::new()
        };

        let p = Paragraph::new(visible).block(
            Block::default()
                .title(Span::styled(format!("{title}{scroll_info}"), title_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(p, output_area);
    } else {
        let p = Paragraph::new("").block(
            Block::default()
                .title(Span::styled(title, title_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(p, output_area);
    }
}

/// Parse raw PTY bytes through vt100, returning styled ratatui Lines
/// with colors, bold, italic, underline preserved.
fn parse_vt100_styled(raw: &[u8], rows: u16, cols: u16) -> Vec<Line<'static>> {
    let mut parser = vt100::Parser::new(rows.max(24), cols.max(80), 0);
    parser.process(raw);
    let screen = parser.screen();
    let mut lines: Vec<Line<'static>> = Vec::new();

    for row in 0..screen.size().0 {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut current_text = String::new();
        let mut current_style = Style::default();

        for col in 0..screen.size().1 {
            let cell = match screen.cell(row, col) {
                Some(c) => c,
                None => continue,
            };

            let style = cell_to_style(&cell);
            let contents = cell.contents();

            if style == current_style {
                current_text.push_str(&contents);
            } else {
                if !current_text.is_empty() {
                    spans.push(Span::styled(current_text.clone(), current_style));
                    current_text.clear();
                }
                current_style = style;
                current_text.push_str(&contents);
            }
        }

        // Flush last run
        if !current_text.is_empty() {
            // Trim trailing spaces from the last span
            let trimmed = current_text.trim_end().to_string();
            if !trimmed.is_empty() {
                spans.push(Span::styled(trimmed, current_style));
            }
        }

        lines.push(Line::from(spans));
    }

    // Remove trailing empty lines
    while lines.last().is_some_and(|l| l.spans.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push(Line::from(""));
    }
    lines
}

fn cell_to_style(cell: &vt100::Cell) -> Style {
    let mut style = Style::default();

    style = match cell.fgcolor() {
        VtColor::Default => style.fg(Color::White),
        VtColor::Idx(i) => style.fg(ansi256_to_color(i)),
        VtColor::Rgb(r, g, b) => style.fg(Color::Rgb(r, g, b)),
    };

    style = match cell.bgcolor() {
        VtColor::Default => style,
        VtColor::Idx(i) => style.bg(ansi256_to_color(i)),
        VtColor::Rgb(r, g, b) => style.bg(Color::Rgb(r, g, b)),
    };

    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if cell.inverse() {
        style = style.add_modifier(Modifier::REVERSED);
    }

    style
}

fn ansi256_to_color(idx: u8) -> Color {
    match idx {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::Gray,
        _ => Color::Indexed(idx),
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let mode_label = match state.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Insert => "INSERT",
        InputMode::Dialog => "DIALOG",
        InputMode::Command => "COMMAND",
    };

    let ws_name = state
        .active_workspace()
        .map(|ws| ws.name.as_str())
        .unwrap_or("—");

    let ws_count = state.workspaces.list().len();

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(format!(" [{mode_label}]"), Theme::status_bar()),
        Span::styled("  Omniknight", Theme::status_bar()),
        Span::styled(format!("  |  {ws_name}"), Theme::status_bar()),
        Span::styled(format!("  |  {ws_count} workspaces"), Theme::status_bar()),
    ]))
    .style(Theme::status_bar());

    frame.render_widget(bar, area);
}

fn render_hints_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let hints = match (state.focused_pane, state.input_mode) {
        (_, InputMode::Insert) => "Esc:normal mode  (typing goes to terminal)",
        (Pane::SessionList, _) => {
            "j/k:select  Enter/l:terminal  n:new workspace  :::palette  q:quit"
        }
        (Pane::Terminal, _) => {
            "i:insert  t:shell  a:agent  [/]:tabs  h:back  j/k:scroll  :::palette"
        }
    };
    let bar = Paragraph::new(Span::styled(format!(" {hints}"), Theme::hint_bar()));
    frame.render_widget(bar, area);
}

fn pane_styles(is_focused: bool) -> (Style, Style) {
    if is_focused {
        (Theme::active_border(), Theme::focused_title())
    } else {
        (Theme::border(), Theme::unfocused_title())
    }
}

fn pane_title(name: &str, is_focused: bool) -> String {
    if is_focused {
        format!(" ▸ {name} ")
    } else {
        format!(" {name} ")
    }
}
