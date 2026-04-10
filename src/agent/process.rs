use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::Result;
use crossbeam_channel::Sender;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use uuid::Uuid;

use crate::agent::state::AgentState;
use crate::app::event::AppEvent;

pub struct AgentProcess {
    pub child: Option<Box<dyn portable_pty::Child + Send>>,
    pub writer: Option<Box<dyn Write + Send>>,
    _reader_handle: Option<std::thread::JoinHandle<()>>,
}

impl AgentProcess {
    pub fn spawn(
        command: &str,
        args: &[String],
        cwd: &Path,
        env: &HashMap<String, String>,
        event_sender: Sender<AppEvent>,
        agent_id: Uuid,
    ) -> Result<Self> {
        let pty_system = NativePtySystem::default();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);
        cmd.cwd(cwd);
        for (k, v) in env {
            cmd.env(k, v);
        }

        let child = pty_pair.slave.spawn_command(cmd)?;
        let reader = pty_pair.master.try_clone_reader()?;
        let writer = pty_pair.master.take_writer()?;

        let sender = event_sender;
        let reader_handle = std::thread::spawn(move || {
            let buf_reader = BufReader::new(reader);
            for line in buf_reader.lines() {
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
            let _ = sender.send(AppEvent::AgentStateChange {
                agent_id,
                state: AgentState::Completed,
            });
        });

        Ok(Self {
            child: Some(child),
            writer: Some(writer),
            _reader_handle: Some(reader_handle),
        })
    }

    pub fn kill(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
        }
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            child.try_wait().ok().flatten().is_none()
        } else {
            false
        }
    }
}
