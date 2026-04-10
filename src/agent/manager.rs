use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use crossbeam_channel::Sender;
use uuid::Uuid;

use crate::agent::output::RingBuffer;
use crate::agent::process::AgentProcess;
use crate::agent::state::{AgentMetrics, AgentState};
use crate::app::event::AppEvent;

pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub workspace_id: Option<Uuid>,
    pub mission_id: Option<Uuid>,
    pub state: AgentState,
    pub task_description: String,
    pub output: RingBuffer<String>,
    pub metrics: AgentMetrics,
    pub process: Option<AgentProcess>,
}

pub struct AgentManager {
    agents: HashMap<Uuid, Agent>,
    event_sender: Option<Sender<AppEvent>>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            event_sender: None,
        }
    }

    pub fn set_event_sender(&mut self, sender: Sender<AppEvent>) {
        self.event_sender = Some(sender);
    }

    pub fn spawn_agent(
        &mut self,
        name: String,
        command: &str,
        args: &[String],
        cwd: &Path,
        env: &HashMap<String, String>,
        task_description: String,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let sender = self
            .event_sender
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Event sender not set"))?;

        let process = AgentProcess::spawn(command, args, cwd, env, sender, id)?;

        let agent = Agent {
            id,
            name,
            workspace_id: None,
            mission_id: None,
            state: AgentState::Running,
            task_description,
            output: RingBuffer::new(10000),
            metrics: AgentMetrics {
                started_at: Some(Utc::now()),
                ..Default::default()
            },
            process: Some(process),
        };

        self.agents.insert(id, agent);
        Ok(id)
    }

    pub fn kill_agent(&mut self, id: Uuid) -> Result<()> {
        if let Some(agent) = self.agents.get_mut(&id) {
            if let Some(mut process) = agent.process.take() {
                process.kill()?;
            }
            agent.state = AgentState::Killed;
            agent.metrics.finished_at = Some(Utc::now());
        }
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Option<&Agent> {
        self.agents.get(&id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    pub fn list(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    pub fn active_count(&self) -> usize {
        self.agents.values().filter(|a| a.state.is_active()).count()
    }

    pub fn kill_all(&mut self) {
        let ids: Vec<Uuid> = self.agents.keys().copied().collect();
        for id in ids {
            let _ = self.kill_agent(id);
        }
    }
}
