use std::io::Stdout;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{select, Receiver};
use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::action::Action;
use crate::app::event::AppEvent;
use crate::app::state::{AppState, View};
use crate::keybinds::handler::handle_key;
use crate::keybinds::mode::InputMode;
use crate::ui::layout;

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut AppState,
    event_rx: Receiver<AppEvent>,
) -> anyhow::Result<()> {
    let tick_rate = Duration::from_millis(100);

    // Spawn crossterm event polling thread
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

    // Main loop
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
            AppEvent::AgentOutput { agent_id, line } => {
                if let Some(agent) = app.agents.get_mut(*agent_id) {
                    agent.output.push(line.clone());
                    agent.metrics.lines_output += 1;
                }
                Action::Noop
            }
            AppEvent::AgentStateChange { agent_id, state } => {
                if let Some(agent) = app.agents.get_mut(*agent_id) {
                    agent.state = *state;
                }
                Action::Noop
            }
        };

        apply_action(app, action);

        if app.should_quit {
            app.agents.kill_all();
            break;
        }
    }

    Ok(())
}

fn apply_action(app: &mut AppState, action: Action) {
    match action {
        Action::Quit => app.should_quit = true,
        Action::SwitchView(view) => app.switch_view(view),
        Action::SetInputMode(mode) => app.input_mode = mode,
        Action::NavigateUp => match app.active_view {
            View::Home => {
                app.selected_workspace = app.selected_workspace.saturating_sub(1);
            }
            View::MissionControl => {
                app.selected_agent = app.selected_agent.saturating_sub(1);
            }
            View::Workspace => {
                app.selected_mission = app.selected_mission.saturating_sub(1);
            }
            View::Terminal => {}
        },
        Action::NavigateDown => match app.active_view {
            View::Home => {
                let max = app.workspaces.list().len().saturating_sub(1);
                app.selected_workspace = (app.selected_workspace + 1).min(max);
            }
            View::MissionControl => {
                let max = app.agents.list().len().saturating_sub(1);
                app.selected_agent = (app.selected_agent + 1).min(max);
            }
            View::Workspace => {
                let max = app.missions.list().len().saturating_sub(1);
                app.selected_mission = (app.selected_mission + 1).min(max);
            }
            View::Terminal => {}
        },
        Action::Select => {
            if let View::Home = app.active_view {
                let workspaces = app.workspaces.list();
                if let Some(ws) = workspaces.get(app.selected_workspace) {
                    app.active_workspace_id = Some(ws.id);
                    app.switch_view(View::Workspace);
                }
            }
        }
        Action::Back => {
            app.switch_view(View::Home);
        }
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
        Action::KillAgent(id) => {
            let _ = app.agents.kill_agent(id);
        }
        Action::SwitchTerminalTab(idx) => {
            app.terminals.switch_tab(idx);
            app.selected_terminal_tab = idx;
        }
        Action::NewTerminalTab => {
            let idx = app.terminals.create_tab("shell".to_string());
            app.selected_terminal_tab = idx;
        }
        Action::SwitchWorkspace(id) => {
            app.active_workspace_id = Some(id);
        }
        Action::LaunchAgent { .. }
        | Action::PauseAgent(_)
        | Action::ResumeAgent(_)
        | Action::FocusAgent(_)
        | Action::CreateWorkspace { .. }
        | Action::CloseTerminalTab(_)
        | Action::TerminalInput(_)
        | Action::CreateMission { .. }
        | Action::Noop => {}
    }
}
