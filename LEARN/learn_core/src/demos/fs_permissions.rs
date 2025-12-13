//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: fs_permissions.rs | LEARN/learn_core/src/demos/fs_permissions.rs
//! PURPOSE: In-memory filesystem with Unix permissions simulation
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::{Demo, ParamMeta};
use std::collections::HashMap;

/// A filesystem node (file or directory)
#[derive(Clone, Debug)]
pub struct INode {
    pub name: String,
    pub is_dir: bool,
    pub owner: String,
    pub group: String,
    pub permissions: u16, // Unix style: rwxrwxrwx
    pub content: String,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
}

impl INode {
    fn new_file(name: &str, owner: &str, group: &str, perms: u16) -> Self {
        Self {
            name: name.to_string(),
            is_dir: false,
            owner: owner.to_string(),
            group: group.to_string(),
            permissions: perms,
            content: String::new(),
            children: Vec::new(),
            parent: None,
        }
    }

    fn new_dir(name: &str, owner: &str, group: &str, perms: u16) -> Self {
        Self {
            name: name.to_string(),
            is_dir: true,
            owner: owner.to_string(),
            group: group.to_string(),
            permissions: perms,
            content: String::new(),
            children: Vec::new(),
            parent: None,
        }
    }
}

/// Simulated terminal command result
#[derive(Clone, Debug)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
}

impl CommandResult {
    fn ok(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
        }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            output: msg.into(),
        }
    }
}

/// Filesystem permissions demo with simulated terminal
///
/// Teaches Unix permissions through an interactive shell simulation.
#[derive(Clone)]
pub struct FsPermissionsDemo {
    // Filesystem
    pub inodes: Vec<INode>,
    root_idx: usize,
    pub cwd: usize,

    // User context
    pub current_user: String,
    pub current_group: String,
    pub users: HashMap<String, Vec<String>>, // user -> groups

    // Terminal state
    pub command_history: Vec<String>,
    pub output_history: Vec<(String, bool)>, // (line, is_error)
    pub prompt: String,
    max_history: usize,
}

impl Default for FsPermissionsDemo {
    fn default() -> Self {
        Self {
            inodes: Vec::new(),
            root_idx: 0,
            cwd: 0,
            current_user: "user".to_string(),
            current_group: "user".to_string(),
            users: HashMap::new(),
            command_history: Vec::new(),
            output_history: Vec::new(),
            prompt: String::new(),
            max_history: 50,
        }
    }
}

impl FsPermissionsDemo {
    /// Initialize the filesystem with a sample structure
    fn init_fs(&mut self) {
        self.inodes.clear();

        // Create root directory (idx 0)
        let mut root = INode::new_dir("/", "root", "root", 0o755);
        root.parent = None;
        self.inodes.push(root);
        self.root_idx = 0;

        // Create /home (idx 1)
        let home = self.create_dir(0, "home", "root", "root", 0o755);

        // Create /home/user (idx 2)
        let user_home = self.create_dir(home, "user", "user", "user", 0o755);

        // Create files in /home/user
        self.create_file(user_home, "readme.txt", "user", "user", 0o644);
        self.create_file(user_home, "secret.txt", "user", "user", 0o600);
        self.create_file(user_home, "script.sh", "user", "user", 0o755);

        // Create /etc (idx 6)
        let etc = self.create_dir(0, "etc", "root", "root", 0o755);

        // Create /etc/passwd (idx 7)
        let passwd = self.create_file(etc, "passwd", "root", "root", 0o644);
        self.inodes[passwd].content = "root:x:0:0:root:/root:/bin/bash\nuser:x:1000:1000:User:/home/user:/bin/bash".to_string();

        // Create /etc/shadow (idx 8)
        let shadow = self.create_file(etc, "shadow", "root", "shadow", 0o640);
        self.inodes[shadow].content = "[encrypted passwords]".to_string();

        // Create /tmp (idx 9)
        let tmp = self.create_dir(0, "tmp", "root", "root", 0o1777);

        // Create shared file in /tmp
        self.create_file(tmp, "shared.txt", "user", "user", 0o666);

        // Set cwd to user home
        self.cwd = user_home;

        // Initialize users
        self.users.insert("root".to_string(), vec!["root".to_string()]);
        self.users
            .insert("user".to_string(), vec!["user".to_string()]);

        self.update_prompt();
    }

