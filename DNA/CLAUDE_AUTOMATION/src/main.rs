use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod config;
mod github;
mod state;
mod worktree;
mod session;
mod agent_router;

use config::Config;
use state::Database;
use github::GitHubClient;
use agent_router::Agent;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("claude_automation=info")
        .init();

    // Load configuration
    let config = Config::load("TOOLS/CLAUDE_AUTOMATION/config.toml")?;
    info!("Configuration loaded");

    // Initialize database
    let db = Database::open(&config.database_path())?;
    info!("Database initialized at {}", config.database_path());

    // Initialize GitHub client
    let github = GitHubClient::new(&config)?;
    info!("GitHub client initialized for {}/{}", config.github.owner, config.github.repo);

    info!("🤖 Claude Automation Daemon started");
    info!("   Polling every {}s for fast, reliable responses", config.daemon.poll_interval_secs);

    loop {
        info!("Polling...");

        // Poll for new triggers (issues labeled with claude-auto)
        match github.poll_triggers().await {
            Ok(new_issues) => {
                if !new_issues.is_empty() {
                    info!("Found {} issue(s) with trigger label", new_issues.len());
                }
                for issue in new_issues {
                    // Only spawn if we haven't created automation record yet
                    if db.automation_exists(issue.number).unwrap_or(false) {
                        continue; // Skip - already processing
                    }

                    info!("New issue #{}: {} - spawning Planner (Opus)", issue.number, issue.title);

                    match session::spawn_planner(&issue, &config, &db).await {
                        Ok(_) => {
                            info!("Successfully spawned Planner for issue #{}", issue.number);
                        }
                        Err(e) => {
                            error!("Failed to spawn Planner for issue #{}: {}", issue.number, e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to poll triggers: {:?}", e);
            }
        }

        // Poll for new comments on active issues
        match db.get_active_issues() {
            Ok(active_issues) => {
                for issue_num in active_issues {
                    match github.get_new_comments(issue_num, db.last_comment_time(issue_num)?).await {
                        Ok(new_comments) if !new_comments.is_empty() => {
                            info!("Issue #{}: {} new comment(s)", issue_num, new_comments.len());

                            // Determine which agent to spawn
                            let agent = agent_router::decide(&new_comments, &db)?;

                            match agent {
                                Agent::Planner => {
                                    info!("Spawning Planner (Opus) for re-planning issue #{}", issue_num);
                                    session::spawn_planner_with_context(issue_num, &config, &db).await?;
                                }
                                Agent::Executor => {
                                    info!("Spawning Executor (Sonnet) for iteration on issue #{}", issue_num);
                                    session::spawn_executor(issue_num, &config, &db).await?;
                                }
                            }

                            // Update conversation history
                            db.add_comments(issue_num, &new_comments)?;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Failed to get comments for issue #{}: {}", issue_num, e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get active issues: {}", e);
            }
        }

        // Monitor PR comments for automation-created PRs
        match github.get_automation_prs().await {
            Ok(prs) => {
                for (pr_number, issue_number) in prs {
                    match github.get_pr_comments(pr_number, db.last_comment_time(issue_number)?).await {
                        Ok(new_comments) if !new_comments.is_empty() => {
                            info!("PR #{} (for issue #{}): {} new comment(s)", pr_number, issue_number, new_comments.len());

                            // Always use Executor for PR feedback (quick fixes)
                            info!("Spawning Executor (Sonnet) for PR #{} feedback", pr_number);
                            if let Err(e) = session::spawn_executor(issue_number, &config, &db).await {
                                error!("Failed to spawn Executor for PR: {}", e);
                            }

                            // Update conversation history
                            db.add_comments(issue_number, &new_comments)?;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Failed to get PR comments for #{}: {}", pr_number, e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get automation PRs: {}", e);
            }
        }

        // Monitor active sessions and enforce budgets
        if let Err(e) = session::monitor_sessions(&db, &config).await {
            warn!("Session monitoring error: {}", e);
        }

        // Cleanup old worktrees and resources
        if let Err(e) = worktree::cleanup_old_worktrees(&db, &config).await {
            warn!("Cleanup error: {}", e);
        }

        sleep(Duration::from_secs(config.daemon.poll_interval_secs)).await;
    }
}
