//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: patterns.rs | DNA/src/security/patterns.rs
//! PURPOSE: Detection patterns for secrets and PII
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Pattern sources:
//! - GitHub Secret Scanning patterns
//! - Gitleaks rules
//! - OWASP recommendations
//! - Industry best practices

use crate::security::types::FindingType;

/// A detection pattern
#[derive(Clone, Debug)]
pub struct Pattern {
    /// Type of finding this pattern detects
    pub finding_type: FindingType,
    /// Regex pattern (as string, compiled at runtime)
    pub regex: &'static str,
    /// Description
    pub description: &'static str,
    /// Whether to check entropy
    pub check_entropy: bool,
}

/// Get all detection patterns
pub fn get_patterns() -> Vec<Pattern> {
    vec![
        // ========================================================================
        // AWS CREDENTIALS
        // ========================================================================
        Pattern {
            finding_type: FindingType::AwsAccessKey,
            regex: r"(?i)(AKIA[0-9A-Z]{16})",
            description: "AWS Access Key ID",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::AwsSecretKey,
            regex: r#"(?i)aws[_-]?secret[_-]?access[_-]?key['"\\s:=]+([A-Za-z0-9/+=]{40})"#,
            description: "AWS Secret Access Key",
            check_entropy: false,
        },
        // ========================================================================
        // GITHUB
        // ========================================================================
        Pattern {
            finding_type: FindingType::GitHubToken,
            regex: r"(?i)(ghp_[a-zA-Z0-9]{36}|gho_[a-zA-Z0-9]{36}|ghu_[a-zA-Z0-9]{36}|ghs_[a-zA-Z0-9]{36}|ghr_[a-zA-Z0-9]{36})",
            description: "GitHub Personal Access Token",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::GitHubToken,
            regex: r#"(?i)github[_-]?token['"\\s:=]+([a-zA-Z0-9]{40})"#,
            description: "GitHub Token",
            check_entropy: false,
        },
        // ========================================================================
        // STRIPE
        // ========================================================================
        Pattern {
            finding_type: FindingType::StripeKey,
            regex: r"(?i)(sk_live_[0-9a-zA-Z]{24,}|pk_live_[0-9a-zA-Z]{24,}|rk_live_[0-9a-zA-Z]{24,})",
            description: "Stripe API Key",
            check_entropy: false,
        },
        // ========================================================================
        // SLACK
        // ========================================================================
        Pattern {
            finding_type: FindingType::SlackToken,
            regex: r"(?i)(xox[pboa]-[0-9]{12}-[0-9]{12}-[0-9]{12}-[a-z0-9]{32})",
            description: "Slack Token",
            check_entropy: false,
        },
        // ========================================================================
        // GENERIC API KEYS
        // ========================================================================
        Pattern {
            finding_type: FindingType::GenericApiKey,
            regex: r#"(?i)api[_-]?key['"\\s:=]+([a-zA-Z0-9_\-]{20,})"#,
            description: "Generic API Key",
            check_entropy: true,
        },
        Pattern {
            finding_type: FindingType::GenericApiKey,
            regex: r#"(?i)api[_-]?secret['"\\s:=]+([a-zA-Z0-9_\-]{20,})"#,
            description: "Generic API Secret",
            check_entropy: true,
        },
        Pattern {
            finding_type: FindingType::GenericApiKey,
            regex: r#"(?i)access[_-]?token['"\\s:=]+([a-zA-Z0-9_\-]{20,})"#,
            description: "Generic Access Token",
            check_entropy: true,
        },
        // ========================================================================
        // PASSWORDS
        // ========================================================================
        Pattern {
            finding_type: FindingType::Password,
            regex: r#"(?i)password['"\\s:=]+['"]([^'"]{8,})['"]"#,
            description: "Password in code",
            check_entropy: true,
        },
        Pattern {
            finding_type: FindingType::Password,
            regex: r#"(?i)passwd['"\\s:=]+['"]([^'"]{8,})['"]"#,
            description: "Password in code",
            check_entropy: true,
        },
        // ========================================================================
        // JWT TOKENS
        // ========================================================================
        Pattern {
            finding_type: FindingType::JwtToken,
            regex: r"eyJ[A-Za-z0-9-_]+\.eyJ[A-Za-z0-9-_]+\.[A-Za-z0-9-_.+/=]+",
            description: "JWT Token",
            check_entropy: false,
        },
        // ========================================================================
        // PRIVATE KEYS
        // ========================================================================
        Pattern {
            finding_type: FindingType::RsaPrivateKey,
            regex: r"-----BEGIN RSA PRIVATE KEY-----",
            description: "RSA Private Key",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::RsaPrivateKey,
            regex: r"-----BEGIN PRIVATE KEY-----",
            description: "Private Key",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::SshPrivateKey,
            regex: r"-----BEGIN OPENSSH PRIVATE KEY-----",
            description: "OpenSSH Private Key",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::PgpPrivateKey,
            regex: r"-----BEGIN PGP PRIVATE KEY BLOCK-----",
            description: "PGP Private Key",
            check_entropy: false,
        },
        // ========================================================================
        // DATABASE CONNECTION STRINGS
        // ========================================================================
        Pattern {
            finding_type: FindingType::MongoDbUrl,
            regex: r"mongodb(\+srv)?://[^\s]+",
            description: "MongoDB Connection String",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::PostgresUrl,
            regex: r"postgres(ql)?://[^\s]+",
            description: "PostgreSQL Connection String",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::DatabaseUrl,
            regex: r#"(?i)database[_-]?url['"\\s:=]+['"]([^'"]+)['"]"#,
            description: "Database URL",
            check_entropy: false,
        },
        Pattern {
            finding_type: FindingType::MySqlPassword,
            regex: r"mysql://[^:]+:([^@]+)@",
            description: "MySQL Connection String with Password",
            check_entropy: false,
        },
        // ========================================================================
        // PII - EMAIL
        // ========================================================================
        Pattern {
            finding_type: FindingType::Email,
            regex: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
            description: "Email Address",
            check_entropy: false,
        },
        // ========================================================================
        // PII - PHONE NUMBERS
        // ========================================================================
        Pattern {
            finding_type: FindingType::PhoneNumber,
            regex: r"\b(?:\+?1[-.]?)?\(?([0-9]{3})\)?[-.]?([0-9]{3})[-.]?([0-9]{4})\b",
            description: "US Phone Number",
            check_entropy: false,
        },
        // ========================================================================
        // PII - CREDIT CARDS (Luhn validated)
        // ========================================================================
        Pattern {
            finding_type: FindingType::CreditCard,
            regex: r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12})\b",
            description: "Credit Card Number",
            check_entropy: false,
        },
        // ========================================================================
        // PII - SOCIAL SECURITY NUMBER
        // ========================================================================
        Pattern {
            finding_type: FindingType::SocialSecurityNumber,
            regex: r"\b\d{3}-\d{2}-\d{4}\b",
            description: "Social Security Number",
            check_entropy: false,
        },
        // ========================================================================
        // PII - IP ADDRESSES
        // ========================================================================
        Pattern {
            finding_type: FindingType::IpAddress,
            regex: r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b",
            description: "IPv4 Address",
            check_entropy: false,
        },
    ]
}

