#!/usr/bin/env python3
"""
Run Claude agent for GitHub automation.

This script uses claude in print mode (-p) which:
- Runs non-interactively
- Outputs response to stdout
- Can use Bash tool for `gh` CLI commands

We use `gh` CLI instead of MCP tools because:
1. Print mode doesn't support MCP servers
2. `gh` is already authenticated and allowed in permissions
3. Simpler and more reliable

GitHub interaction pattern:
- Read issue: gh issue view N --json body,comments
- Post comment: gh issue comment N --body "..."
- Create PR: gh pr create --title "..." --body "..."
"""
import subprocess
import sys
import os
import argparse
import signal
import time

# Main project path
MAIN_PROJECT_PATH = "/home/curious/S3M2P"

def log(msg):
    """Log to stderr with timestamp"""
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{timestamp}] [run_agent] {msg}", file=sys.stderr, flush=True)

def main():
    parser = argparse.ArgumentParser(description="Run claude agent")
    parser.add_argument("--model", required=True, help="Claude model to use")
    parser.add_argument("--prompt-file", required=True, help="Path to file containing the prompt")
    parser.add_argument("--worktree", required=True, help="Working directory")
    args = parser.parse_args()

    # Read prompt
    with open(args.prompt_file, "r") as f:
        prompt = f.read()

    log(f"Starting claude with model={args.model}")
    log(f"Worktree: {args.worktree}")
    log(f"Prompt length: {len(prompt)} chars")

    # Truncate prompt if too long (keep last 50KB to stay under shell limits)
    MAX_PROMPT_SIZE = 50000
    if len(prompt) > MAX_PROMPT_SIZE:
        log(f"Truncating prompt from {len(prompt)} to {MAX_PROMPT_SIZE} chars")
        prompt = "...[truncated]\n\n" + prompt[-MAX_PROMPT_SIZE:]

    # Get issue number from environment for logging
    issue_number = os.environ.get("ISSUE_NUMBER", "unknown")

    # Build the enhanced prompt that includes instructions to use gh CLI
    enhanced_prompt = f"""You are working in the repository at {args.worktree}.

IMPORTANT: To interact with GitHub, use the `gh` CLI tool via Bash:
- Read issue: gh issue view <number> --repo Shivam-Bhardwaj/S3M2P --json title,body,comments
- Post comment: gh issue comment <number> --repo Shivam-Bhardwaj/S3M2P --body "<message>"
- Create PR: gh pr create --repo Shivam-Bhardwaj/S3M2P --title "<title>" --body "<body>"

Your task:
{prompt}

After completing your task, ALWAYS post a summary comment to the GitHub issue using `gh issue comment`.
"""

    # Write enhanced prompt to temp file to avoid "Argument list too long" error
    import tempfile
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write(enhanced_prompt)
        prompt_temp_file = f.name

    log(f"Prompt written to temp file ({len(enhanced_prompt)} chars)")

    # Build command - read prompt from stdin
    cmd = [
        "claude",
        "--model", args.model,
        "--permission-mode", "bypassPermissions",
        "-p", "-",  # Print mode, read from stdin
    ]

    log(f"Running claude -p with prompt via stdin for issue #{issue_number}")

    # Use Popen for better control and streaming
    process = None
    try:
        with open(prompt_temp_file, 'r') as prompt_input:
            process = subprocess.Popen(
                cmd,
                cwd=args.worktree,
                stdin=prompt_input,
                stdout=sys.stdout,  # Stream directly to our stdout (captured by parent)
                stderr=subprocess.STDOUT,  # Combine stderr into stdout
                env=os.environ,
                text=True,
            )

        # Wait with timeout (20 minutes for complex tasks)
        timeout_seconds = 1200
        start_time = time.time()

        while process.poll() is None:
            elapsed = time.time() - start_time
            if elapsed > timeout_seconds:
                log(f"Timeout after {timeout_seconds} seconds, killing process")
                process.kill()
                process.wait()
                os.unlink(prompt_temp_file)
                return 1

            # Log progress every minute
            if int(elapsed) % 60 == 0 and int(elapsed) > 0:
                log(f"Still running... ({int(elapsed)}s elapsed)")

            time.sleep(1)

        returncode = process.returncode

        # Clean up temp file
        os.unlink(prompt_temp_file)

        log(f"Claude exited with code {returncode} for issue #{issue_number}")
        return returncode

    except Exception as e:
        log(f"Error: {e}")
        if process and process.poll() is None:
            process.kill()
        if os.path.exists(prompt_temp_file):
            os.unlink(prompt_temp_file)
        return 1

if __name__ == "__main__":
    sys.exit(main())
