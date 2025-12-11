//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: scanner.rs | DNA/src/security/scanner.rs
//! PURPOSE: Security scanner engine
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::security::{
    patterns::{calculate_entropy, get_patterns, is_variable_name},
    types::{Finding, FindingType, ScanConfig, ScanResult},
};
use std::path::Path;

/// Security scanner
pub struct Scanner {
    patterns: Vec<CompiledPattern>,
    config: ScanConfig,
}

struct CompiledPattern {
    finding_type: FindingType,
    regex: regex::Regex,
    check_entropy: bool,
}

impl Scanner {
    /// Create a new scanner with default configuration
    pub fn new() -> Self {
        Self::with_config(ScanConfig::default())
    }

    /// Create a scanner with custom configuration
    pub fn with_config(config: ScanConfig) -> Self {
        let patterns = get_patterns();
        let mut compiled = Vec::new();

        for pattern in patterns {
            // Skip patterns based on config
            if !config.detect_secrets
                && matches!(
                    pattern.finding_type.category(),
                    crate::security::types::Category::Secret
                        | crate::security::types::Category::CryptoKey
                        | crate::security::types::Category::CloudCredential
                        | crate::security::types::Category::DatabaseCredential
                )
            {
                continue;
            }
            if !config.detect_pii
                && pattern.finding_type.category() == crate::security::types::Category::PII
            {
                continue;
            }

            if let Ok(regex) = regex::Regex::new(pattern.regex) {
                compiled.push(CompiledPattern {
                    finding_type: pattern.finding_type,
                    regex,
                    check_entropy: pattern.check_entropy,
                });
            }
        }

        Self {
            patterns: compiled,
            config,
        }
    }

    /// Scan a text string
    pub fn scan_text(&self, text: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        for (line_num, line) in text.lines().enumerate() {
            for pattern in &self.patterns {
                for mat in pattern.regex.find_iter(line) {
                    let matched_text = mat.as_str().to_string();

                    // Skip if it's a variable name (for patterns that check entropy)
                    if pattern.check_entropy && is_variable_name(&matched_text) {
                        continue;
                    }

                    // For entropy-checking patterns, validate entropy
                    if pattern.check_entropy {
                        let entropy = calculate_entropy(&matched_text);
                        if entropy < 3.5 {
                            // Low entropy, likely not a secret
                            continue;
                        }
                    }

                    // Additional validation for specific types
                    if let Some(validated) =
                        self.validate_finding(&pattern.finding_type, &matched_text)
                    {
                        if !validated {
                            continue;
                        }
                    }

                    let finding = Finding::new(
                        pattern.finding_type.clone(),
                        file_path.to_string(),
                        line_num + 1,
                        mat.start(),
                        matched_text,
                        line.to_string(),
                    );

                    // Check minimum severity
                    if finding.severity >= self.config.min_severity {
                        findings.push(finding);
                    }
                }
            }
        }

        findings
    }

    /// Scan a file
    pub fn scan_file(&self, path: &Path) -> Result<Vec<Finding>, std::io::Error> {
        let text = std::fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();
        Ok(self.scan_text(&text, &path_str))
    }

