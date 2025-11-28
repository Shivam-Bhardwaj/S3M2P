.PHONY: all dev-helios dev-toofoo build sync clean check test worktree-list worktree-clean

# Detect OS for specific commands if needed
UNAME_S := $(shell uname -s)

# Default target
all: build

# Development
dev-helios:
	trunk serve helios/index.html

dev-toofoo:
	trunk serve too.foo/index.html

# Quality
check:
	cargo check --workspace

test:
	cargo test --workspace

validate: check test
	trunk build --release helios/index.html
	trunk build --release too.foo/index.html

# Data Pipeline
sync:
	./sync_data.sh

generate-galaxy:
	cd simulation-cli && cargo run --release -- generate --count 100000 --output ../data

# Build
build: build-core build-cli build-web

build-core:
	cargo build --release -p core
	cargo build --release -p storage-server

build-cli:
	cargo build --release -p simulation-cli

build-web:
	trunk build --release helios/index.html
	trunk build --release too.foo/index.html

# Worktree management
worktree-list:
	./scripts/worktree.sh list

worktree-clean:
	./scripts/worktree.sh clean

# Maintenance
clean:
	cargo clean
	rm -rf helios/dist too.foo/dist data/

