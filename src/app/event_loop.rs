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

        Action::FocusWorkspaces => {
            app.focused_pane = Pane::Workspaces;
        }
        Action::FocusTerminal => {
            if let Some(ws_id) = app.active_workspace_id {
                // Auto-spawn terminal on first focus
                if !app.workspaces.has_sessions(ws_id) {
                    let _ = app.workspaces.spawn_shell(ws_id);
                }
            }
            app.focused_pane = Pane::Terminal;
            app.terminal_scroll_offset = 0;
        }

        Action::NavigateUp => navigate(app, -1),
        Action::NavigateDown => navigate(app, 1),
        Action::NavigateN { direction, count } => {
            for _ in 0..count {
                navigate(app, direction);
            }
        }
        Action::GotoTop => match app.focused_pane {
            Pane::Workspaces => app.selected_workspace = 0,
            Pane::Terminal => app.terminal_scroll_offset = 999999,
        },
        Action::GotoBottom => match app.focused_pane {
            Pane::Workspaces => {
                app.selected_workspace = app.workspaces.list().len().saturating_sub(1);
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

        Action::Select => match app.focused_pane {
            Pane::Workspaces => {
                let workspaces = app.workspaces.list();
                if let Some(ws) = workspaces.get(app.selected_workspace) {
                    let ws_id = ws.id;
                    app.active_workspace_id = Some(ws_id);
                    app.terminal_scroll_offset = 0;
                    // Auto-spawn terminal
                    if !app.workspaces.has_sessions(ws_id) {
                        let _ = app.workspaces.spawn_shell(ws_id);
                    }
                    app.focused_pane = Pane::Terminal;
                }
            }
            Pane::Terminal => {}
        },
        Action::Back => {
            if app.focused_pane == Pane::Terminal {
                app.focused_pane = Pane::Workspaces;
            }
        }

        // Terminal input — forward keypress to workspace PTY
        Action::TerminalInput(key) => {
            if let Some(ws_id) = app.active_workspace_id {
                let bytes = key_to_bytes(key.code);
                let _ = app.workspaces.write_to_active_session(ws_id, &bytes);
            }
        }

        // Sessions
        Action::NewShellSession => {
            if let Some(ws_id) = app.active_workspace_id {
                let _ = app.workspaces.spawn_shell(ws_id);
                app.terminal_scroll_offset = 0;
            }
        }
        Action::SpawnAgent {
            command,
            args,
            title,
        } => {
            if let Some(ws_id) = app.active_workspace_id {
                let _ = app.workspaces.spawn_agent(ws_id, title, &command, &args);
                app.terminal_scroll_offset = 0;
                app.focused_pane = Pane::Terminal;
            }
        }
        Action::NextSession => {
            if let Some(ws_id) = app.active_workspace_id {
                app.workspaces.next_session(ws_id);
                app.terminal_scroll_offset = 0;
            }
        }
        Action::PrevSession => {
            if let Some(ws_id) = app.active_workspace_id {
                app.workspaces.prev_session(ws_id);
                app.terminal_scroll_offset = 0;
            }
        }

        // Workspace
        Action::CreateWorkspace { name, root } => {
            let id = app.workspaces.create(name, root);
            app.active_workspace_id = Some(id);
            // Spawn terminal immediately
            let _ = app.workspaces.spawn_shell(id);
            app.focused_pane = Pane::Terminal;
            app.terminal_scroll_offset = 0;
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
                _ => return,
            };
            app.dialog = Some(dialog);
            app.input_mode = InputMode::Dialog;
        }
        Action::DialogInput(ch) => {
            if let Some(dialog) = &mut app.dialog {
                dialog.input_char(ch);
            }
        }
        Action::DialogBackspace => {
            if let Some(dialog) = &mut app.dialog {
                dialog.backspace();
            }
        }
        Action::DialogNextField => {
            if let Some(dialog) = &mut app.dialog {
                dialog.next_field();
            }
        }
        Action::DialogPrevField => {
            if let Some(dialog) = &mut app.dialog {
                dialog.prev_field();
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
                    _ => {}
                }
            }
        }

        Action::Noop => {}
    }
}

fn navigate(app: &mut AppState, direction: i32) {
    match app.focused_pane {
        Pane::Workspaces => {
            let max = app.workspaces.list().len().saturating_sub(1);
            if direction > 0 {
                app.selected_workspace = (app.selected_workspace + 1).min(max);
            } else {
                app.selected_workspace = app.selected_workspace.saturating_sub(1);
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
