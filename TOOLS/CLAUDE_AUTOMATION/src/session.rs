use anyhow::{Context, Result};
use std::process::Command;
use crate::config::Config;
use crate::state::Database;
use crate::github::Issue;
use crate::worktree;

/// Spawn Planner agent (Opus) for initial planning
pub async fn spawn_planner(issue: &Issue, config: &Config, db: &Database) -> Result<()> {
    // Create worktree
    let worktree_path = worktree::create_automation_worktree(issue, config)?;

    // Create automation record
    db.create_automation(issue.number, worktree_path.to_str().unwrap())?;

    // Extract project from labels
    let project = worktree::extract_project_from_labels(&issue.labels)
        .context("No project label found")?;

    tracing::info!("Spawning Planner (Opus) for issue #{} in {:?}", issue.number, worktree_path);

    // Spawn claude with planner agent
    let output = Command::new("claude")
        .args([
            "--agent", &config.agents.planner_agent,
            "--model", &config.agents.planner_model,
        ])
        .env("ISSUE_NUMBER", issue.number.to_string())
        .env("PROJECT", project)
        .current_dir(&worktree_path)
        .spawn()
        .context("Failed to spawn claude process")?;

    // Mark as running
    db.update_status(issue.number, "running")?;

    tracing::info!("Planner spawned with PID: {:?}", output.id());

    Ok(())
}

/// Spawn Planner with existing conversation context (re-planning)
pub async fn spawn_planner_with_context(issue_number: u64, config: &Config, db: &Database) -> Result<()> {
    // Get worktree path from database
    let worktree_path: String = db.conn.query_row(
        "SELECT worktree_path FROM automations WHERE issue_number = ?1",
        rusqlite::params![issue_number as i64],
        |row| row.get(0),
    )?;

    // Get conversation history
    let mut stmt = db.conn.prepare(
        "SELECT author, body, created_at FROM conversation_history
         WHERE issue_number = ?1 ORDER BY created_at ASC"
    )?;

    let history: Vec<(String, String, String)> = stmt.query_map(
        rusqlite::params![issue_number as i64],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    )?
    .filter_map(|r| r.ok())
    .collect();

    let history_json = serde_json::to_string(&history)?;

    tracing::info!("Spawning Planner (Opus) with context for issue #{}", issue_number);

    // Spawn claude with context
    Command::new("claude")
        .args([
            "--agent", &config.agents.planner_agent,
            "--model", &config.agents.planner_model,
        ])
        .env("ISSUE_NUMBER", issue_number.to_string())
        .env("CONVERSATION_HISTORY", history_json)
        .current_dir(&worktree_path)
        .spawn()
        .context("Failed to spawn claude process")?;

    Ok(())
}

/// Spawn Executor agent (Sonnet) for implementation
pub async fn spawn_executor(issue_number: u64, config: &Config, db: &Database) -> Result<()> {
    // Get worktree path
    let worktree_path: String = db.conn.query_row(
        "SELECT worktree_path FROM automations WHERE issue_number = ?1",
        rusqlite::params![issue_number as i64],
        |row| row.get(0),
    )?;

    tracing::info!("Spawning Executor (Sonnet) for issue #{}", issue_number);

    // Spawn claude with executor agent
    Command::new("claude")
        .args([
            "--agent", &config.agents.executor_agent,
            "--model", &config.agents.executor_model,
        ])
        .env("ISSUE_NUMBER", issue_number.to_string())
        .current_dir(&worktree_path)
        .spawn()
        .context("Failed to spawn claude process")?;

    Ok(())
}

/// Monitor active Claude sessions
pub async fn monitor_sessions(_db: &Database, _config: &Config) -> Result<()> {
    // TODO: Track running Claude processes
    // - Check if sessions are still alive
    // - Enforce budget limits
    // - Timeout long-running sessions
    Ok(())
}
