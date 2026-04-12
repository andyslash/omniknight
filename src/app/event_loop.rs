use std::io::Stdout;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{select, Receiver};
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::action::Action;
use crate::app::event::AppEvent;
use crate::app::state::{AppState, Pane};
use crate::keybinds::handler::handle_key;
use crate::keybinds::mode::InputMode;
use crate::ui::layout;
use crate::ui::widgets::input_dialog::{DialogIntent, InputDialogState};
use crate::workspace::manager::SessionTreeEntry;

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut AppState,
    event_rx: Receiver<AppEvent>,
) -> anyhow::Result<()> {
    let tick_rate = Duration::from_millis(100);

    let (input_tx, input_rx) = crossbeam_channel::unbounded();
    thread::spawn(move || loop {
        if event::poll(tick_rate).unwrap_or(false) {
            if let Ok(evt) = event::read() {
                let app_event = match evt {
                    Event::Key(key) => AppEvent::Key(key),
                    Event::Resize(w, h) => AppEvent::Resize(w, h),
                    _ => continue,
                };
                if input_tx.send(app_event).is_err() {
                    break;
                }
            }
        } else if input_tx.send(AppEvent::Tick).is_err() {
            break;
        }
    });

    loop {
        terminal.draw(|frame| {
            layout::render(frame, app);
        })?;

        let event = select! {
            recv(input_rx) -> msg => match msg {
                Ok(e) => e,
                Err(_) => break,
            },
            recv(event_rx) -> msg => match msg {
                Ok(e) => e,
                Err(_) => {
                    match input_rx.recv() {
                        Ok(e) => e,
                        Err(_) => break,
                    }
                }
            },
        };

        let action = match &event {
            AppEvent::Key(key) => handle_key(*key, app),
            AppEvent::Tick => Action::Noop,
            AppEvent::Resize(_, _) => Action::Noop,
            AppEvent::WorkspaceOutput { .. } => Action::Noop,
        };

        apply_action(app, action);

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn apply_action(app: &mut AppState, action: Action) {
    match action {
        Action::Quit => app.should_quit = true,
        Action::SetInputMode(mode) => app.input_mode = mode,

        Action::FocusSessionList => {
            app.focused_pane = Pane::SessionList;
        }
        Action::FocusTerminal => {
            if app.active_session.is_some() {
                app.focused_pane = Pane::Terminal;
                app.terminal_scroll_offset = 0;
            }
        }

        Action::NavigateUp => navigate(app, -1),
        Action::NavigateDown => navigate(app, 1),
        Action::NavigateN { direction, count } => {
            for _ in 0..count {
                navigate(app, direction);
            }
        }
        Action::GotoTop => match app.focused_pane {
            Pane::SessionList => app.selected_index = 0,
            Pane::Terminal => app.terminal_scroll_offset = 999999,
        },
        Action::GotoBottom => match app.focused_pane {
            Pane::SessionList => {
                let tree = app.workspaces.session_tree();
                app.selected_index = tree.len().saturating_sub(1);
            }
            Pane::Terminal => app.terminal_scroll_offset = 0,
        },
        Action::PageDown => {
            for _ in 0..10 {
                navigate(app, 1);
            }
        }
        Action::PageUp => {
            for _ in 0..10 {
                navigate(app, -1);
            }
        }

        Action::Select => {
            let tree = app.workspaces.session_tree();
            if let Some(entry) = tree.get(app.selected_index) {
                match entry {
                    SessionTreeEntry::WorkspaceHeader { workspace_id, .. } => {
                        let ws_id = *workspace_id;
                        // If workspace has no sessions, spawn shell and activate
                        if !app.workspaces.has_sessions(ws_id) {
                            if let Ok(idx) = app.workspaces.spawn_shell(ws_id) {
                                app.active_session = Some((ws_id, idx));
                                app.focused_pane = Pane::Terminal;
                                app.terminal_scroll_offset = 0;
                            }
                        } else {
                            // Toggle collapse
                            app.workspaces.toggle_collapse(ws_id);
                        }
                    }
                    SessionTreeEntry::SessionItem {
                        workspace_id,
                        session_idx,
                        ..
                    } => {
                        app.active_session = Some((*workspace_id, *session_idx));
                        app.focused_pane = Pane::Terminal;
                        app.terminal_scroll_offset = 0;
                    }
                }
            }
        }
        Action::ToggleCollapse => {
            let tree = app.workspaces.session_tree();
            if let Some(entry) = tree.get(app.selected_index) {
                let ws_id = match entry {
                    SessionTreeEntry::WorkspaceHeader { workspace_id, .. } => *workspace_id,
                    SessionTreeEntry::SessionItem { workspace_id, .. } => *workspace_id,
                };
                app.workspaces.toggle_collapse(ws_id);
            }
        }
        Action::Back => {
            if app.focused_pane == Pane::Terminal {
                app.focused_pane = Pane::SessionList;
            }
        }

        // Terminal input
        Action::TerminalInput(key) => {
            if let Some((ws_id, sess_idx)) = app.active_session {
                let bytes = key_to_bytes(key.code);
                let _ = app.workspaces.write_to_session(ws_id, sess_idx, &bytes);
            }
        }

        // Sessions
        Action::NewShellSession => {
            if let Some(ws_id) = resolve_workspace(app) {
                if let Ok(idx) = app.workspaces.spawn_shell(ws_id) {
                    app.active_session = Some((ws_id, idx));
                    app.terminal_scroll_offset = 0;
                    app.focused_pane = Pane::Terminal;
                }
            }
        }
        Action::SpawnAgent {
            command,
            args,
            title,
        } => {
            if let Some(ws_id) = resolve_workspace(app) {
                if let Ok(idx) = app.workspaces.spawn_agent(ws_id, title, &command, &args) {
                    app.active_session = Some((ws_id, idx));
                    app.terminal_scroll_offset = 0;
                    app.focused_pane = Pane::Terminal;
                }
            }
        }
        Action::NextSession => {
            if let Some((ws_id, _)) = app.active_session {
                app.workspaces.next_session(ws_id);
                // Sync active_session with manager
                if let Some(mw) = app.workspaces.list().iter().position(|w| w.id == ws_id) {
                    let tree = app.workspaces.session_tree();
                    // Find the new active session idx from manager
                    for (i, entry) in tree.iter().enumerate() {
                        if let SessionTreeEntry::SessionItem {
                            workspace_id,
                            session_idx,
                            ..
                        } = entry
                        {
                            if *workspace_id == ws_id {
                                // Check if this is now the active one
                                let titles = app.workspaces.session_titles(ws_id);
                                if titles.get(*session_idx).is_some_and(|(_, active)| *active) {
                                    app.active_session = Some((ws_id, *session_idx));
                                    app.selected_index = i;
                                    break;
                                }
                            }
                        }
                    }
                }
                app.terminal_scroll_offset = 0;
            }
        }
        Action::PrevSession => {
            if let Some((ws_id, _)) = app.active_session {
                app.workspaces.prev_session(ws_id);
                let tree = app.workspaces.session_tree();
                for (i, entry) in tree.iter().enumerate() {
                    if let SessionTreeEntry::SessionItem {
                        workspace_id,
                        session_idx,
                        ..
                    } = entry
                    {
                        if *workspace_id == ws_id {
                            let titles = app.workspaces.session_titles(ws_id);
                            if titles.get(*session_idx).is_some_and(|(_, active)| *active) {
                                app.active_session = Some((ws_id, *session_idx));
                                app.selected_index = i;
                                break;
                            }
                        }
                    }
                }
                app.terminal_scroll_offset = 0;
            }
        }

        // Workspace
        Action::CreateWorkspace { name, root } => {
            let id = app.workspaces.create(name, root);
            if let Ok(idx) = app.workspaces.spawn_shell(id) {
                app.active_session = Some((id, idx));
                app.focused_pane = Pane::Terminal;
                app.terminal_scroll_offset = 0;
            }
        }

        // Command palette
        Action::OpenCommandPalette => {
            app.command_palette.is_open = true;
            app.input_mode = InputMode::Command;
        }
        Action::CloseCommandPalette => {
            app.command_palette.is_open = false;
            app.command_palette.input.clear();
            app.command_palette.selected = 0;
            app.input_mode = InputMode::Normal;
        }
        Action::CommandPaletteInput(ch) => {
            app.command_palette.input.push(ch);
            app.command_palette.filter();
        }
        Action::CommandPaletteBackspace => {
            app.command_palette.input.pop();
            app.command_palette.filter();
        }
        Action::CommandPaletteSelect => {
            if let Some(action) = app.command_palette.selected_action() {
                let action = action.clone();
                app.command_palette.is_open = false;
                app.command_palette.input.clear();
                app.input_mode = InputMode::Normal;
                apply_action(app, action);
            }
        }
        Action::CommandPaletteNavigateUp => {
            if app.command_palette.selected > 0 {
                app.command_palette.selected -= 1;
            }
        }
        Action::CommandPaletteNavigateDown => {
            let max = app.command_palette.filtered.len().saturating_sub(1);
            if app.command_palette.selected < max {
                app.command_palette.selected += 1;
            }
        }

        // Dialog
        Action::OpenDialog(intent) => {
            let dialog = match intent {
                DialogIntent::CreateWorkspace => InputDialogState::workspace_dialog(),
                DialogIntent::LaunchAgent => InputDialogState::agent_dialog(),
            };
            app.dialog = Some(dialog);
            app.input_mode = InputMode::Dialog;
        }
        Action::DialogInput(ch) => {
            if let Some(d) = &mut app.dialog {
                d.input_char(ch);
            }
        }
        Action::DialogBackspace => {
            if let Some(d) = &mut app.dialog {
                d.backspace();
            }
        }
        Action::DialogNextField => {
            if let Some(d) = &mut app.dialog {
                d.next_field();
            }
        }
        Action::DialogPrevField => {
            if let Some(d) = &mut app.dialog {
                d.prev_field();
            }
        }
        Action::DialogCancel => {
            app.dialog = None;
            app.input_mode = InputMode::Normal;
        }
        Action::DialogConfirm => {
            if let Some(dialog) = app.dialog.take() {
                app.input_mode = InputMode::Normal;
                match dialog.intent {
                    DialogIntent::CreateWorkspace => {
                        let name = dialog.fields[0].value.trim().to_string();
                        if name.is_empty() {
                            return;
                        }
                        let root = std::path::PathBuf::from(dialog.fields[1].value.trim());
                        apply_action(app, Action::CreateWorkspace { name, root });
                    }
                    DialogIntent::LaunchAgent => {
                        let raw = dialog.fields[0].value.trim().to_string();
                        if raw.is_empty() {
                            return;
                        }
                        let mut parts = raw.split_whitespace();
                        let command = parts.next().unwrap().to_string();
                        let args: Vec<String> = parts.map(String::from).collect();
                        let title = dialog.fields[1].value.trim().to_string();
                        let title = if title.is_empty() { raw.clone() } else { title };
                        apply_action(
                            app,
                            Action::SpawnAgent {
                                command,
                                args,
                                title,
                            },
                        );
                    }
                }
            }
        }

        Action::Noop => {}
    }
}

/// Resolve which workspace the current action should target.
/// Uses the selected tree entry's workspace, or the active session's workspace.
fn resolve_workspace(app: &mut AppState) -> Option<uuid::Uuid> {
    // First try from selected tree entry
    let ws_id = app.workspaces.workspace_for_tree_index(app.selected_index);
    if ws_id.is_some() {
        return ws_id;
    }
    // Fallback to active session
    app.active_session.map(|(ws_id, _)| ws_id)
}

fn navigate(app: &mut AppState, direction: i32) {
    match app.focused_pane {
        Pane::SessionList => {
            let tree = app.workspaces.session_tree();
            let max = tree.len().saturating_sub(1);
            if direction > 0 {
                app.selected_index = (app.selected_index + 1).min(max);
            } else {
                app.selected_index = app.selected_index.saturating_sub(1);
            }
        }
        Pane::Terminal => {
            if direction > 0 {
                app.terminal_scroll_offset = app.terminal_scroll_offset.saturating_sub(1);
            } else {
                app.terminal_scroll_offset += 1;
            }
        }
    }
}

fn key_to_bytes(code: KeyCode) -> Vec<u8> {
    match code {
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            s.as_bytes().to_vec()
        }
        KeyCode::Enter => vec![13],
        KeyCode::Backspace => vec![127],
        KeyCode::Tab => vec![9],
        KeyCode::Up => vec![27, 91, 65],
        KeyCode::Down => vec![27, 91, 66],
        KeyCode::Right => vec![27, 91, 67],
        KeyCode::Left => vec![27, 91, 68],
        _ => vec![],
    }
}