    fn create_dir(
        &mut self,
        parent_idx: usize,
        name: &str,
        owner: &str,
        group: &str,
        perms: u16,
    ) -> usize {
        let idx = self.inodes.len();
        let mut dir = INode::new_dir(name, owner, group, perms);
        dir.parent = Some(parent_idx);
        self.inodes.push(dir);
        self.inodes[parent_idx].children.push(idx);
        idx
    }

    fn create_file(
        &mut self,
        parent_idx: usize,
        name: &str,
        owner: &str,
        group: &str,
        perms: u16,
    ) -> usize {
        let idx = self.inodes.len();
        let mut file = INode::new_file(name, owner, group, perms);
        file.parent = Some(parent_idx);
        self.inodes.push(file);
        self.inodes[parent_idx].children.push(idx);
        idx
    }

    fn update_prompt(&mut self) {
        let path = self.get_path(self.cwd);
        self.prompt = format!("{}@ubuntu:{}$ ", self.current_user, path);
    }

    /// Get full path to an inode
    pub fn get_path(&self, idx: usize) -> String {
        let mut parts = Vec::new();
        let mut current = idx;

        loop {
            let node = &self.inodes[current];
            if node.name != "/" {
                parts.push(node.name.clone());
            }
            match node.parent {
                Some(p) => current = p,
                None => break,
            }
        }

        if parts.is_empty() {
            "/".to_string()
        } else {
            parts.reverse();
            format!("/{}", parts.join("/"))
        }
    }

    /// Check if current user has permission
    fn check_permission(&self, inode: &INode, perm: u8) -> bool {
        // Root can do anything
        if self.current_user == "root" {
            return true;
        }

        let bits = if self.current_user == inode.owner {
            (inode.permissions >> 6) & 7
        } else if self
            .users
            .get(&self.current_user)
            .map(|g| g.contains(&inode.group))
            .unwrap_or(false)
        {
            (inode.permissions >> 3) & 7
        } else {
            inode.permissions & 7
        };

        (bits as u8 & perm) == perm
    }

    /// Format permissions as string (e.g., "-rwxr-xr-x")
    pub fn format_permissions(&self, perms: u16, is_dir: bool) -> String {
        let d = if is_dir { 'd' } else { '-' };
        let mut s = String::with_capacity(10);
        s.push(d);

        for shift in (0..9).rev() {
            let bit = (perms >> shift) & 1;
            let c = match shift % 3 {
                2 => {
                    if bit == 1 {
                        'r'
                    } else {
                        '-'
                    }
                }
                1 => {
                    if bit == 1 {
                        'w'
                    } else {
                        '-'
                    }
                }
                0 => {
                    if bit == 1 {
                        'x'
                    } else {
                        '-'
                    }
                }
                _ => '-',
            };
            s.push(c);
        }

        s
    }

    /// Resolve a path to an inode index
    fn resolve_path(&self, path: &str) -> Option<usize> {
        let start = if path.starts_with('/') {
            self.root_idx
        } else {
            self.cwd
        };

        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = start;

        for part in parts {
            if part == "." {
                continue;
            } else if part == ".." {
                if let Some(p) = self.inodes[current].parent {
                    current = p;
                }
            } else {
                let node = &self.inodes[current];
                let mut found = false;
                for &child_idx in &node.children {
                    if self.inodes[child_idx].name == part {
                        current = child_idx;
                        found = true;
                        break;
                    }
                }
                if !found {
                    return None;
                }
            }
        }

        Some(current)
    }

