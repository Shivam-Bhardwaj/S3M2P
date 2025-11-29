use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Comment {
    pub id: String,
    pub lesson_id: String,
    pub author: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CommentStore {
    file_path: String,
    comments: Arc<Mutex<Vec<Comment>>>,
}

impl CommentStore {
    pub fn new(file_path: &str) -> Self {
        let comments = if let Ok(mut file) = File::open(file_path) {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        Self {
            file_path: file_path.to_string(),
            comments: Arc::new(Mutex::new(comments)),
        }
    }

    pub fn add_comment(&self, lesson_id: String, author: String, content: String) -> Comment {
        let mut comments = self.comments.lock().unwrap();
        
        let new_comment = Comment {
            id: Uuid::new_v4().to_string(),
            lesson_id,
            author,
            content,
            timestamp: Utc::now(),
        };

        comments.push(new_comment.clone());
        self.save(&comments);
        new_comment
    }

    pub fn get_comments(&self, lesson_id: &str) -> Vec<Comment> {
        let comments = self.comments.lock().unwrap();
        comments.iter()
            .filter(|c| c.lesson_id == lesson_id)
            .cloned()
            .collect()
    }

    fn save(&self, comments: &[Comment]) {
        if let Ok(json) = serde_json::to_string_pretty(comments) {
            let _ = fs::write(&self.file_path, json);
        }
    }
}
