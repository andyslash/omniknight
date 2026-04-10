use std::io::BufRead;

use crossbeam_channel::Sender;
use uuid::Uuid;

use crate::app::event::AppEvent;

/// Spawns a thread that reads lines from a reader and sends them as AgentOutput events.
pub fn bridge_agent_output<R: BufRead + Send + 'static>(
    reader: R,
    agent_id: Uuid,
    sender: Sender<AppEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for line in reader.lines() {
            match line {
                Ok(text) => {
                    if sender
                        .send(AppEvent::AgentOutput {
                            agent_id,
                            line: text,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    })
}
