use std::path::PathBuf;

use anyhow::Result;

use crate::workspace::model::Workspace;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omniknight")
        .join("workspaces")
}

pub fn save_workspace(ws: &Workspace) -> Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.toml", ws.id));
    let content = toml::to_string_pretty(ws)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn load_workspaces() -> Result<Vec<Workspace>> {
    let dir = config_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut workspaces = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(ws) = toml::from_str::<Workspace>(&content) {
                workspaces.push(ws);
            }
        }
    }
    Ok(workspaces)
}

pub fn delete_workspace_file(id: uuid::Uuid) -> Result<()> {
    let path = config_dir().join(format!("{}.toml", id));
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
