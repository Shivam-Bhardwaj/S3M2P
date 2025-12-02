#!/bin/bash
# Start the GitHub issue poller in the background
# Usage: ./start_poller.sh [start|stop|status|logs|watch|agents|verbose]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
POLLER_SCRIPT="$SCRIPT_DIR/poll_issues.py"
DATA_DIR="$SCRIPT_DIR/.data"
PID_FILE="$DATA_DIR/poller.pid"
LOG_FILE="$DATA_DIR/poller.log"
LOGS_DIR="$DATA_DIR/logs"
DB_FILE="$DATA_DIR/poll_issues.db"

mkdir -p "$DATA_DIR" "$LOGS_DIR"

start() {
    local verbose="$1"
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            echo "Poller already running (PID=$PID)"
            return 1
        fi
        rm -f "$PID_FILE"
    fi

    echo "Starting poller..."
    if [ "$verbose" = "--verbose" ]; then
        echo "  Verbose mode enabled"
        CLAUDE_VERBOSE=1 nohup python3 "$POLLER_SCRIPT" >> "$LOG_FILE" 2>&1 &
    else
        nohup python3 "$POLLER_SCRIPT" >> "$LOG_FILE" 2>&1 &
    fi
    echo $! > "$PID_FILE"
    echo "Poller started (PID=$!)"
    echo "Logs: $LOG_FILE"
    echo ""
    echo "Commands:"
    echo "  ./start_poller.sh logs     - Follow poller logs"
    echo "  ./start_poller.sh watch N  - Watch Claude agent for issue N"
    echo "  ./start_poller.sh agents   - Show all active agents"
}

stop() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            echo "Stopping poller (PID=$PID)..."
            kill "$PID"
            rm -f "$PID_FILE"
            echo "Stopped"
            return 0
        fi
        rm -f "$PID_FILE"
    fi
    echo "Poller not running"
}

status() {
    echo "=== Poller Status ==="
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            echo "Poller: RUNNING (PID=$PID)"
        else
            echo "Poller: STOPPED (stale PID file)"
            rm -f "$PID_FILE"
        fi
    else
        echo "Poller: STOPPED"
    fi

    echo ""
    echo "=== Active Agents ==="
    if [ -f "$DB_FILE" ]; then
        sqlite3 "$DB_FILE" "SELECT issue_number, pid, started_at, log_file FROM active_agents" 2>/dev/null | while IFS='|' read -r issue pid started log; do
            if kill -0 "$pid" 2>/dev/null; then
                status="RUNNING"
            else
                status="DEAD"
            fi
            echo "  Issue #$issue: PID=$pid ($status) - started $started"
            if [ -n "$log" ] && [ -f "$log" ]; then
                echo "    Log: $log"
            fi
        done
    else
        echo "  No database found"
    fi

    echo ""
    echo "=== Recent Log Files ==="
    ls -lt "$LOGS_DIR"/*.log 2>/dev/null | head -5 | while read -r line; do
        echo "  $line"
    done
    if [ ! "$(ls -A "$LOGS_DIR" 2>/dev/null)" ]; then
        echo "  No log files yet"
    fi
}

logs() {
    if [ -f "$LOG_FILE" ]; then
        echo "Following $LOG_FILE (Ctrl+C to stop)..."
        echo ""
        tail -f "$LOG_FILE"
    else
        echo "No log file found at $LOG_FILE"
        echo "Start the poller first: ./start_poller.sh start"
    fi
}

watch_issue() {
    local issue="$1"
    if [ -z "$issue" ]; then
        echo "Usage: ./start_poller.sh watch <issue-number>"
        echo ""
        echo "Example: ./start_poller.sh watch 17"
        return 1
    fi

    # Find the latest log file for this issue
    local latest_log=$(ls -t "$LOGS_DIR"/issue-"$issue"-*.log 2>/dev/null | head -1)

    if [ -z "$latest_log" ]; then
        echo "No log file found for issue #$issue"
        echo ""
        echo "Available logs:"
        ls -lt "$LOGS_DIR"/*.log 2>/dev/null | head -10
        return 1
    fi

    echo "=== Watching Claude Agent for Issue #$issue ==="
    echo "Log file: $latest_log"
    echo ""
    echo "Press Ctrl+C to stop watching"
    echo "==========================================="
    echo ""
    tail -f "$latest_log"
}

agents() {
    echo "=== Active Claude Agents ==="
    echo ""

    if [ ! -f "$DB_FILE" ]; then
        echo "No database found. Start the poller first."
        return 1
    fi

    local has_agents=false
    sqlite3 "$DB_FILE" "SELECT issue_number, pid, started_at, log_file FROM active_agents" 2>/dev/null | while IFS='|' read -r issue pid started log; do
        has_agents=true
        echo "Issue #$issue"
        echo "  PID: $pid"
        if kill -0 "$pid" 2>/dev/null; then
            echo "  Status: RUNNING"
        else
            echo "  Status: EXITED"
        fi
        echo "  Started: $started"

        if [ -n "$log" ] && [ -f "$log" ]; then
            echo "  Log: $log"
            echo ""
            echo "  Last 5 lines of output:"
            tail -5 "$log" | sed 's/^/    /'
        fi
        echo ""
        echo "---"
    done

    if [ "$has_agents" = false ]; then
        echo "No active agents"
    fi

    echo ""
    echo "=== Recent Sessions ==="
    ls -lt "$LOGS_DIR"/*.log 2>/dev/null | head -5 | while read -r perms links owner group size month day time file; do
        # Extract issue number from filename
        basename "$file"
    done
}

verbose() {
    echo "Starting poller in verbose mode (foreground)..."
    echo "Press Ctrl+C to stop"
    echo ""
    CLAUDE_VERBOSE=1 python3 "$POLLER_SCRIPT"
}

case "${1:-start}" in
    start)
        start "$2"
        ;;
    stop)
        stop
        ;;
    status)
        status
        ;;
    logs)
        logs
        ;;
    watch)
        watch_issue "$2"
        ;;
    agents)
        agents
        ;;
    verbose)
        verbose
        ;;
    restart)
        stop
        sleep 1
        start "$2"
        ;;
    *)
        echo "GitHub Issue Poller - Claude Automation"
        echo ""
        echo "Usage: $0 {command}"
        echo ""
        echo "Commands:"
        echo "  start [--verbose]  Start the poller (background)"
        echo "  stop               Stop the poller"
        echo "  status             Show poller and agent status"
        echo "  logs               Follow poller logs"
        echo "  watch <issue>      Watch Claude output for an issue"
        echo "  agents             Show active agents with recent output"
        echo "  verbose            Run poller in foreground with debug logs"
        echo "  restart [--verbose] Restart the poller"
        echo ""
        echo "Data directory: $DATA_DIR"
        exit 1
        ;;
esac
