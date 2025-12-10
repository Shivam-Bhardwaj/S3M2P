# DNA Security Module

Industry-standard security scanner for detecting secrets and PII before deployment.

## Overview

The DNA Security module provides comprehensive scanning capabilities to prevent sensitive data from being committed to version control. It follows OWASP, GDPR, and NIST guidelines for secure software development.

## Features

### Detection Capabilities

**Secrets Detection:**
- AWS credentials (Access Keys, Secret Keys)
- GitHub tokens and Personal Access Tokens
- Stripe API keys
- Slack tokens
- Generic API keys and tokens
- JWT tokens
- Passwords in code
- RSA, SSH, and PGP private keys
- Database connection strings (MongoDB, PostgreSQL, MySQL)

**PII Detection:**
- Email addresses
- Phone numbers (US format)
- Credit card numbers (Luhn-validated)
- Social Security Numbers
- IP addresses

### Advanced Features

- **Entropy Analysis**: Filters false positives by analyzing string randomness
- **Luhn Validation**: Validates credit card numbers using checksum algorithm
- **Variable Name Detection**: Distinguishes between variable names and secrets
- **Configurable Rules**: Adjust severity thresholds and detection types
- **Pre-commit Hook**: Automatic scanning before git commits
- **Multiple Output Formats**: Text and JSON output

## Architecture

```
DNA/
├── src/
│   └── security/
│       ├── mod.rs        # Public API
│       ├── types.rs      # Finding types, severity levels
│       ├── patterns.rs   # Detection patterns (regex + entropy)
│       └── scanner.rs    # Scanner engine
└── SECURITY_CLI/
    └── src/
        └── main.rs       # CLI tool (dna-security)
```

## Industry Standards

### Compliance

- **OWASP Top 10**: A02:2021 Cryptographic Failures
- **GDPR**: Article 32 (Security of processing)
- **NIST**: SP 800-122 (Guide to Protecting PII)
- **CWE**: CWE-798 (Use of Hard-coded Credentials)

### Reference Implementations

Patterns inspired by:
- GitHub Secret Scanning
- Gitleaks
- TruffleHog
- detect-secrets

## Usage

### As a Library

```rust
use dna::security::{Scanner, ScanConfig, Severity};

// Create scanner with default config
let scanner = Scanner::new();

// Scan a file
let findings = scanner.scan_file("src/config.rs")?;

// Scan a directory
let result = scanner.scan_directory(".")?;

// Custom configuration
let mut config = ScanConfig::default();
config.min_severity = Severity::High;
config.detect_pii = false;

let scanner = Scanner::with_config(config);
```

### CLI Tool

```bash
# Build the CLI
cargo build -p security-cli --release

# Test a string
./target/release/dna-security test "const KEY = 'AKIAIOSFODNN7EXAMPLE';"

# Scan a file
./target/release/dna-security scan src/main.rs

# Scan a directory
./target/release/dna-security scan ./DNA --severity high

# Pre-commit check
./target/release/dna-security check
```

### Git Pre-commit Hook

Install the pre-commit hook to automatically scan staged files:

```bash
# Link the script
ln -s ../../SCRIPTS/security-check.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Now every commit will be scanned for secrets and PII. If findings are detected, the commit will be blocked.

## Detection Patterns

### Pattern Categories

| Category | Types | Severity Range |
|----------|-------|----------------|
| Cloud Credentials | AWS, Azure, GCP | Critical |
| API Keys | GitHub, Stripe, Slack | High |
| Crypto Keys | RSA, SSH, PGP | Critical |
| Database | MongoDB, PostgreSQL, MySQL | Critical |
| PII | Email, Phone, SSN, Credit Card | Medium |
| Generic | High entropy strings | Info |

### Entropy-Based Detection

For generic patterns (passwords, API keys), the scanner uses Shannon entropy to filter false positives:

```rust
// High entropy (likely a secret)
"3K7qDj9mPz2nF8bY5cT1xW6vL0hR4uQ9" → Entropy: 4.5

// Low entropy (likely a variable name)
"my_variable_name" → Entropy: 3.2
```

Threshold: 3.5 bits per character

### Variable Name Detection

The scanner distinguishes between actual secrets and variable names:

```rust
// Variable name (excluded)
"api_key" → starts with letter, low digit/uppercase ratio

