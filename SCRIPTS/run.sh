#!/bin/bash
# Smart dev server launcher with auto port selection
# Usage: run welcome | run helios | run mcad | run website

set -e

# Port registry (auto-assigns if not in use)
declare -A PORTS=(
    [welcome]=3000
    [helios]=3001
    [mcad]=3002
    [ecad]=3003
    [simulations]=3004
    [blog]=3005
    [learn]=3006
    [arch]=3007
    [website]=3030
)

# Project paths and commands
declare -A PROJECTS=(
    [welcome]="WELCOME/index.html|trunk"
    [helios]="HELIOS/index.html|trunk"
    #[mcad]="MCAD/WEB/index.html|trunk"
    #[ecad]="ECAD/WEB/index.html|trunk"
    [simulations]="SIMULATIONS/CHLADNI/index.html|trunk"
    [blog]="BLOG/index.html|trunk"
    [learn]="LEARN/index.html|trunk"
    [arch]="ARCH/index.html|trunk"
    [website]="/home/curious/workspace/shivambhardwaj.com|leptos"
)

# Function to check if port is in use
port_in_use() {
    lsof -i:$1 >/dev/null 2>&1
}

# Function to find available port
find_port() {
    local base_port=$1
    local port=$base_port
    while port_in_use $port; do
        ((port++))
    done
    echo $port
}

# Function to normalize project name (case insensitive)
normalize() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]'
}

# Function to list available projects
list_projects() {
    echo "Available projects:"
    echo ""
    printf "  %-15s %-10s %s\n" "NAME" "PORT" "PATH"
    echo "  ────────────────────────────────────────────────"
    for proj in "${!PROJECTS[@]}"; do
        local path="${PROJECTS[$proj]%%|*}"
        printf "  %-15s %-10s %s\n" "$proj" "${PORTS[$proj]}" "$path"
    done
    echo ""
    echo "Usage: run <project>  (case insensitive)"
    echo "Example: run welcome | run HELIOS | run mcad"
}

# Main logic
PROJECT=$(normalize "${1:-}")

if [[ -z "$PROJECT" ]] || [[ "$PROJECT" == "list" ]] || [[ "$PROJECT" == "help" ]]; then
    list_projects
    exit 0
fi

# Check if project exists
if [[ ! -v PROJECTS[$PROJECT] ]]; then
    echo "Error: Unknown project '$1'"
    echo ""
    list_projects
    exit 1
fi

# Parse project config
IFS='|' read -r PROJECT_PATH TYPE <<< "${PROJECTS[$PROJECT]}"
BASE_PORT=${PORTS[$PROJECT]}

# Find available port
PORT=$(find_port $BASE_PORT)
if [[ $PORT -ne $BASE_PORT ]]; then
    echo "⚠️  Port $BASE_PORT in use, using $PORT instead"
fi

echo "🚀 Starting $PROJECT on port $PORT..."
echo ""

# Change to appropriate directory
if [[ "$TYPE" == "trunk" ]]; then
    # Extract directory from PROJECT_PATH
    PROJECT_DIR=$(dirname "$PROJECT_PATH")
    PROJECT_FILE=$(basename "$PROJECT_PATH")
    
    cd ~/S3M2P/"$PROJECT_DIR"
    echo "📂 Working directory: ~/S3M2P/$PROJECT_DIR"
    echo "🌐 URL: http://localhost:$PORT"
    echo ""
    exec trunk serve "$PROJECT_FILE" --port $PORT --open
elif [[ "$TYPE" == "leptos" ]]; then
    cd "$PROJECT_PATH"
    echo "📂 Working directory: $PROJECT_PATH"
    echo "🌐 URL: http://localhost:$PORT"
    echo ""
    exec cargo leptos watch --port $PORT
fi
