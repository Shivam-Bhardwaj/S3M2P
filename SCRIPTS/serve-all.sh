#!/usr/bin/env bash
# S3M2P - Serve All Projects
# Starts all dev servers in the background and provides a summary.

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$REPO_ROOT/logs"
mkdir -p "$LOG_DIR"

# Port assignments (must match dev-serve.sh)
declare -A PROJECT_PORTS=(
    ["welcome"]="8080"
    ["helios"]="8081"
    ["chladni"]="8082"
    ["sensors"]="8083"
    ["autocrate"]="8084"
    ["blog"]="8085"
    ["learn"]="8086"
    ["arch"]="8087"
    ["pll"]="8090"
    ["power"]="8091"
    ["ai"]="8100"
    ["ubuntu"]="8101"
    ["opencv"]="8102"
    ["arduino"]="8103"
    ["esp32"]="8104"
    ["swarm"]="8105"
    ["slam"]="8106"
    ["coming_soon"]="8107"
)

# Project directory mappings (must match dev-serve.sh)
declare -A PROJECT_DIRS=(
    ["welcome"]="WELCOME"
    ["helios"]="HELIOS"
    ["chladni"]="SIMULATIONS/CHLADNI"
    ["sensors"]="LEARN/SENSORS"
    ["autocrate"]="TOOLS/AUTOCRATE"
    ["blog"]="BLOG"
    ["learn"]="LEARN"
    ["arch"]="ARCH"
    ["pll"]="TOOLS/PLL"
    ["power"]="TOOLS/POWER_CIRCUITS"
    ["ai"]="LEARN/AI"
    ["ubuntu"]="LEARN/UBUNTU"
    ["opencv"]="LEARN/OPENCV"
    ["arduino"]="LEARN/ARDUINO"
    ["esp32"]="LEARN/ESP32"
    ["swarm"]="LEARN/SWARM_ROBOTICS"
    ["slam"]="LEARN/SLAM"
    ["coming_soon"]="COMING_SOON"
)

kill_port() {
    local port="$1"
    local pid
    pid=$(lsof -t -i :"$port" 2>/dev/null || true)

    if [[ -n "$pid" ]]; then
        kill "$pid" 2>/dev/null || true
    fi
}

echo "🚀 Starting all S3M2P dev servers..."
echo "Logs will be written to $LOG_DIR"
echo ""

printf "%-15s %-10s %-35s %s\n" "PROJECT" "PORT" "URL" "STATUS"
echo "────────────────────────────────────────────────────────────────────────────────"

# Sort keys for consistent output
IFS=$'\n' sorted_projects=($(sort <<<"${!PROJECT_PORTS[*]}"))
unset IFS

for project in "${sorted_projects[@]}"; do
    port="${PROJECT_PORTS[$project]}"
    dir="${PROJECT_DIRS[$project]}"
    project_path="$REPO_ROOT/$dir"
    log_file="$LOG_DIR/$project.log"

    # Kill existing
    kill_port "$port"

    # Start in background
    if [[ -d "$project_path" ]]; then
        # Check for Trunk.toml OR index.html (for static sites like COMING_SOON)
        if [[ -f "$project_path/Trunk.toml" ]]; then
            (
                cd "$project_path"
                trunk serve index.html --port "$port" > "$log_file" 2>&1
            ) &
            sleep 0.2
            printf "%-15s %-10s %-35s %s\n" "$project" "$port" "http://127.0.0.1:$port" "✅ Started (Trunk)"
        elif [[ -f "$project_path/index.html" ]]; then
            (
                cd "$project_path"
                trunk serve index.html --port "$port" > "$log_file" 2>&1
            ) &
            sleep 0.2
            printf "%-15s %-10s %-35s %s\n" "$project" "$port" "http://127.0.0.1:$port" "✅ Started (Static)"
        else
            printf "%-15s %-10s %-35s %s\n" "$project" "$port" "-" "❌ No Entry Point"
        fi
    else
        printf "%-15s %-10s %-35s %s\n" "$project" "$port" "-" "❌ Not Found"
    fi
done

echo ""
echo "All servers running in background."
echo "To stop all servers, run: pkill trunk"
echo "To view logs: tail -f $LOG_DIR/*.log"