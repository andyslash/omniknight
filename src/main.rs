mod agent;
mod app;
mod config;
mod ipc;
mod keybinds;
mod mission;
mod terminal;
mod ui;
mod workspace;

use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::event_loop;
use crate::app::state::AppState;
use crate::ipc::bus::EventBus;

fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and event bus
    let mut app = AppState::new();
    let event_bus = EventBus::new();
    app.agents.set_event_sender(event_bus.sender());

    // Run the main event loop
    let result = event_loop::run(&mut terminal, &mut app, event_bus.receiver().clone());

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
