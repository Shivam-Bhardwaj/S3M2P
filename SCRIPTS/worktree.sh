#!/usr/bin/env bash
# S3M2P Worktree Management Script
# Usage: ./scripts/worktree.sh <command> [options]
#
# Commands:
#   create <issue-number>  - Create worktree for an issue (fetches project from GitHub)
#   list                   - List all worktrees
#   clean                  - Remove prunable worktrees
#   goto <issue-number>    - Print path to worktree for issue
#   remove <issue-number>  - Remove worktree for issue

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKTREE_BASE="$(dirname "$REPO_ROOT")"

# Project aliases for branch naming
declare -A PROJECT_ALIASES=(
    ["dna"]="dna"
    ["DNA"]="dna"
    ["helios"]="helios"
    ["HELIOS"]="helios"
    ["too.foo"]="toofoo"
    ["too-foo"]="toofoo"
    ["toofoo"]="toofoo"
    ["TOOFOO"]="toofoo"
    ["autocrate"]="autocrate"
    ["AUTOCRATE"]="autocrate"
    ["chladni"]="chladni"
    ["CHLADNI"]="chladni"
    ["portfolio"]="portfolio"
    ["PORTFOLIO"]="portfolio"
    ["storage-server"]="storage"
    ["storage"]="storage"
    ["STORAGE_SERVER"]="storage"
    ["simulation-cli"]="simcli"
    ["simcli"]="simcli"
    ["SIMULATION_CLI"]="simcli"
    ["pll"]="pll"
    ["PLL"]="pll"
    ["PROJECT_N"]="projectn"
    ["projectn"]="projectn"
    ["blog"]="blog"
    ["BLOG"]="blog"
    ["learn"]="learn"
    ["LEARN"]="learn"
    ["ml"]="ml"
    ["ML"]="ml"
    ["infra"]="infra"
)

# Valid projects - UPPERCASE categories/projects
VALID_PROJECTS="DNA SIM/HELIOS SIM/TOOFOO SW/AUTOCRATE SW/CHLADNI SW/PORTFOLIO TOOLS/SIMULATION_CLI TOOLS/STORAGE_SERVER TOOLS/PLL BLOG LEARN PROJECT_N infra"

usage() {
    echo "S3M2P Worktree Manager"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  create <issue-number>   Create worktree for GitHub issue"
    echo "  list                    List all worktrees"
    echo "  clean                   Remove prunable worktrees"
    echo "  goto <issue-number>     Print worktree path for issue"
    echo "  remove <issue-number>   Remove worktree for issue"
    echo ""
    echo "Examples:"
    echo "  $0 create 23            # Creates worktree for issue #23"
    echo "  $0 list                 # Shows all worktrees"
    echo "  $0 goto 23              # Prints path to issue #23 worktree"
}

get_issue_info() {
    local issue_num="$1"

    # Fetch issue from GitHub
    local issue_json
    issue_json=$(gh issue view "$issue_num" --json title,labels,body 2>/dev/null) || {
        echo "ERROR: Could not fetch issue #$issue_num from GitHub" >&2
        echo "Make sure you're authenticated with 'gh auth login'" >&2
        return 1
    }

    echo "$issue_json"
}

extract_project() {
    local issue_json="$1"

    # Try to extract from labels first (project:xxx)
    local project
    project=$(echo "$issue_json" | jq -r '.labels[]?.name // empty' | grep -oP 'project:\K\S+' | head -1)

    if [[ -z "$project" ]]; then
        # Try to extract from title [project] prefix
        project=$(echo "$issue_json" | jq -r '.title' | grep -oP '^\[\K[^\]]+' | head -1)
    fi

    if [[ -z "$project" ]]; then
        echo "ERROR: Could not determine project from issue" >&2
        echo "Issue should have a 'project:xxx' label or [project] title prefix" >&2
        return 1
    fi

    # Normalize project name
    local alias="${PROJECT_ALIASES[$project]:-}"
    if [[ -z "$alias" ]]; then
        echo "ERROR: Unknown project '$project'" >&2
        echo "Valid projects: $VALID_PROJECTS" >&2
        return 1
    fi

    echo "$alias"
}

create_worktree() {
    local issue_num="$1"

    if [[ -z "$issue_num" ]]; then
        echo "ERROR: Issue number required" >&2
        usage
        return 1
    fi

    echo "Fetching issue #$issue_num from GitHub..."
    local issue_json
    issue_json=$(get_issue_info "$issue_num") || return 1

    local project
    project=$(extract_project "$issue_json") || return 1

    local title
    title=$(echo "$issue_json" | jq -r '.title' | sed 's/\[.*\]//' | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9' '-' | sed 's/^-//;s/-$//' | cut -c1-30)

    local branch_name="${project}/issue-${issue_num}"
    local worktree_path="${WORKTREE_BASE}/S3M2P-${project}-${issue_num}"

    echo "Project: $project"
    echo "Branch: $branch_name"
    echo "Path: $worktree_path"

    # Check if worktree already exists
    if git -C "$REPO_ROOT" worktree list | grep -q "$worktree_path"; then
        echo ""
        echo "Worktree already exists at: $worktree_path"
        return 0
    fi

    # Create branch if it doesn't exist
    if ! git -C "$REPO_ROOT" show-ref --verify --quiet "refs/heads/$branch_name"; then
        echo "Creating branch: $branch_name"
        git -C "$REPO_ROOT" branch "$branch_name" main
    fi

    # Create worktree
    echo "Creating worktree..."
    git -C "$REPO_ROOT" worktree add "$worktree_path" "$branch_name"

    echo ""
    echo "Worktree created successfully!"
    echo "  Path: $worktree_path"
    echo "  Branch: $branch_name"
    echo ""
    echo "To start working:"
    echo "  cd $worktree_path"
    echo "  claude"
}

list_worktrees() {
    echo "S3M2P Worktrees:"
    echo ""
    git -C "$REPO_ROOT" worktree list
}

clean_worktrees() {
    echo "Cleaning prunable worktrees..."
    git -C "$REPO_ROOT" worktree prune
    echo "Done."
}

goto_worktree() {
    local issue_num="$1"

    if [[ -z "$issue_num" ]]; then
        echo "ERROR: Issue number required" >&2
        return 1
    fi

    # Find worktree matching issue number
    local path
    path=$(git -C "$REPO_ROOT" worktree list | grep -E "issue-${issue_num}\b" | awk '{print $1}')

    if [[ -z "$path" ]]; then
        echo "ERROR: No worktree found for issue #$issue_num" >&2
        echo "Create one with: $0 create $issue_num" >&2
        return 1
    fi

    echo "$path"
}

remove_worktree() {
    local issue_num="$1"

    if [[ -z "$issue_num" ]]; then
        echo "ERROR: Issue number required" >&2
        return 1
    fi

    local path
    path=$(goto_worktree "$issue_num" 2>/dev/null) || {
        echo "ERROR: No worktree found for issue #$issue_num" >&2
        return 1
    }

    echo "Removing worktree: $path"
    git -C "$REPO_ROOT" worktree remove "$path"
    echo "Done."
}

# Main command dispatch
case "${1:-}" in
    create)
        create_worktree "$2"
        ;;
    list)
        list_worktrees
        ;;
    clean)
        clean_worktrees
        ;;
    goto)
        goto_worktree "$2"
        ;;
    remove)
        remove_worktree "$2"
        ;;
    -h|--help|help|"")
        usage
        ;;
    *)
        echo "ERROR: Unknown command '$1'" >&2
        usage
        exit 1
        ;;
esac
