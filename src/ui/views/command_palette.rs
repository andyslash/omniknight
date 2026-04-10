use ratatui::widgets::Clear;
use ratatui::Frame;

use crate::app::state::AppState;
use crate::ui::widgets::fuzzy_input;

pub fn render_overlay(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let popup_area = fuzzy_input::centered_rect(50, 40, area);

    frame.render_widget(Clear, popup_area);
    fuzzy_input::render(frame, popup_area, &state.command_palette);
}
