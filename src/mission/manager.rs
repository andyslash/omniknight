use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::mission::model::{Mission, MissionStatus, MissionStep, StepStatus};

pub struct MissionManager {
    missions: HashMap<Uuid, Mission>,
}

impl MissionManager {
    pub fn new() -> Self {
        Self {
            missions: HashMap::new(),
        }
    }

    pub fn create(&mut self, workspace_id: Uuid, title: String) -> Uuid {
        let id = Uuid::new_v4();
        let mission = Mission {
            id,
            workspace_id,
            title,
            steps: Vec::new(),
            status: MissionStatus::Planning,
            created_at: Utc::now(),
        };
        self.missions.insert(id, mission);
        id
    }

    pub fn get(&self, id: Uuid) -> Option<&Mission> {
        self.missions.get(&id)
    }

    pub fn list(&self) -> Vec<&Mission> {
        self.missions.values().collect()
    }

    pub fn list_for_workspace(&self, workspace_id: Uuid) -> Vec<&Mission> {
        self.missions
            .values()
            .filter(|m| m.workspace_id == workspace_id)
            .collect()
    }

    pub fn add_step(&mut self, mission_id: Uuid, description: String) {
        if let Some(mission) = self.missions.get_mut(&mission_id) {
            mission.steps.push(MissionStep {
                description,
                assigned_agent: None,
                status: StepStatus::Pending,
            });
        }
    }

    pub fn assign_agent(&mut self, mission_id: Uuid, step_index: usize, agent_id: Uuid) {
        if let Some(mission) = self.missions.get_mut(&mission_id) {
            if let Some(step) = mission.steps.get_mut(step_index) {
                step.assigned_agent = Some(agent_id);
            }
        }
    }

    pub fn update_step_status(&mut self, mission_id: Uuid, step_index: usize, status: StepStatus) {
        if let Some(mission) = self.missions.get_mut(&mission_id) {
            if let Some(step) = mission.steps.get_mut(step_index) {
                step.status = status;
            }
        }
    }

    pub fn refresh_status(&mut self, mission_id: Uuid) {
        if let Some(mission) = self.missions.get_mut(&mission_id) {
            if mission.steps.is_empty() {
                return;
            }
            let all_completed = mission
                .steps
                .iter()
                .all(|s| s.status == StepStatus::Completed);
            let any_failed = mission.steps.iter().any(|s| s.status == StepStatus::Failed);
            let any_in_progress = mission
                .steps
                .iter()
                .any(|s| s.status == StepStatus::InProgress);

            mission.status = if all_completed {
                MissionStatus::Completed
            } else if any_failed {
                MissionStatus::Failed
            } else if any_in_progress {
                MissionStatus::InProgress
            } else {
                MissionStatus::Planning
            };
        }
    }
}