    /// Execute a command and return the result
    pub fn execute(&mut self, cmd: &str) -> CommandResult {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            return CommandResult::ok("");
        }

        // Add to history
        self.command_history.push(cmd.to_string());
        if self.command_history.len() > self.max_history {
            self.command_history.remove(0);
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let result = match parts[0] {
            "ls" => self.cmd_ls(&parts[1..]),
            "cd" => self.cmd_cd(&parts[1..]),
            "pwd" => CommandResult::ok(self.get_path(self.cwd)),
            "cat" => self.cmd_cat(&parts[1..]),
            "chmod" => self.cmd_chmod(&parts[1..]),
            "chown" => self.cmd_chown(&parts[1..]),
            "mkdir" => self.cmd_mkdir(&parts[1..]),
            "touch" => self.cmd_touch(&parts[1..]),
            "whoami" => CommandResult::ok(&self.current_user),
            "id" => self.cmd_id(),
            "su" => self.cmd_su(&parts[1..]),
            "help" => self.cmd_help(),
            _ => CommandResult::err(format!("{}: command not found", parts[0])),
        };

        // Add output to history
        if !result.output.is_empty() {
            for line in result.output.lines() {
                self.output_history.push((line.to_string(), !result.success));
            }
            while self.output_history.len() > self.max_history {
                self.output_history.remove(0);
            }
        }

        result
    }

    fn cmd_ls(&self, args: &[&str]) -> CommandResult {
        let show_long = args.contains(&"-l");
        let target = args
            .iter()
            .find(|a| !a.starts_with('-'))
            .copied()
            .unwrap_or(".");

        let idx = match self.resolve_path(target) {
            Some(i) => i,
            None => return CommandResult::err(format!("ls: cannot access '{}': No such file or directory", target)),
        };

        let node = &self.inodes[idx];

        // Check read permission
        if !self.check_permission(node, 4) {
            return CommandResult::err(format!("ls: cannot open directory '{}': Permission denied", target));
        }

        if !node.is_dir {
            // Single file
            if show_long {
                return CommandResult::ok(format!(
                    "{} {} {} {}",
                    self.format_permissions(node.permissions, node.is_dir),
                    node.owner,
                    node.group,
                    node.name
                ));
            } else {
                return CommandResult::ok(&node.name);
            }
        }

        let mut lines = Vec::new();
        for &child_idx in &node.children {
            let child = &self.inodes[child_idx];
            if show_long {
                lines.push(format!(
                    "{} {} {} {}",
                    self.format_permissions(child.permissions, child.is_dir),
                    child.owner,
                    child.group,
                    child.name
                ));
            } else {
                lines.push(child.name.clone());
            }
        }

        CommandResult::ok(lines.join("\n"))
    }

    fn cmd_cd(&mut self, args: &[&str]) -> CommandResult {
        let path = args.first().copied().unwrap_or("~");
        let path = if path == "~" { "/home/user" } else { path };

        let idx = match self.resolve_path(path) {
            Some(i) => i,
            None => return CommandResult::err(format!("cd: {}: No such file or directory", path)),
        };

        let node = &self.inodes[idx];

        if !node.is_dir {
            return CommandResult::err(format!("cd: {}: Not a directory", path));
        }

        // Check execute permission (needed to enter directory)
        if !self.check_permission(node, 1) {
            return CommandResult::err(format!("cd: {}: Permission denied", path));
        }

        self.cwd = idx;
        self.update_prompt();
        CommandResult::ok("")
    }

    fn cmd_cat(&self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::err("cat: missing operand");
        }

        let path = args[0];
        let idx = match self.resolve_path(path) {
            Some(i) => i,
            None => return CommandResult::err(format!("cat: {}: No such file or directory", path)),
        };

        let node = &self.inodes[idx];

