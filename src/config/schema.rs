use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub keybinds: KeybindConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_tick_rate")]
    pub tick_rate_ms: u64,
    #[serde(default = "default_max_agents")]
    pub max_agents: usize,
    #[serde(default = "default_ring_size")]
    pub output_ring_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub defaults: AgentDefaults,
}

#[derive(Debug, Deserialize)]
pub struct AgentDefaults {
    #[serde(default = "default_command")]
    pub command: String,
    #[serde(default = "default_shell")]
    pub shell: String,
    #[serde(default)]
    pub auto_restart: bool,
    #[serde(default = "default_shutdown_timeout")]
    pub graceful_shutdown_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_style")]
    pub style: String,
}

#[derive(Debug, Deserialize)]
pub struct KeybindConfig {
    #[serde(default = "default_leader")]
    pub leader: String,
}

fn default_tick_rate() -> u64 {
    100
}
fn default_max_agents() -> usize {
    20
}
fn default_ring_size() -> usize {
    10000
}
fn default_command() -> String {
    "claude".to_string()
}
fn default_shell() -> String {
    "/bin/zsh".to_string()
}
fn default_shutdown_timeout() -> u64 {
    5
}
fn default_style() -> String {
    "dark".to_string()
}
fn default_leader() -> String {
    ":".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: default_tick_rate(),
            max_agents: default_max_agents(),
            output_ring_size: default_ring_size(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            defaults: AgentDefaults::default(),
        }
    }
}

impl Default for AgentDefaults {
    fn default() -> Self {
        Self {
            command: default_command(),
            shell: default_shell(),
            auto_restart: false,
            graceful_shutdown_timeout_secs: default_shutdown_timeout(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            style: default_style(),
        }
    }
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            leader: default_leader(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            agent: AgentConfig::default(),
            theme: ThemeConfig::default(),
            keybinds: KeybindConfig::default(),
        }
    }
}
