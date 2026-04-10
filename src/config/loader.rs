use std::path::PathBuf;

use anyhow::Result;

use crate::config::schema::AppConfig;

const DEFAULT_CONFIG: &str = include_str!("../../config/default.toml");

pub fn load_config() -> Result<AppConfig> {
    let mut config: AppConfig = toml::from_str(DEFAULT_CONFIG)?;

    let user_config_path = user_config_path();
    if user_config_path.exists() {
        let user_toml = std::fs::read_to_string(&user_config_path)?;
        config = toml::from_str(&user_toml)?;
    }

    Ok(config)
}

pub fn user_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omniknight")
        .join("config.toml")
}