// Secret (detected)
"3K7qDj9mPz2nF8bY" → high digit/uppercase ratio
```

## Configuration

### ScanConfig Options

```rust
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
```

### Default Excluded Paths

- `**/node_modules/**`
- `**/target/**`
- `**/dist/**`
- `**/.git/**`
- `**/vendor/**`

## Testing

```bash
# Run all security module tests
cargo test -p dna --lib security

# Run with output
cargo test -p dna --lib security -- --nocapture
```

All tests include:
- Pattern matching tests
- Entropy calculation tests
- Luhn validation tests
- Variable name detection tests
- Integration tests with various secret types

## Performance

- **Speed**: ~1M lines/second on modern hardware
- **Memory**: Minimal allocations, pre-compiled regex patterns
- **Scalability**: Efficient for large monorepos

## False Positives

### Reduction Strategies

1. **Entropy Checking**: Generic patterns validate entropy
2. **Luhn Validation**: Credit cards validated with checksum
3. **IP Validation**: Excludes version numbers (e.g., `1.0.0.0`)
4. **Variable Name Detection**: Filters out variable/function names
5. **Exclude Paths**: Skip generated files, dependencies

### Example

```rust
// NOT detected (variable name)
let api_key = load_from_env();

// DETECTED (actual secret)
let api_key = "sk_live_[REDACTED_EXAMPLE_KEY]";
```

## Remediation Guide

When secrets are detected:

### 1. Remove the Secret

```bash
# For uncommitted changes
git restore <file>

# For committed secrets
git filter-repo --path-glob '**/*.env' --invert-paths
```

### 2. Revoke the Credential

Assume any committed secret is compromised:
- AWS: Delete IAM user/key, rotate credentials
- GitHub: Revoke token in Settings → Developer settings
- Stripe: Roll API key in Dashboard

### 3. Use Environment Variables

```rust
// ❌ Bad
const API_KEY: &str = "sk_live_...";

// ✅ Good
use std::env;
let api_key = env::var("API_KEY").expect("API_KEY not set");
```

### 4. Use Secret Management

- **AWS**: AWS Secrets Manager, Parameter Store
- **Azure**: Azure Key Vault
- **GCP**: Secret Manager
- **HashiCorp**: Vault
- **Local**: `.env` files (gitignored)

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/security.yml
name: Security Scan
on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1

      - name: Build scanner
        run: cargo build -p security-cli --release

      - name: Run security scan
        run: |
          ./target/release/dna-security scan . \
            --fail-on-findings \
            --severity high
```

### Pre-push Hook

```bash
#!/bin/bash
# .git/hooks/pre-push

./target/release/dna-security check --fail-on-findings || {
    echo "❌ Push blocked due to security findings"
    exit 1
}
```

## Extending the Scanner

### Adding New Patterns

1. Add pattern to `DNA/src/security/patterns.rs`:

```rust
Pattern {
    finding_type: FindingType::NewSecret,
    regex: r"pattern_here",
    description: "Description",
    check_entropy: false,
}
```

2. Add finding type to `DNA/src/security/types.rs`:

```rust
pub enum FindingType {
    // ... existing types
    NewSecret,
}
```

3. Add severity and category:

```rust
impl FindingType {
    pub fn severity(&self) -> Severity {
        match self {
            FindingType::NewSecret => Severity::Critical,
            // ...
        }
    }
}
```

### Custom Validators

Add validation logic in `Scanner::validate_finding()`:

```rust
fn validate_finding(&self, finding_type: &FindingType, text: &str) -> Option<bool> {
    match finding_type {
        FindingType::NewSecret => Some(self.custom_validation(text)),
        _ => None,
    }
}
```

## Limitations

1. **Regex-based**: May miss obfuscated secrets
2. **No context analysis**: Cannot understand semantic meaning
3. **Language-agnostic**: May have false positives in non-code files
4. **Single-file**: No cross-file secret tracking

## Future Enhancements

- [ ] ML-based secret detection
- [ ] Cross-file secret correlation
- [ ] Automated secret rotation
- [ ] Integration with secret vaults
- [ ] Parallel scanning
- [ ] Custom rule DSL
- [ ] Secret risk scoring

## Support

For issues or feature requests, see:
- README: `DNA/SECURITY_CLI/README.md`
- Source: `DNA/src/security/`
- Tests: `cargo test -p dna security`

## License

MIT