        if node.is_dir {
            return CommandResult::err(format!("cat: {}: Is a directory", path));
        }

        // Check read permission
        if !self.check_permission(node, 4) {
            return CommandResult::err(format!("cat: {}: Permission denied", path));
        }

        CommandResult::ok(&node.content)
    }

    fn cmd_chmod(&mut self, args: &[&str]) -> CommandResult {
        if args.len() < 2 {
            return CommandResult::err("chmod: missing operand");
        }

        let mode = args[0];
        let path = args[1];

        let idx = match self.resolve_path(path) {
            Some(i) => i,
            None => return CommandResult::err(format!("chmod: cannot access '{}': No such file or directory", path)),
        };

        // Only owner or root can chmod
        if self.current_user != "root" && self.current_user != self.inodes[idx].owner {
            return CommandResult::err(format!("chmod: changing permissions of '{}': Operation not permitted", path));
        }

        // Parse octal mode
        let perms = match u16::from_str_radix(mode, 8) {
            Ok(p) if p <= 0o7777 => p,
            _ => return CommandResult::err(format!("chmod: invalid mode: '{}'", mode)),
        };

        self.inodes[idx].permissions = perms;
        CommandResult::ok("")
    }

    fn cmd_chown(&mut self, args: &[&str]) -> CommandResult {
        if args.len() < 2 {
            return CommandResult::err("chown: missing operand");
        }

        // Only root can chown
        if self.current_user != "root" {
            return CommandResult::err("chown: Operation not permitted");
        }

        let owner_group = args[0];
        let path = args[1];

        let idx = match self.resolve_path(path) {
            Some(i) => i,
            None => return CommandResult::err(format!("chown: cannot access '{}': No such file or directory", path)),
        };

        let parts: Vec<&str> = owner_group.split(':').collect();
        self.inodes[idx].owner = parts[0].to_string();
        if parts.len() > 1 {
            self.inodes[idx].group = parts[1].to_string();
        }

        CommandResult::ok("")
    }

    fn cmd_mkdir(&mut self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::err("mkdir: missing operand");
        }

        let path = args[0];

        // Get parent directory
        let parent_path = if path.contains('/') {
            path.rsplit_once('/').map(|(p, _)| p).unwrap_or(".")
        } else {
            "."
        };
        let name = path.rsplit('/').next().unwrap();

        let parent_idx = match self.resolve_path(parent_path) {
            Some(i) => i,
            None => {
                return CommandResult::err(format!(
                    "mkdir: cannot create directory '{}': No such file or directory",
                    path
                ))
            }
        };

        // Check write permission on parent
        if !self.check_permission(&self.inodes[parent_idx], 2) {
            return CommandResult::err(format!(
                "mkdir: cannot create directory '{}': Permission denied",
                path
            ));
        }

        // Create directory
        self.create_dir(
            parent_idx,
            name,
            &self.current_user.clone(),
            &self.current_group.clone(),
            0o755,
        );
        CommandResult::ok("")
    }

    fn cmd_touch(&mut self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::err("touch: missing file operand");
        }

        let path = args[0];

        // Check if file exists
        if self.resolve_path(path).is_some() {
            return CommandResult::ok(""); // touch existing file does nothing
        }

        // Get parent directory
        let parent_path = if path.contains('/') {
            path.rsplit_once('/').map(|(p, _)| p).unwrap_or(".")
        } else {
            "."
        };
        let name = path.rsplit('/').next().unwrap();

        let parent_idx = match self.resolve_path(parent_path) {
            Some(i) => i,
            None => {
                return CommandResult::err(format!(
                    "touch: cannot touch '{}': No such file or directory",
                    path
                ))
            }
        };

        // Check write permission on parent
        if !self.check_permission(&self.inodes[parent_idx], 2) {
            return CommandResult::err(format!("touch: cannot touch '{}': Permission denied", path));
        }

        // Create file
        self.create_file(
            parent_idx,
            name,
            &self.current_user.clone(),
            &self.current_group.clone(),
            0o644,
        );
        CommandResult::ok("")
    }

    fn cmd_id(&self) -> CommandResult {
        let groups = self
            .users
            .get(&self.current_user)
            .map(|g| g.join(","))
            .unwrap_or_default();
        CommandResult::ok(format!(
            "uid=1000({}) gid=1000({}) groups={}",
            self.current_user, self.current_group, groups
        ))
    }

    fn cmd_su(&mut self, args: &[&str]) -> CommandResult {
        let user = args.first().copied().unwrap_or("root");

        if !self.users.contains_key(user) {
            return CommandResult::err(format!("su: user {} does not exist", user));
        }

        self.current_user = user.to_string();
        self.current_group = user.to_string();
        self.update_prompt();
        CommandResult::ok("")
    }

    fn cmd_help(&self) -> CommandResult {
        CommandResult::ok(
            "Available commands:\n\
             ls [-l] [path]     - List directory contents\n\
             cd [path]          - Change directory\n\
             pwd                - Print working directory\n\
             cat <file>         - Display file contents\n\
             chmod <mode> <file> - Change file permissions\n\
             chown <owner> <file> - Change file owner (root only)\n\
             mkdir <dir>        - Create directory\n\
             touch <file>       - Create empty file\n\
             whoami             - Print current user\n\
             id                 - Print user identity\n\
             su [user]          - Switch user (default: root)",
        )
    }
}