    /// Scan a directory recursively
    pub fn scan_directory(&self, dir: &Path) -> Result<ScanResult, std::io::Error> {
        use std::time::Instant;
        let start = Instant::now();

        let mut result = ScanResult::new();

        self.scan_directory_recursive(dir, &mut result)?;

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    fn scan_directory_recursive(
        &self,
        dir: &Path,
        result: &mut ScanResult,
    ) -> Result<(), std::io::Error> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Check exclude patterns
            if self.should_exclude(&path) {
                continue;
            }

            if path.is_dir() {
                self.scan_directory_recursive(&path, result)?;
            } else if path.is_file() {
                // Check file extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_string();
                    if !self.config.file_extensions.contains(&ext_str) {
                        continue;
                    }
                }

                if let Ok(text) = std::fs::read_to_string(&path) {
                    let path_str = path.to_string_lossy().to_string();
                    let line_count = text.lines().count();
                    let findings = self.scan_text(&text, &path_str);

                    result.files_scanned += 1;
                    result.lines_scanned += line_count;
                    result.findings.extend(findings);
                }
            }
        }

        Ok(())
    }

    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.exclude_paths {
            // Simple glob matching
            if self.matches_glob(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_glob(&self, path: &str, pattern: &str) -> bool {
        // Simple glob implementation (supports **)
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1].trim_start_matches('/');

                return path.contains(prefix) && (suffix.is_empty() || path.contains(suffix));
            }
        }

        // Simple wildcard matching
        path.contains(pattern.trim_matches('*'))
    }

    /// Additional validation for specific finding types
    fn validate_finding(&self, finding_type: &FindingType, text: &str) -> Option<bool> {
        match finding_type {
            FindingType::CreditCard => Some(self.validate_credit_card(text)),
            FindingType::IpAddress => Some(self.validate_ip_address(text)),
            _ => None,
        }
    }

    /// Validate credit card using Luhn algorithm
    fn validate_credit_card(&self, text: &str) -> bool {
        let digits: Vec<u32> = text.chars().filter_map(|c| c.to_digit(10)).collect();

        if digits.len() < 13 || digits.len() > 19 {
            return false;
        }

        let mut sum = 0;
        let mut double = false;

        for &digit in digits.iter().rev() {
            let mut d = digit;
            if double {
                d *= 2;
                if d > 9 {
                    d -= 9;
                }
            }
            sum += d;
            double = !double;
        }

        sum % 10 == 0
    }

    /// Validate IPv4 address
    fn validate_ip_address(&self, text: &str) -> bool {
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() != 4 {
            return false;
        }

        for part in parts {
            if let Ok(num) = part.parse::<u32>() {
                if num > 255 {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Exclude common false positives like version numbers
        // If it starts with 0. or ends with .0.0, likely not a real IP
        if text.starts_with("0.") || text.ends_with(".0.0") {
            return false;
        }

        true
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a simple regex-based scanner
pub fn quick_scan(text: &str) -> bool {
    let scanner = Scanner::new();
    let findings = scanner.scan_text(text, "inline");
    !findings.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_key_detection() {
        let scanner = Scanner::new();
        let text = "const key = 'AKIAIOSFODNN7EXAMPLE';";
        let findings = scanner.scan_text(text, "test.js");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding_type, FindingType::AwsAccessKey);
    }

    #[test]
    fn test_email_detection() {
        let scanner = Scanner::new();
        let text = "Contact: user@example.com";
        let findings = scanner.scan_text(text, "test.txt");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding_type, FindingType::Email);
    }

    #[test]
    fn test_private_key_detection() {
        let scanner = Scanner::new();
        let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQ...";
        let findings = scanner.scan_text(text, "key.pem");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding_type, FindingType::RsaPrivateKey);
    }

    #[test]
    fn test_github_token_detection() {
        let scanner = Scanner::new();
        let text = "token = 'ghp_abcdefghijklmnopqrstuvwxyz1234567890'"; // 36 chars after ghp_
        let findings = scanner.scan_text(text, "config.py");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding_type, FindingType::GitHubToken);
    }

    #[test]
    fn test_jwt_detection() {
        let scanner = Scanner::new();
        let text = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let findings = scanner.scan_text(text, "test.txt");

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, FindingType::JwtToken);
    }

    #[test]
    fn test_luhn_validation() {
        let scanner = Scanner::new();

        // Valid credit card (test number)
        assert!(scanner.validate_credit_card("4532015112830366"));

        // Invalid credit card
        assert!(!scanner.validate_credit_card("4532015112830367"));
    }

    #[test]
    fn test_ip_validation() {
        let scanner = Scanner::new();

        assert!(scanner.validate_ip_address("192.168.1.1"));
        assert!(!scanner.validate_ip_address("256.1.1.1"));
        assert!(!scanner.validate_ip_address("192.168.1"));
    }

    #[test]
    fn test_config_filtering() {
        let mut config = ScanConfig::default();
        config.detect_pii = false;

        let scanner = Scanner::with_config(config);
        let text = "Email: user@example.com and AWS_KEY='AKIAIOSFODNN7EXAMPLE'";
        let findings = scanner.scan_text(text, "test.txt");

        // Should only detect AWS key, not email
        assert!(findings
            .iter()
            .all(|f| f.finding_type != FindingType::Email));
        assert!(findings
            .iter()
            .any(|f| f.finding_type == FindingType::AwsAccessKey));
    }

    #[test]
    fn test_quick_scan() {
        // Test with AWS key (which we know works from other tests)
        assert!(quick_scan("const key = 'AKIAIOSFODNN7EXAMPLE';"));
        assert!(!quick_scan("let x = 42;"));
    }
}
