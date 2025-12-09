#!/bin/bash
# S3M2P Configuration - Single Source of Truth
# Sourced by other scripts

# Repository Root
# If sourced from SCRIPTS dir, use parent. If from root, use .
if [[ -d "SCRIPTS" ]]; then
    REPO_ROOT="$(pwd)"
elif [[ -f "config.sh" ]]; then
    REPO_ROOT="$(cd .. && pwd)"
else
    REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
fi

# Log Directory
LOG_DIR="$REPO_ROOT/logs"

# Project Port Assignments
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

# Project Directory Mappings (Relative to REPO_ROOT)
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

# Export for sub-shells
export REPO_ROOT
export LOG_DIR
export PROJECT_PORTS
export PROJECT_DIRS
