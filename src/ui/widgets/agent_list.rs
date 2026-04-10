use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::agent::manager::Agent;
use crate::agent::state::AgentState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, agents: &[&Agent], selected: usize) {
    let items: Vec<ListItem> = agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let bullet = match agent.state {
                AgentState::Running => "●",
                AgentState::Paused => "◐",
                AgentState::Idle => "○",
                AgentState::Completed => "✓",
                AgentState::Failed => "✕",
                AgentState::Killed => "⊘",
            };

            let style = if i == selected {
                Theme::selected()
            } else {
                Theme::agent_state(agent.state)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {bullet} "), Theme::agent_state(agent.state)),
                Span::styled(
                    format!("{:<20} {:>10}", agent.name, agent.state.label()),
                    style,
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Agents ")
            .borders(Borders::ALL)
            .border_style(Theme::active_border()),
    );

    frame.render_widget(list, area);
}
