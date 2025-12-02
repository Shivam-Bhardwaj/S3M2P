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

    // Keywords that explicitly request implementation (Executor)
    let executor_keywords = [
        "implement",
        "go ahead",
        "execute",
        "looks good",
        "lgtm",
        "ship it",
        "do it",
        "make it",
        "build it",
        "proceed",
        "fix",
        "address",
    ];

    // Check for explicit implementation request
    if executor_keywords.iter().any(|kw| body.contains(kw)) {
        // Force executor even without plan flag
        return Ok(Agent::Executor);
    }

    // Keywords that trigger re-planning (Opus)
    let replanning_keywords = [
        "different approach",
        "rethink",
        "redesign",
        "change architecture",
        "breaking change",
        "major refactor",
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_db() -> (Database, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::open(temp_file.path().to_str().unwrap()).unwrap();
        (db, temp_file)
    }

    fn make_comment(issue_number: u64, body: &str) -> Comment {
        Comment {
            id: 1,
            issue_number,
            user: "testuser".to_string(),
            body: body.to_string(),
            created_at: "2025-11-30T00:00:00Z".to_string(),
            updated_at: "2025-11-30T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_executor_keywords() {
        let (db, _temp) = create_test_db();
        db.create_automation(1, "/tmp/test").unwrap();
        db.set_has_plan(1).unwrap();

        let test_cases = vec![
            "implement this",
            "go ahead and do it",
            "execute the plan",
            "looks good!",
            "LGTM ship it",
            "fix the bug",
            "please address this",
        ];

        for body in test_cases {
            let comments = vec![make_comment(1, body)];
            let agent = decide(&comments, &db).unwrap();
            assert!(matches!(agent, Agent::Executor), "Failed for: {}", body);
        }
    }

    #[test]
    fn test_planner_keywords() {
        let (db, _temp) = create_test_db();
        db.create_automation(1, "/tmp/test").unwrap();

        let test_cases = vec![
            "let's try a different approach",
            "we should rethink this",
            "redesign the whole thing",
            "this needs major refactor",
        ];

        for body in test_cases {
            let comments = vec![make_comment(1, body)];
            let agent = decide(&comments, &db).unwrap();
            assert!(matches!(agent, Agent::Planner), "Failed for: {}", body);
        }
    }

    #[test]
    fn test_no_plan_triggers_planner() {
        let (db, _temp) = create_test_db();
        // Don't set has_plan

        let comments = vec![make_comment(1, "just a regular comment")];
        let agent = decide(&comments, &db).unwrap();
        assert!(matches!(agent, Agent::Planner));
    }

    #[test]
    fn test_with_plan_defaults_executor() {
        let (db, _temp) = create_test_db();
        db.create_automation(1, "/tmp/test").unwrap();
        db.set_has_plan(1).unwrap();

        let comments = vec![make_comment(1, "some regular feedback")];
        let agent = decide(&comments, &db).unwrap();
        assert!(matches!(agent, Agent::Executor));
    }
}
