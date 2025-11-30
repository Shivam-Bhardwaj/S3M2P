use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub github: GitHubConfig,
    pub worktree: WorktreeConfig,
    pub agents: AgentsConfig,
    pub limits: LimitsConfig,
    pub brain: BrainConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DaemonConfig {
    pub poll_interval_idle_secs: u64,
    pub poll_interval_active_secs: u64,
    pub poll_interval_very_active_secs: u64,
    pub activity_timeout_minutes: u64,
    pub max_concurrent_automations: usize,
    pub session_timeout_hours: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    pub trigger_pattern: String,
    pub auto_label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorktreeConfig {
    pub base_path: String,
    pub cleanup_after_hours: u64,
    pub branch_prefix: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentsConfig {
    pub planner_agent: String,
    pub planner_model: String,
    pub executor_agent: String,
    pub executor_model: String,
    pub bypass_permissions: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitsConfig {
    pub max_cost_per_issue_usd: f64,
    pub max_planner_cost_usd: f64,
    pub max_executor_cost_usd: f64,
    pub max_tokens_planner: usize,
    pub max_tokens_executor: usize,
    pub daily_automation_limit: usize,
    pub max_concurrent: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BrainConfig {
    pub enabled: bool,
    pub sync_on_pr_merge: bool,
    pub architecture_regeneration_interval_days: u64,
    pub max_recent_changes: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn database_path(&self) -> String {
        dirs::home_dir()
            .unwrap()
            .join(".claude/automation.db")
            .to_str()
            .unwrap()
            .to_string()
    }
}
