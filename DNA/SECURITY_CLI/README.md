# DNA Security Scanner

Industry-standard security scanner for detecting secrets and PII in code before deployment.

## Features

- ✅ **Secret Detection**: API keys, passwords, private keys, tokens
- ✅ **PII Detection**: Emails, phone numbers, credit cards, SSNs
- ✅ **Pre-commit Hook**: Block commits with secrets
- ✅ **Configurable**: Adjust severity levels and detection rules
- ✅ **Fast**: Regex-based scanning with minimal overhead

## Industry Standards

Follows guidance from:
- OWASP Top 10 (A02:2021 Cryptographic Failures)
- GDPR Article 32 (Security of processing)
- NIST SP 800-122 (Guide to Protecting PII)
- CWE-798 (Use of Hard-coded Credentials)

Reference tools:
- GitHub Secret Scanning
- Gitleaks
- TruffleHog
- detect-secrets

## Installation

```bash
cd DNA/SECURITY_CLI
cargo build --release
```

Add to PATH (optional):
```bash
export PATH="$PATH:$(pwd)/target/release"
```

## Usage

### Scan a File or Directory

```bash
# Scan a single file
dna-security scan src/main.rs

# Scan a directory
dna-security scan ./DNA

# Scan with specific severity threshold
dna-security scan ./DNA --severity high

# Output as JSON
dna-security scan ./DNA --format json

# Fail with exit code 1 if findings detected
dna-security scan ./DNA --fail-on-findings
```

### Pre-commit Hook

Check staged files before committing:

```bash
dna-security check
```

Install as git hook:

```bash
# From project root
ln -s ../../SCRIPTS/security-check.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Now every commit will be scanned for secrets and PII automatically.

### Test a String

```bash
dna-security test "const AWS_KEY = 'AKIAIOSFODNN7EXAMPLE';"
```

## Detection Patterns

### Secrets

| Type | Pattern | Severity |
|------|---------|----------|
| AWS Access Key | `AKIA[0-9A-Z]{16}` | Critical |
| AWS Secret Key | `aws_secret_access_key` + base64 | Critical |
| GitHub Token | `ghp_*`, `gho_*`, etc. | High |
| Stripe Key | `sk_live_*`, `pk_live_*` | High |
| Generic API Key | `api_key` + high entropy | High |
| Private Key | `-----BEGIN PRIVATE KEY-----` | Critical |
| SSH Key | `-----BEGIN OPENSSH PRIVATE KEY-----` | Critical |
| Database URL | `mongodb://`, `postgres://` | Critical |

### PII

| Type | Pattern | Severity |
|------|---------|----------|
| Email | `user@example.com` | Medium |
| Phone Number | `(123) 456-7890` | Medium |
| Credit Card | Luhn-validated card numbers | Medium |
| SSN | `123-45-6789` | Medium |
| IP Address | `192.168.1.1` | Low |

## Configuration

Configure via `ScanConfig`:

```rust
use dna::security::{ScanConfig, Scanner, Severity};

let mut config = ScanConfig::default();
config.min_severity = Severity::High;
config.detect_pii = false;  // Disable PII detection

let scanner = Scanner::with_config(config);
```

## Examples

### Example 1: Scan Before Push

```bash
#!/bin/bash
# pre-push hook
dna-security check --fail-on-findings || {
    echo "❌ Push blocked due to secrets in code"
    exit 1
}
```

### Example 2: CI/CD Integration

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
        run: cd DNA/SECURITY_CLI && cargo build --release
      - name: Run security scan
        run: DNA/SECURITY_CLI/target/release/dna-security scan . --fail-on-findings
```

### Example 3: Library Usage

```rust
use dna::security::{Scanner, ScanConfig};

let scanner = Scanner::new();
let findings = scanner.scan_file("config.json")?;

for finding in findings {
    println!("{:?}: {} at {}:{}",
        finding.severity,
        finding.finding_type.name(),
        finding.file_path,
        finding.line_number
    );
}
```

## False Positives

To reduce false positives:

1. **Entropy checking**: Generic patterns use Shannon entropy to filter variable names
2. **Luhn validation**: Credit cards validated with checksum algorithm
3. **IP validation**: Excludes version numbers like `1.0.0.0`
4. **Exclude paths**: Configure paths to skip (node_modules, target, etc.)

## Remediation

When secrets are detected:

1. **Remove the secret** from code
2. **Revoke the credential** (assume it's compromised)
3. **Use environment variables** or secret management:
   - AWS: Use IAM roles or AWS Secrets Manager
   - GitHub: Use GitHub Apps or environment secrets
   - General: Use `.env` files (gitignored) or HashiCorp Vault

4. **Rewrite git history** if secret was committed:
   ```bash
   # Use git-filter-repo or BFG Repo-Cleaner
   git filter-repo --path-glob '**/*.env' --invert-paths
   ```

## Testing

```bash
cd DNA/SECURITY_CLI
cargo test
```

## Performance

- ~1M lines/second on modern hardware
- Minimal memory footprint
- Parallel scanning (future enhancement)

## License

MIT
