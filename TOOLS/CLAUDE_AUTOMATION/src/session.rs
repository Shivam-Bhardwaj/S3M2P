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

    // Read planner agent prompt from .claude/agents/planner.md
    let agent_path = "/home/curious/S3M2P/.claude/agents/planner.md";
    let agent_content = std::fs::read_to_string(agent_path)
        .context("Failed to read planner agent file")?;

    // Extract prompt after YAML frontmatter
    let prompt = agent_content
        .split("---")
        .nth(2)
        .unwrap_or(&agent_content)
        .trim();

    // Create initial prompt file for Claude to process
    let prompt_file = worktree_path.join(".claude_prompt");
    std::fs::write(&prompt_file, format!(
        "You are working on GitHub issue #{}. Use github_issue_read({}) to analyze and create an implementation plan. Post the plan to the issue using github_issue_comment().",
        issue.number, issue.number
    ))?;

    // Spawn claude interactively (can use MCP tools)
    let output = Command::new("claude")
        .args([
            "--model", &config.agents.planner_model,
            "--append-system-prompt", prompt,
            "--permission-mode", "bypassPermissions",
        ])
        .stdin(std::fs::File::open(&prompt_file)?)
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

    // Read planner agent prompt
    let agent_path = "/home/curious/S3M2P/.claude/agents/planner.md";
    let agent_content = std::fs::read_to_string(agent_path)?;
    let prompt = agent_content.split("---").nth(2).unwrap_or(&agent_content).trim();

    // Create prompt file for interactive mode
    let prompt_file = std::path::PathBuf::from(&worktree_path).join(".claude_prompt_replanning");
    std::fs::write(&prompt_file, format!(
        "Re-planning issue #{}. Previous conversation: {}. Use github_issue_read({}) to get current issue state and post your updated plan using github_issue_comment().",
        issue_number, history_json, issue_number
    ))?;

    // Spawn claude with context (interactive mode for GitHub posting)
    let output = Command::new("claude")
        .args([
            "--model", &config.agents.planner_model,
            "--append-system-prompt", prompt,
            "--permission-mode", "bypassPermissions",
        ])
        .stdin(std::fs::File::open(&prompt_file)?)
        .env("ISSUE_NUMBER", issue_number.to_string())
        .current_dir(&worktree_path)
        .spawn()
        .context("Failed to spawn claude process")?;

    // Mark as running
    db.update_status(issue_number, "running")?;

    tracing::info!("Re-planner spawned with PID: {:?}", output.id());

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

    // Extract project from labels
    let github = crate::github::GitHubClient::new(config)?;
    let issue = github.fetch_issue(issue_number).await?;
    let project = worktree::extract_project_from_labels(&issue.labels)
        .context("No project label found")?;

    tracing::info!("Spawning Executor (Sonnet) for issue #{}", issue_number);

    // Read executor agent prompt
    let agent_path = "/home/curious/S3M2P/.claude/agents/executor.md";
    let agent_content = std::fs::read_to_string(agent_path)?;
    let prompt = agent_content.split("---").nth(2).unwrap_or(&agent_content).trim();

    // Create prompt file for interactive mode
    let prompt_file = std::path::PathBuf::from(&worktree_path).join(".claude_executor_prompt");
    std::fs::write(&prompt_file, format!(
        "Execute the plan for issue #{}. Use github_issue_read({}) to get details.",
        issue_number, issue_number
    ))?;

    // Spawn claude interactively (can use MCP tools for GitHub posting)
    let output = Command::new("claude")
        .args([
            "--model", &config.agents.executor_model,
            "--append-system-prompt", prompt,
            "--permission-mode", "bypassPermissions",
        ])
        .stdin(std::fs::File::open(&prompt_file)?)
        .env("ISSUE_NUMBER", issue_number.to_string())
        .env("PROJECT", project)
        .current_dir(&worktree_path)
        .spawn()
        .context("Failed to spawn claude process")?;

    // Mark as running
    db.update_status(issue_number, "running")?;

    tracing::info!("Executor spawned with PID: {:?}", output.id());

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