/// Calculate Shannon entropy of a string
/// Higher entropy suggests random/secret data
pub fn calculate_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    let mut char_counts = std::collections::HashMap::new();
    for c in s.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    let len = s.len() as f64;
    let mut entropy = 0.0;

    for count in char_counts.values() {
        let probability = *count as f64 / len;
        entropy -= probability * probability.log2();
    }

    entropy
}

/// Check if a string has high entropy (likely a secret)
pub fn is_high_entropy(s: &str, threshold: f64) -> bool {
    if s.len() < 16 {
        return false; // Too short to be meaningful
    }
    calculate_entropy(s) > threshold
}

/// Check if a string looks like a variable name (not a secret)
pub fn is_variable_name(s: &str) -> bool {
    if s.is_empty() || s.len() > 50 {
        return false;
    }

    // Check for common variable patterns
    let all_alphanum_underscore = s.chars().all(|c| c.is_alphanumeric() || c == '_');
    let has_lowercase = s.chars().any(|c| c.is_lowercase());
    let starts_with_letter_or_underscore = s
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_');

    // Secrets typically have high ratio of mixed case or lots of numbers
    let uppercase_count = s.chars().filter(|c| c.is_uppercase()).count();
    let digit_count = s.chars().filter(|c| c.is_ascii_digit()).count();
    let total_len = s.len();

    // If more than 40% uppercase or more than 40% digits, likely not a variable name
    let uppercase_ratio = uppercase_count as f64 / total_len as f64;
    let digit_ratio = digit_count as f64 / total_len as f64;

    all_alphanum_underscore
        && has_lowercase
        && starts_with_letter_or_underscore
        && uppercase_ratio < 0.4
        && digit_ratio < 0.4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_high() {
        let secret = "3K7qDj9mPz2nF8bY5cT1xW6vL0hR4uQ9";
        assert!(calculate_entropy(secret) > 4.0);
    }

    #[test]
    fn test_entropy_low() {
        let variable = "my_variable_name";
        assert!(calculate_entropy(variable) < 4.0);
    }

    #[test]
    fn test_is_variable_name() {
        assert!(is_variable_name("api_key"));
        assert!(is_variable_name("myPassword"));
        assert!(!is_variable_name("3K7qDj9mPz2nF8bY5cT1xW6vL0hR4uQ9"));
    }

    #[test]
    fn test_pattern_count() {
        let patterns = get_patterns();
        assert!(patterns.len() > 20, "Should have multiple patterns");
    }
}
