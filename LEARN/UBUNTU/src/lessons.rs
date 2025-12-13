//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lessons.rs | UBUNTU/src/lessons.rs
//! PURPOSE: Ubuntu lesson definitions and curriculum structure
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → UBUNTU
//! ═══════════════════════════════════════════════════════════════════════════════

/// A single Ubuntu/Linux lesson
pub struct Lesson {
    pub id: usize,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub intuition: &'static str,
    pub key_concepts: &'static [&'static str],
}

/// All Ubuntu lessons
pub static LESSONS: &[Lesson] = &[
    Lesson {
        id: 0,
        title: "File Permissions",
        subtitle: "Unix rwx Model",
        icon: "🔐",
        description: "Understand Unix file permissions - read, write, execute for owner, group, and others. Learn to use chmod and chown commands.",
        intuition: "Every file has three permission sets: what the owner can do, what group members can do, and what everyone else can do. Each set has read (r), write (w), and execute (x) flags represented as octal numbers.",
        key_concepts: &["rwx", "Owner/Group/Other", "chmod", "chown", "Octal Notation"],
    },
    Lesson {
        id: 1,
        title: "Directory Navigation",
        subtitle: "pwd, cd, ls",
        icon: "📁",
        description: "Navigate the Linux filesystem hierarchy. Learn absolute and relative paths, special directories like ~ and ..",
        intuition: "The filesystem is a tree starting at root (/). Each directory can contain files and other directories. You can move around using paths that start from root (absolute) or from where you are (relative).",
        key_concepts: &["Root (/)", "Home (~)", "Relative Path", "Absolute Path", ".. and ."],
    },
    Lesson {
        id: 2,
        title: "User Management",
        subtitle: "Users, Groups, su",
        icon: "👥",
        description: "Learn about Linux users and groups. Understand how user identity affects file access and system permissions.",
        intuition: "Linux is a multi-user system. Each user has a unique identity (UID) and belongs to one or more groups (GIDs). The 'root' user (UID 0) has unlimited power over the system.",
        key_concepts: &["UID/GID", "root", "su/sudo", "/etc/passwd", "Groups"],
    },
    Lesson {
        id: 3,
        title: "File Operations",
        subtitle: "touch, mkdir, cat",
        icon: "📝",
        description: "Create, view, and manage files and directories. Learn the basic file operations used daily in Linux.",
        intuition: "Files store data, directories organize files. Creating a file requires write permission in the parent directory. Reading requires read permission on the file itself.",
        key_concepts: &["touch", "mkdir", "cat", "rm", "cp/mv"],
    },
];
