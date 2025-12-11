//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: types.rs | DNA/src/security/types.rs
//! PURPOSE: Security scanner type definitions
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

/// Severity level for security findings
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational - no immediate risk
    Info,
    /// Low severity - minor concern
    Low,
    /// Medium severity - should be reviewed
    Medium,
    /// High severity - needs attention
    High,
    /// Critical severity - must be fixed immediately
    Critical,
}

impl Severity {
    /// Get color code for terminal output
    pub fn color_code(&self) -> &'static str {
        match self {
            Severity::Info => "\x1b[36m",     // Cyan
            Severity::Low => "\x1b[32m",      // Green
            Severity::Medium => "\x1b[33m",   // Yellow
            Severity::High => "\x1b[31m",     // Red
            Severity::Critical => "\x1b[35m", // Magenta
        }
    }

    /// Get emoji indicator
    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Info => "ℹ️",
            Severity::Low => "⚠️",
            Severity::Medium => "⚡",
            Severity::High => "🔥",
            Severity::Critical => "💀",
        }
    }
}

/// Category of security finding
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    /// API keys, tokens, passwords
    Secret,
    /// Personally Identifiable Information
    PII,
    /// Cryptographic keys (private keys, certificates)
    CryptoKey,
    /// Database credentials and connection strings
    DatabaseCredential,
    /// Cloud provider credentials
    CloudCredential,
    /// Generic suspicious pattern
    Suspicious,
}

impl Category {
    pub fn name(&self) -> &'static str {
        match self {
            Category::Secret => "Secret",
            Category::PII => "PII",
            Category::CryptoKey => "Crypto Key",
            Category::DatabaseCredential => "Database Credential",
            Category::CloudCredential => "Cloud Credential",
            Category::Suspicious => "Suspicious",
        }
    }
}

/// Type of detected secret or PII
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingType {
    // Secrets
    AwsAccessKey,
    AwsSecretKey,
    GitHubToken,
    StripeKey,
    SlackToken,
    GenericApiKey,
    JwtToken,
    Password,

    // Crypto Keys
    RsaPrivateKey,
    SshPrivateKey,
    PgpPrivateKey,
    Certificate,

    // Database
    DatabaseUrl,
    MongoDbUrl,
    PostgresUrl,
    MySqlPassword,

    // PII
    Email,
    PhoneNumber,
    CreditCard,
    SocialSecurityNumber,
    IpAddress,

    // Generic
    HighEntropy,
    Suspicious,
}

impl FindingType {
    pub fn name(&self) -> &'static str {
        match self {
            FindingType::AwsAccessKey => "AWS Access Key",
            FindingType::AwsSecretKey => "AWS Secret Key",
            FindingType::GitHubToken => "GitHub Token",
            FindingType::StripeKey => "Stripe API Key",
            FindingType::SlackToken => "Slack Token",
            FindingType::GenericApiKey => "Generic API Key",
            FindingType::JwtToken => "JWT Token",
            FindingType::Password => "Password",
            FindingType::RsaPrivateKey => "RSA Private Key",
            FindingType::SshPrivateKey => "SSH Private Key",
            FindingType::PgpPrivateKey => "PGP Private Key",
            FindingType::Certificate => "Certificate",
            FindingType::DatabaseUrl => "Database URL",
            FindingType::MongoDbUrl => "MongoDB Connection String",
            FindingType::PostgresUrl => "PostgreSQL Connection String",
            FindingType::MySqlPassword => "MySQL Password",
            FindingType::Email => "Email Address",
            FindingType::PhoneNumber => "Phone Number",
            FindingType::CreditCard => "Credit Card Number",
            FindingType::SocialSecurityNumber => "Social Security Number",
            FindingType::IpAddress => "IP Address",
            FindingType::HighEntropy => "High Entropy String",
            FindingType::Suspicious => "Suspicious Pattern",
        }
    }

    pub fn category(&self) -> Category {
        match self {
            FindingType::AwsAccessKey | FindingType::AwsSecretKey => Category::CloudCredential,
            FindingType::GitHubToken
            | FindingType::StripeKey
            | FindingType::SlackToken
            | FindingType::GenericApiKey
            | FindingType::JwtToken
            | FindingType::Password => Category::Secret,
            FindingType::RsaPrivateKey
            | FindingType::SshPrivateKey
            | FindingType::PgpPrivateKey
            | FindingType::Certificate => Category::CryptoKey,
            FindingType::DatabaseUrl
            | FindingType::MongoDbUrl
            | FindingType::PostgresUrl
            | FindingType::MySqlPassword => Category::DatabaseCredential,
            FindingType::Email
            | FindingType::PhoneNumber
            | FindingType::CreditCard
            | FindingType::SocialSecurityNumber
            | FindingType::IpAddress => Category::PII,
            FindingType::HighEntropy | FindingType::Suspicious => Category::Suspicious,
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            // Critical: Exposed credentials
            FindingType::AwsAccessKey
            | FindingType::AwsSecretKey
            | FindingType::RsaPrivateKey
            | FindingType::SshPrivateKey
            | FindingType::DatabaseUrl => Severity::Critical,

            // High: API keys and tokens
            FindingType::GitHubToken
            | FindingType::StripeKey
            | FindingType::SlackToken
            | FindingType::GenericApiKey
            | FindingType::Password
            | FindingType::MongoDbUrl
            | FindingType::PostgresUrl => Severity::High,

            // Medium: PII
            FindingType::Email
            | FindingType::PhoneNumber
            | FindingType::CreditCard
            | FindingType::SocialSecurityNumber => Severity::Medium,

            // Low: Less sensitive
            FindingType::IpAddress | FindingType::JwtToken => Severity::Low,

            // Info: Suspicious patterns
            FindingType::HighEntropy
            | FindingType::Suspicious
            | FindingType::PgpPrivateKey
            | FindingType::Certificate
            | FindingType::MySqlPassword => Severity::Info,
        }
    }
}

