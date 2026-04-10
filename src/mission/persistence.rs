use std::path::PathBuf;

use anyhow::Result;

use crate::mission::model::Mission;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omniknight")
        .join("missions")
}

pub fn save_mission(mission: &Mission) -> Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.toml", mission.id));
    let content = toml::to_string_pretty(mission)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn load_missions() -> Result<Vec<Mission>> {
    let dir = config_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut missions = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(m) = toml::from_str::<Mission>(&content) {
                missions.push(m);
            }
        }
    }
    Ok(missions)
}
