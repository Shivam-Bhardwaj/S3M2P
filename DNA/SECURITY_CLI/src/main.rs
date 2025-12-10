//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: main.rs | DNA/SECURITY_CLI/src/main.rs
//! PURPOSE: CLI tool for scanning code for secrets and PII
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation tool)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Usage:
//!   dna-security scan <path>           # Scan a file or directory
//!   dna-security check                 # Check current git changes (pre-commit)
//!   dna-security test <text>           # Test a string for secrets

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dna::security::{ScanConfig, Scanner, Severity};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "dna-security")]
#[command(about = "Scan code for secrets and PII before deployment", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a file or directory
    Scan {
        /// Path to scan
        path: PathBuf,

        /// Minimum severity to report (info, low, medium, high, critical)
        #[arg(short, long, default_value = "low")]
        severity: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Exit with error code if findings detected
        #[arg(long)]
        fail_on_findings: bool,

        /// Disable PII detection
        #[arg(long)]
        no_pii: bool,

        /// Disable secret detection
        #[arg(long)]
        no_secrets: bool,
    },

    /// Check current git changes (for pre-commit hook)
    Check {
        /// Exit with error code if findings detected
        #[arg(long, default_value = "true")]
        fail_on_findings: bool,
    },

    /// Test a string for secrets/PII
    Test {
        /// Text to test
        text: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            path,
            severity,
            format,
            fail_on_findings,
            no_pii,
            no_secrets,
        } => {
            let min_severity = parse_severity(&severity)?;
            let mut config = ScanConfig::default();
            config.min_severity = min_severity;
            config.detect_pii = !no_pii;
            config.detect_secrets = !no_secrets;

            let scanner = Scanner::with_config(config);

            if path.is_file() {
                // Scan single file
                let findings = scanner.scan_file(&path)?;
                if format == "json" {
                    print_json(&findings)?;
                } else {
                    print_findings(&findings, &path.to_string_lossy());
                }

                if fail_on_findings && !findings.is_empty() {
                    process::exit(1);
                }
            } else if path.is_dir() {
                // Scan directory
                let result = scanner.scan_directory(&path)?;

                if format == "json" {
                    print_json_result(&result)?;
                } else {
                    print_scan_result(&result);
                }

                if fail_on_findings && !result.findings.is_empty() {
                    process::exit(1);
                }
            } else {
                eprintln!("❌ Path not found: {}", path.display());
                process::exit(1);
            }
        }

        Commands::Check { fail_on_findings } => {
            // Check git staged files
            let staged_files = get_staged_files()?;

            if staged_files.is_empty() {
                println!("✅ No staged files to check");
                return Ok(());
            }

            let scanner = Scanner::new();
            let mut all_findings = Vec::new();

            for file_path in &staged_files {
                let path = PathBuf::from(file_path);
                if path.exists() && path.is_file() {
                    if let Ok(findings) = scanner.scan_file(&path) {
                        all_findings.extend(findings);
                    }
                }
            }

            if all_findings.is_empty() {
                println!("✅ No secrets or PII detected in staged files");
            } else {
                println!(
                    "\n{} {} security findings detected in staged files:\n",
                    "⚠️".red(),
                    all_findings.len()
                );
                for finding in &all_findings {
                    print_finding(finding);
                }

                if fail_on_findings {
                    println!("\n{} Commit blocked due to security findings", "❌".red());
                    println!("Fix the issues above or use --no-verify to bypass (not recommended)");
                    process::exit(1);
                }
            }
        }

        Commands::Test { text } => {
            let scanner = Scanner::new();
            let findings = scanner.scan_text(&text, "<test>");

            if findings.is_empty() {
                println!("✅ No secrets or PII detected");
            } else {
                println!("⚠️  {} findings detected:\n", findings.len());
                for finding in &findings {
                    print_finding(finding);
                }
            }
        }
    }

    Ok(())
}

fn parse_severity(s: &str) -> Result<Severity> {
    match s.to_lowercase().as_str() {
        "info" => Ok(Severity::Info),
        "low" => Ok(Severity::Low),
        "medium" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        "critical" => Ok(Severity::Critical),
        _ => Err(anyhow::anyhow!("Invalid severity level: {}", s)),
    }
}

fn get_staged_files() -> Result<Vec<String>> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACM"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get staged files from git"));
    }

    let files = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}

fn print_finding(finding: &dna::security::Finding) {
    let severity_icon = finding.severity.emoji();
    let severity_text = format!("{:?}", finding.severity);

    println!("  {} {} {}", severity_icon, severity_text.bold(), finding.finding_type.name());
    println!("     📄 {}:{}", finding.file_path.cyan(), finding.line_number);
    println!("     🔍 {}", finding.matched_text.yellow());
    println!("     💡 {}", finding.description.dimmed());
    println!();
}

fn print_findings(findings: &[dna::security::Finding], path: &str) {
    if findings.is_empty() {
        println!("✅ No secrets or PII detected in {}", path.green());
    } else {
        println!(
            "\n{} {} security findings detected in {}:\n",
            "⚠️".red(),
            findings.len(),
            path.yellow()
        );
        for finding in findings {
            print_finding(finding);
        }
    }
}

fn print_scan_result(result: &dna::security::ScanResult) {
    println!("\n{}", "Security Scan Report".bold().underline());
    println!("📊 Files scanned: {}", result.files_scanned);
    println!("📝 Lines scanned: {}", result.lines_scanned);
    println!("⏱️  Duration: {}ms", result.duration_ms);
    println!();

    if result.findings.is_empty() {
        println!("{} No security issues detected!", "✅".green());
    } else {
        println!(
            "{} {} security findings detected:\n",
            "⚠️".red(),
            result.findings.len()
        );

        // Group by severity
        for severity in [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ] {
            let findings = result.by_severity(severity);
            if !findings.is_empty() {
                println!("  {} {:?}: {} findings", severity.emoji(), severity, findings.len());
            }
        }

        println!("\nDetails:\n");
        for finding in &result.findings {
            print_finding(finding);
        }
    }
}

fn print_json(findings: &[dna::security::Finding]) -> Result<()> {
    let json = serde_json::to_string_pretty(findings)?;
    println!("{}", json);
    Ok(())
}

fn print_json_result(result: &dna::security::ScanResult) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    println!("{}", json);
    Ok(())
}
