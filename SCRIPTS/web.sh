#!/bin/bash
# Open GitHub pages for the current context
# Usage: web [issue|pr|branch|repo]

ACTION="${1:-repo}"
REPO_URL="https://github.com/Shivam-Bhardwaj/S3M2P"

# Get current branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Extract issue number from branch (e.g., issue-42 -> 42)
if [[ "$BRANCH" =~ issue-([0-9]+) ]]; then
    ISSUE_NUM="${BASH_REMATCH[1]}"
fi

open_url() {
    echo "Opening $1..."
    if command -v xdg-open &> /dev/null; then
        xdg-open "$1"
    elif command -v python3 &> /dev/null; then
        python3 -m webbrowser "$1"
    else
        echo "Could not detect web browser. URL: $1"
    fi
}

case "$ACTION" in
    issue)
        if [ -n "$ISSUE_NUM" ]; then
            open_url "$REPO_URL/issues/$ISSUE_NUM"
        else
            echo "Current branch '$BRANCH' does not look like an issue branch (issue-N)."
            echo "Opening issues page..."
            open_url "$REPO_URL/issues"
        fi
        ;;
    pr)
        # Try to find open PR for this branch
        PR_URL=$(gh pr view --json url --jq .url 2>/dev/null)
        if [ -n "$PR_URL" ]; then
            open_url "$PR_URL"
        else
            echo "No open PR found for branch '$BRANCH'."
            echo "Opening Pull Requests page..."
            open_url "$REPO_URL/pulls"
        fi
        ;;
    branch)
        open_url "$REPO_URL/tree/$BRANCH"
        ;;
    repo)
        open_url "$REPO_URL"
        ;;
    *)
        echo "Usage: web [issue|pr|branch|repo]"
        exit 1
        ;;
esac
