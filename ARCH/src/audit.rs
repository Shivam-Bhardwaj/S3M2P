//! Audit module for tracking file validation and git metadata
//!
//! This module provides structures and functions to validate files against
//! the project structure and track git commit information.

use serde::{Deserialize, Serialize};

/// Git metadata for a file or crate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitMetadata {
    /// Last commit hash
    pub last_commit_hash: String,
    /// Last commit author
    pub author: String,
    /// Last commit date (ISO 8601)
    pub date: String,
    /// Commit message
    pub message: String,
}

/// Validation status of a crate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Valid - all files present and match expected structure
    Valid,
    /// Missing files - some expected files are missing
    MissingFiles(Vec<String>),
    /// Modified - files have uncommitted changes
    Modified,
    /// Unknown - status could not be determined
    Unknown,
}

/// Audit information for a single crate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrateAudit {
    /// Crate name
    pub name: String,
    /// Git metadata
    pub git: Option<GitMetadata>,
    /// Validation status
    pub status: ValidationStatus,
    /// File count
    pub file_count: usize,
    /// Total lines of code
    pub loc: usize,
}

impl CrateAudit {
    /// Create a new audit record
    pub fn new(name: String) -> Self {
        Self {
            name,
            git: None,
            status: ValidationStatus::Unknown,
            file_count: 0,
            loc: 0,
        }
    }
}
