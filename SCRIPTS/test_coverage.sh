#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== S3M2P Regression Testing ===${NC}"

# 1. Run Standard Tests
echo -e "\n${GREEN}[1/3] Running Cargo Tests...${NC}"
if cargo test --workspace; then
    echo -e "${GREEN}Tests passed!${NC}"
else
    echo -e "${RED}Tests failed!${NC}"
    exit 1
fi

# 2. Run Clippy (Linting Safeguard)
echo -e "\n${GREEN}[2/3] Running Clippy...${NC}"
if cargo clippy --workspace -- -D warnings; then
    echo -e "${GREEN}Clippy passed!${NC}"
else
    echo -e "${RED}Clippy failed!${NC}"
    exit 1
fi

# 3. Code Coverage (if available)
echo -e "\n${GREEN}[3/3] Checking Code Coverage...${NC}"
if command -v cargo-llvm-cov &> /dev/null; then
    echo "Generating coverage report..."
    cargo llvm-cov --workspace --summary-only
    # Optional: Generate HTML report
    # cargo llvm-cov --workspace --html --output-dir target/llvm-cov/html
    echo -e "${GREEN}Coverage check complete.${NC}"
else
    echo "cargo-llvm-cov not found. Skipping coverage report."
    echo "Install with: cargo install cargo-llvm-cov"
fi

echo -e "\n${GREEN}=== System Safeguarded! All checks passed. ===${NC}"
