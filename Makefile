.PHONY: all dev-helios dev-toofoo build sync clean

# Detect OS for specific commands if needed
UNAME_S := $(shell uname -s)

# Default target
all: build

# Development
dev-helios:
	cd helios && trunk serve

dev-toofoo:
	cd too.foo && trunk serve

# Data Pipeline
sync:
	./sync_data.sh

generate-galaxy:
	cd simulation-cli && cargo run --release -- generate --count 100000 --output ../data

# Build
build: build-core build-cli build-web

build-core:
	cargo build --release -p antimony-core
	cargo build --release -p storage-server

build-cli:
	cargo build --release -p simulation-cli

build-web:
	cd helios && trunk build --release
	cd too.foo && trunk build --release

# Maintenance
clean:
	cargo clean
	rm -rf helios/dist too.foo/dist data/