/// A security finding detected in code
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    /// Type of finding
    pub finding_type: FindingType,
    /// Severity level
    pub severity: Severity,
    /// Category
    pub category: Category,
    /// File path where found
    pub file_path: String,
    /// Line number (1-indexed)
    pub line_number: usize,
    /// Column number (0-indexed)
    pub column: usize,
    /// Matched text (may be redacted)
    pub matched_text: String,
    /// Context (surrounding lines)
    pub context: String,
    /// Description/remediation advice
    pub description: String,
}

impl Finding {
    pub fn new(
        finding_type: FindingType,
        file_path: String,
        line_number: usize,
        column: usize,
        matched_text: String,
        context: String,
    ) -> Self {
        let severity = finding_type.severity();
        let category = finding_type.category();
        let description = Self::get_description(&finding_type);

        Self {
            finding_type,
            severity,
            category,
            file_path,
            line_number,
            column,
            matched_text,
            context,
            description,
        }
    }

    fn get_description(finding_type: &FindingType) -> String {
        match finding_type {
            FindingType::AwsAccessKey | FindingType::AwsSecretKey => {
                "AWS credentials detected. Remove and use AWS IAM roles or environment variables.".to_string()
            }
            FindingType::GitHubToken => {
                "GitHub token detected. Revoke immediately and use GitHub Apps or environment variables.".to_string()
            }
            FindingType::RsaPrivateKey | FindingType::SshPrivateKey => {
                "Private key detected. Never commit private keys. Use key management systems.".to_string()
            }
            FindingType::DatabaseUrl => {
                "Database connection string detected. Use environment variables or secret management.".to_string()
            }
            FindingType::Email => {
                "Email address detected. Consider if this PII should be in version control.".to_string()
            }
            FindingType::CreditCard => {
                "Possible credit card number detected. This is PCI-DSS violation if real.".to_string()
            }
            FindingType::SocialSecurityNumber => {
                "Possible SSN detected. This is a serious PII violation if real.".to_string()
            }
            _ => format!("{} detected. Review before committing.", finding_type.name()),
        }
    }

    /// Redact sensitive portions of matched text
    pub fn redacted_match(&self) -> String {
        let len = self.matched_text.len();
        if len <= 8 {
            "*".repeat(len)
        } else {
            format!(
                "{}***{}",
                &self.matched_text[..4],
                &self.matched_text[len - 4..]
            )
        }
    }
}

/// Configuration for the security scanner
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Minimum severity to report
    pub min_severity: Severity,
    /// Enable PII detection
    pub detect_pii: bool,
    /// Enable secret detection
    pub detect_secrets: bool,
    /// Enable high entropy string detection
    pub detect_high_entropy: bool,
    /// File extensions to scan
    pub file_extensions: Vec<String>,
    /// Paths to exclude (glob patterns)
    pub exclude_paths: Vec<String>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            min_severity: Severity::Low,
            detect_pii: true,
            detect_secrets: true,
            detect_high_entropy: false,
            file_extensions: vec![
                "rs".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "py".to_string(),
                "java".to_string(),
                "go".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "h".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "env".to_string(),
                "txt".to_string(),
                "md".to_string(),
            ],
            exclude_paths: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/dist/**".to_string(),
                "**/.git/**".to_string(),
                "**/vendor/**".to_string(),
            ],
        }
    }
}

/// Scan results
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResult {
    /// All findings
    pub findings: Vec<Finding>,
    /// Number of files scanned
    pub files_scanned: usize,
    /// Number of lines scanned
    pub lines_scanned: usize,
    /// Scan duration in milliseconds
    pub duration_ms: u64,
}

impl ScanResult {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            files_scanned: 0,
            lines_scanned: 0,
            duration_ms: 0,
        }
    }

    /// Check if scan passed (no findings above min severity)
    pub fn passed(&self, min_severity: Severity) -> bool {
        !self.findings.iter().any(|f| f.severity >= min_severity)
    }

    /// Get findings by severity
    pub fn by_severity(&self, severity: Severity) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .collect()
    }

    /// Get findings by category
    pub fn by_category(&self, category: Category) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.category == category)
            .collect()
    }
}

impl Default for ScanResult {
    fn default() -> Self {
        Self::new()
    }
}
