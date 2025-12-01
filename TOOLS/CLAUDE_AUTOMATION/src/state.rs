use crate::github::Comment;
use anyhow::Result;
use rusqlite::{params, Connection};

pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS automations (
                id INTEGER PRIMARY KEY,
                issue_number INTEGER NOT NULL UNIQUE,
                status TEXT NOT NULL,
                worktree_path TEXT,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                cost_usd REAL DEFAULT 0.0,
                tokens_used INTEGER DEFAULT 0,
                has_plan INTEGER DEFAULT 0,
                last_agent_comment_at TEXT,
                waiting_for_user INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversation_history (
                id INTEGER PRIMARY KEY,
                issue_number INTEGER NOT NULL,
                comment_id INTEGER NOT NULL,
                author TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(issue_number) REFERENCES automations(issue_number)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issue_number ON conversation_history(issue_number)",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn automation_exists(&self, issue_number: u64) -> Result<bool> {
        let exists = self
            .conn
            .query_row(
                "SELECT 1 FROM automations WHERE issue_number = ?1",
                params![issue_number as i64],
                |_| Ok(true),
            )
            .unwrap_or(false);

        Ok(exists)
    }

    pub fn has_plan(&self, issue_number: u64) -> Result<bool> {
        let count: i32 = self
            .conn
            .query_row(
                "SELECT has_plan FROM automations WHERE issue_number = ?1",
                params![issue_number as i64],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(count > 0)
    }

    pub fn set_has_plan(&self, issue_number: u64) -> Result<()> {
        self.conn.execute(
            "UPDATE automations SET has_plan = 1 WHERE issue_number = ?1",
            params![issue_number as i64],
        )?;
        Ok(())
    }

    pub fn get_active_issues(&self) -> Result<Vec<u64>> {
        let mut stmt = self.conn.prepare(
            "SELECT issue_number FROM automations WHERE status IN ('triggered', 'running')",
        )?;

        let issues = stmt
            .query_map([], |row| {
                let num: i64 = row.get(0)?;
                Ok(num as u64)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(issues)
    }

    pub fn last_comment_time(&self, issue_number: u64) -> Result<Option<String>> {
        let time: Option<String> = self
            .conn
            .query_row(
                "SELECT created_at FROM conversation_history
             WHERE issue_number = ?1
             ORDER BY created_at DESC LIMIT 1",
                params![issue_number as i64],
                |row| row.get(0),
            )
            .ok();

        Ok(time)
    }

    pub fn add_comments(&self, issue_number: u64, comments: &[Comment]) -> Result<()> {
        for comment in comments {
            self.conn.execute(
                "INSERT OR IGNORE INTO conversation_history
                 (issue_number, comment_id, author, body, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    issue_number as i64,
                    comment.id as i64,
                    comment.user,
                    comment.body,
                    comment.created_at,
                ],
            )?;
        }
        Ok(())
    }

    pub fn create_automation(&self, issue_number: u64, worktree_path: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT OR REPLACE INTO automations
             (issue_number, status, worktree_path, started_at)
             VALUES (?1, 'triggered', ?2, ?3)",
            params![issue_number as i64, worktree_path, now],
        )?;

        Ok(())
    }

    pub fn update_status(&self, issue_number: u64, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE automations SET status = ?1 WHERE issue_number = ?2",
            params![status, issue_number as i64],
        )?;
        Ok(())
    }

    pub fn complete_automation(
        &self,
        issue_number: u64,
        cost_usd: f64,
        tokens_used: usize,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "UPDATE automations
             SET status = 'completed', completed_at = ?1, cost_usd = ?2, tokens_used = ?3
             WHERE issue_number = ?4",
            params![now, cost_usd, tokens_used as i64, issue_number as i64],
        )?;

        Ok(())
    }
}