impl Demo for FsPermissionsDemo {
    fn reset(&mut self, _seed: u64) {
        self.init_fs();
        self.command_history.clear();
        self.output_history.clear();
        self.current_user = "user".to_string();
        self.current_group = "user".to_string();
        self.update_prompt();
    }

    fn step(&mut self, _dt: f32) {
        // No continuous simulation - this demo is event-driven (commands)
    }

    fn set_param(&mut self, _name: &str, _value: f32) -> bool {
        // No tunable parameters for this demo
        false
    }

    fn params() -> &'static [ParamMeta] {
        &[] // No slider parameters - this is a command-driven demo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_fs() {
        let mut demo = FsPermissionsDemo::default();
        demo.reset(42);

        // Should have root directory
        assert!(!demo.inodes.is_empty());
        assert_eq!(demo.inodes[0].name, "/");
    }

    #[test]
    fn test_ls() {
        let mut demo = FsPermissionsDemo::default();
        demo.reset(42);

        let result = demo.execute("ls");
        assert!(result.success);
        assert!(result.output.contains("readme.txt"));
    }

    #[test]
    fn test_cd() {
        let mut demo = FsPermissionsDemo::default();
        demo.reset(42);

        demo.execute("cd /etc");
        let result = demo.execute("pwd");
        assert_eq!(result.output, "/etc");
    }

    #[test]
    fn test_permission_denied() {
        let mut demo = FsPermissionsDemo::default();
        demo.reset(42);

        // Try to read shadow file as user
        let result = demo.execute("cat /etc/shadow");
        assert!(!result.success);
        assert!(result.output.contains("Permission denied"));

        // Switch to root and try again
        demo.execute("su root");
        let result = demo.execute("cat /etc/shadow");
        assert!(result.success);
    }

    #[test]
    fn test_chmod() {
        let mut demo = FsPermissionsDemo::default();
        demo.reset(42);

        demo.execute("chmod 777 readme.txt");
        let result = demo.execute("ls -l");
        assert!(result.output.contains("-rwxrwxrwx"));
    }

    #[test]
    fn test_format_permissions() {
        let demo = FsPermissionsDemo::default();

        assert_eq!(demo.format_permissions(0o755, false), "-rwxr-xr-x");
        assert_eq!(demo.format_permissions(0o644, false), "-rw-r--r--");
        assert_eq!(demo.format_permissions(0o755, true), "drwxr-xr-x");
    }
}
