use anyhow::Result;
use crate::github::Comment;
use crate::state::Database;

pub enum Agent {
    Planner,  // Opus - for complex reasoning and architecture
    Executor, // Sonnet - for fast iteration
}

/// Decide which agent to spawn based on comment content
pub fn decide(comments: &[Comment], db: &Database) -> Result<Agent> {
    if comments.is_empty() {
        return Ok(Agent::Executor);
    }

    let last_comment = &comments[comments.len() - 1];
    let body = last_comment.body.to_lowercase();

    // Keywords that trigger re-planning (Opus)
    let replanning_keywords = [
        "different approach",
        "rethink",
        "redesign",
        "change architecture",
        "breaking change",
        "major refactor",
        "let's try",
        "instead of",
        "better way",
    ];

    // Check if we need architectural thinking
    if replanning_keywords.iter().any(|kw| body.contains(kw)) {
        return Ok(Agent::Planner);
    }

    // Check if we have a plan yet
    if !db.has_plan(last_comment.issue_number)? {
        return Ok(Agent::Planner); // Need initial plan
    }

    // Otherwise, use Executor for fast iteration
    Ok(Agent::Executor)
}
