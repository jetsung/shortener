# Justfile for shortener project
# https://github.com/casey/just

# Default recipe to display help
default:
    @just --list

# ============================================================================
# Build
# ============================================================================

alias b := build

# Build all packages
build:
    cargo build --release

# Build server only
build-server:
    cargo build --release -p shortener-server

# Build CLI only
build-cli:
    cargo build --release -p shortener-cli

# Clean build artifacts
clean:
    cargo clean

# ============================================================================
# Run
# ============================================================================

# Run server
run:
    cargo run -p shortener-server

# Run CLI
run-cli *ARGS:
    cargo run -p shortener-cli -- {{ARGS}}

# ============================================================================
# Test
# ============================================================================

alias t := test

# Run all tests
test:
    cargo test --all

# Run tests with output
test-verbose:
    cargo test --all -- --nocapture

# Run benchmarks
bench:
    cargo bench --all

# ============================================================================
# Code Quality
# ============================================================================

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all checks
check: fmt-check clippy test

# ============================================================================
# Docker
# ============================================================================

# Build Docker image (Debian)
docker-build:
    docker build -f docker/Dockerfile -t shortener-server:latest .

# Build Docker image (Alpine)
docker-build-alpine:
    docker build -f docker/Dockerfile.alpine -t shortener-server:alpine .

# Run with docker compose
docker-run:
    docker compose -f docker/docker-compose.yml up -d

# Run development environment
docker-run-dev:
    docker compose -f docker/docker-compose.dev.yml up -d

# Stop Docker containers
docker-stop:
    docker compose -f docker/docker-compose.yml down
    docker compose -f docker/docker-compose.dev.yml down

# View Docker logs
docker-logs:
    docker compose -f docker/docker-compose.yml logs -f

# ============================================================================
# Cross-compilation
# ============================================================================

# Build for all targets
cross-all:
    ./scripts/build-cross.sh --all

# Build server for all targets
cross-server:
    ./scripts/build-cross.sh --server

# Build CLI for all targets
cross-cli:
    ./scripts/build-cross.sh --cli

# Build for specific target
cross-target TARGET PACKAGE:
    ./scripts/build-cross.sh -t {{TARGET}} -p {{PACKAGE}}

# List available cross-compilation targets
cross-list:
    ./scripts/build-cross.sh --list

# ============================================================================
# Release
# ============================================================================

# Create a new release
release VERSION:
    @echo "Creating release {{VERSION}}"
    @echo "{{VERSION}}" > version.txt
    git add version.txt
    git commit -m "Release {{VERSION}}"
    git tag -a "v{{VERSION}}" -m "Release {{VERSION}}"
    @echo "Push with: git push origin main --tags"

# Build release binaries
release-build:
    just cross-all

# ============================================================================
# Deployment
# ============================================================================

# Install systemd service
install-systemd:
    cd deploy/systemd && sudo ./install.sh

# Uninstall systemd service
uninstall-systemd:
    cd deploy/systemd && sudo ./uninstall.sh

# ============================================================================
# Development
# ============================================================================

# Watch and rebuild on changes
watch:
    cargo watch -x 'run -p shortener-server'

# Watch and run tests
watch-test:
    cargo watch -x test

# Generate documentation
doc:
    cargo doc --all --no-deps --open

# Update dependencies
update:
    cargo update

# Audit dependencies for security issues
audit:
    cargo audit

# Install development tools
install-tools:
    cargo install cargo-watch
    cargo install cargo-audit
    cargo install cross --git https://github.com/cross-rs/cross
    cargo install cargo-outdated

# ============================================================================
# Documentation
# ============================================================================

# Serve documentation locally
docs:
    @echo "Installing documentation dependencies..."
    @pip install -q -r docs/requirements.txt
    @echo "Starting documentation server at http://127.0.0.1:8000"
    @mkdocs serve

# Build documentation
docs-build:
    @echo "Installing documentation dependencies..."
    @pip install -q -r docs/requirements.txt
    @echo "Building documentation..."
    @mkdocs build
    @echo "Documentation built to site/"

# Deploy documentation to GitHub Pages
docs-deploy:
    @echo "Installing documentation dependencies..."
    @pip install -q -r docs/requirements.txt
    @echo "Deploying documentation to GitHub Pages..."
    @mkdocs gh-deploy --force

# ============================================================================
# Utilities
# ============================================================================

# Show project statistics
stats:
    @echo "Lines of code:"
    @find . -name '*.rs' -not -path './target/*' | xargs wc -l | tail -1
    @echo ""
    @echo "Number of Rust files:"
    @find . -name '*.rs' -not -path './target/*' | wc -l
    @echo ""
    @echo "Dependencies:"
    @cargo tree --depth 1

# Check for outdated dependencies
outdated:
    cargo outdated

# Show binary sizes
sizes:
    @echo "Binary sizes:"
    @ls -lh target/release/shortener-server 2>/dev/null || echo "Server not built"
    @ls -lh target/release/shortener-cli 2>/dev/null || echo "CLI not built"
