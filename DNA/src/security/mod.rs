//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/security/mod.rs
//! PURPOSE: Security scanning - detect secrets and PII before deployment
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Industry Standards:
//! - OWASP Top 10 (A02:2021 Cryptographic Failures)
//! - GDPR Article 32 (Security of processing)
//! - NIST SP 800-122 (Guide to Protecting PII)
//! - CWE-798 (Use of Hard-coded Credentials)
//!
//! Reference Tools:
//! - GitHub Secret Scanning
//! - Gitleaks
//! - TruffleHog
//! - detect-secrets

pub mod patterns;
pub mod scanner;
pub mod types;

pub use patterns::*;
pub use scanner::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_secret_detection() {
        let scanner = Scanner::new();
        let test_code = "const AWS_KEY = 'AKIAIOSFODNN7EXAMPLE';";

        let findings = scanner.scan_text(test_code, "test.js");

        assert!(!findings.is_empty(), "Should detect AWS key");
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_email_pii_detection() {
        let scanner = Scanner::new();
        let test_text = "Contact: john.doe@example.com";

        let findings = scanner.scan_text(test_text, "readme.txt");

        assert!(!findings.is_empty(), "Should detect email");
        assert_eq!(findings[0].category, Category::PII);
    }
}
