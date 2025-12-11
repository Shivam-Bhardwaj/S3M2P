//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: worktree.rs | DNA/CLAUDE_AUTOMATION/src/worktree.rs
//! PURPOSE: Provides 2 public functions for src
//! MODIFIED: 2025-12-02
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::config::Config;
use crate::github::Issue;
use crate::state::Database;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Extract project name from issue labels
pub fn extract_project_from_labels(labels: &[String]) -> Option<String> {
    for label in labels {
        if let Some(project) = label.strip_prefix("project:") {
            return Some(project.to_lowercase());
        }
    }
    None
}

/// Create a worktree for an issue
pub fn create_automation_worktree(issue: &Issue, config: &Config) -> Result<PathBuf> {
    let project =
        extract_project_from_labels(&issue.labels).context("No project: label found on issue")?;

    let base = PathBuf::from(&config.worktree.base_path);
    std::fs::create_dir_all(&base)?;

    let worktree_name = format!("{}-{}", project, issue.number);
    let path = base.join(&worktree_name);

    if !path.exists() {
        let branch_name = format!("{}/issue-{}", config.worktree.branch_prefix, issue.number);

        tracing::info!("Creating worktree at {:?} for branch {}", path, branch_name);

        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                &branch_name,
                path.to_str().unwrap(),
                "main",
            ])
            .current_dir("/home/curious/S3M2P")
            .output()
            .context("Failed to create worktree")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git worktree creation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        tracing::info!("Worktree created successfully: {:?}", path);
    } else {
        tracing::info!("Worktree already exists: {:?}", path);
    }

    Ok(path)
}

/// Cleanup old worktrees for closed/completed issues
pub async fn cleanup_old_worktrees(_db: &Database, config: &Config) -> Result<()> {
    let base = PathBuf::from(&config.worktree.base_path);

    if !base.exists() {
        return Ok(());
    }

    // List all worktrees
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir("/home/curious/S3M2P")
        .output()?;

    if !output.status.success() {
        return Ok(());
    }

    let worktrees = String::from_utf8_lossy(&output.stdout);

    // Find worktrees in the auto path that are old
    for line in worktrees.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if path.contains("/worktrees/auto/") {
                // Check if worktree is old (based on last modification)
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        let age = std::time::SystemTime::now()
                            .duration_since(modified)
                            .unwrap_or_default();

                        // Cleanup after configured hours
                        if age.as_secs() > config.worktree.cleanup_after_hours * 3600 {
                            tracing::info!("Cleaning up old worktree: {}", path);

                            let _ = Command::new("git")
                                .args(["worktree", "remove", path, "--force"])
                                .current_dir("/home/curious/S3M2P")
                                .output();
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
