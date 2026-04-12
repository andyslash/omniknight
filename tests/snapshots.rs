use ratatui::backend::TestBackend;
use ratatui::Terminal;

use omniknight::app::state::AppState;
use omniknight::ui::layout;

fn render_to_string(state: &AppState, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            layout::render(frame, state);
        })
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            output.push_str(cell.symbol());
        }
        output.push('\n');
    }
    output
}

#[test]
fn snapshot_empty() {
    let state = AppState::new();
    let output = render_to_string(&state, 100, 20);
    insta::assert_snapshot!("empty", output);
}

#[test]
fn snapshot_with_workspace() {
    let mut state = AppState::new();
    let id = state
        .workspaces
        .create("my-project".to_string(), "/tmp/test".into());
    state.active_session = Some((id, 0));
    let output = render_to_string(&state, 100, 20);
    insta::assert_snapshot!("with_workspace", output);
}
