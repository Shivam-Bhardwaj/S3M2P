#!/usr/bin/env bash
# S3M2P Dev Server Script
# Usage: ./SCRIPTS/dev-serve.sh <project>
#
# Automatically kills any existing process on the project's port before starting.
# Each project has a dedicated port to allow multiple services to run simultaneously.

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Port assignments for each project
declare -A PROJECT_PORTS=(
    ["welcome"]="8080"
    ["helios"]="8081"
    ["chladni"]="8082"
    ["sensors"]="8083"
    ["autocrate"]="8084"
    ["blog"]="8085"
    ["learn"]="8086"
    ["pll"]="8090"
    ["power"]="8091"
    ["ai"]="8100"
    ["ubuntu"]="8101"
    ["opencv"]="8102"
    ["arduino"]="8103"
    ["esp32"]="8104"
    ["swarm"]="8105"
    ["slam"]="8106"
)

# Project directory mappings
declare -A PROJECT_DIRS=(
    ["welcome"]="WELCOME"
    ["helios"]="HELIOS"
    ["chladni"]="SIMULATIONS/CHLADNI"
    ["sensors"]="TOOLS/SENSORS"
    ["autocrate"]="TOOLS/AUTOCRATE"
    ["blog"]="BLOG"
    ["learn"]="LEARN"
    ["pll"]="TOOLS/PLL"
    ["power"]="TOOLS/POWER_CIRCUITS"
    ["ai"]="LEARN/AI"
    ["ubuntu"]="LEARN/UBUNTU"
    ["opencv"]="LEARN/OPENCV"
    ["arduino"]="LEARN/ARDUINO"
    ["esp32"]="LEARN/ESP32"
    ["swarm"]="LEARN/SWARM_ROBOTICS"
    ["slam"]="LEARN/SLAM"
)

usage() {
    echo "S3M2P Dev Server"
    echo ""
    echo "Usage: $0 <project>"
    echo ""
    echo "Projects and their ports:"
    echo "  welcome   (8080)  - too.foo landing page"
    echo "  helios    (8081)  - Solar system visualization"
    echo "  chladni   (8082)  - Chladni wave patterns"
    echo "  sensors   (8083)  - Sensor test tool"
    echo "  autocrate (8084)  - Shipping crate generator"
    echo "  blog      (8085)  - Blog platform"
    echo "  learn     (8086)  - Learning hub"
    echo "  pll       (8090)  - PLL circuit designer"
    echo "  power     (8091)  - Power circuit designer"
    echo "  ai        (8100)  - AI tutorials"
    echo "  ubuntu    (8101)  - Ubuntu tutorials"
    echo "  opencv    (8102)  - OpenCV tutorials"
    echo "  arduino   (8103)  - Arduino tutorials"
    echo "  esp32     (8104)  - ESP32 tutorials"
    echo "  swarm     (8105)  - Swarm robotics tutorials"
    echo "  slam      (8106)  - SLAM tutorials"
    echo ""
    echo "Example: $0 welcome"
}

kill_port() {
    local port="$1"
    local pid
    pid=$(lsof -t -i :"$port" 2>/dev/null || true)

    if [[ -n "$pid" ]]; then
        echo "Killing existing process on port $port (PID: $pid)..."
        kill "$pid" 2>/dev/null || true
        sleep 0.5
    fi
}

serve_project() {
    local project="$1"
    local port="${PROJECT_PORTS[$project]}"
    local dir="${PROJECT_DIRS[$project]}"

    if [[ -z "$port" ]] || [[ -z "$dir" ]]; then
        echo "ERROR: Unknown project '$project'"
        usage
        exit 1
    fi

    local project_path="$REPO_ROOT/$dir"

    if [[ ! -d "$project_path" ]]; then
        echo "ERROR: Project directory not found: $project_path"
        exit 1
    fi

    if [[ ! -f "$project_path/Trunk.toml" ]]; then
        echo "ERROR: No Trunk.toml found in $project_path"
        exit 1
    fi

    # Kill any existing process on this port
    kill_port "$port"

    echo "Starting $project on http://127.0.0.1:$port/"
    echo "Directory: $project_path"
    echo ""

    cd "$project_path"
    exec trunk serve index.html
}

# Main
case "${1:-}" in
    -h|--help|help|"")
        usage
        ;;
    *)
        serve_project "$1"
        ;;
esac
