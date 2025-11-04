# Justfile for shortener project
# https://github.com/casey/just

# Default recipe to display help
default:
    @just --list

# ============================================================================
# Build
# ============================================================================

alias b := build

# Build all packages (backend + frontend)
build: build-backend build-frontend

# Build backend packages
build-backend:
    cargo build --release

# Build server only
build-server:
    cargo build --release -p shortener-server

# Build CLI only
build-cli:
    cargo build --release -p shortener-cli

# Build frontend for production
build-frontend:
    cd shortener-frontend && pnpm install && pnpm build

# Build frontend with bundle analysis
build-frontend-analyze:
    cd shortener-frontend && pnpm install && pnpm build:analyze

# Clean build artifacts
clean: clean-backend clean-frontend

# Clean backend artifacts
clean-backend:
    cargo clean

# Clean frontend artifacts
clean-frontend:
    cd shortener-frontend && pnpm clean

# ============================================================================
# Run
# ============================================================================

# Run server
run:
    cargo run -p shortener-server

# Run CLI
run-cli *ARGS:
    cargo run -p shortener-cli -- {{ARGS}}

# Run frontend development server
run-frontend:
    cd shortener-frontend && pnpm dev

# Preview frontend production build
preview-frontend:
    cd shortener-frontend && pnpm preview

# ============================================================================
# Test
# ============================================================================

alias t := test

# Run all tests (backend + frontend)
test: test-backend test-frontend

# Run backend tests
test-backend:
    cargo test --all

# Run backend tests with output
test-verbose:
    cargo test --all -- --nocapture

# Run benchmarks
bench:
    cargo bench --all

# Run frontend tests
test-frontend:
    cd shortener-frontend && pnpm test

# Run frontend tests in watch mode
test-frontend-watch:
    cd shortener-frontend && pnpm test:watch

# Run frontend tests with coverage
test-frontend-coverage:
    cd shortener-frontend && pnpm test:coverage

# Run frontend tests with UI
test-frontend-ui:
    cd shortener-frontend && pnpm test:ui

# ============================================================================
# Code Quality
# ============================================================================

# Format all code (backend + frontend)
fmt: fmt-backend fmt-frontend

# Format backend code
fmt-backend:
    cargo fmt --all

# Format frontend code
fmt-frontend:
    cd shortener-frontend && pnpm prettier

# Check formatting for all code
fmt-check: fmt-check-backend fmt-check-frontend

# Check backend formatting
fmt-check-backend:
    cargo fmt --all -- --check

# Check frontend formatting
fmt-check-frontend:
    cd shortener-frontend && pnpm prettier:check

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Lint frontend code
lint-frontend:
    cd shortener-frontend && pnpm lint

# Fix frontend linting issues
lint-frontend-fix:
    cd shortener-frontend && pnpm lint:fix

# Type check frontend
type-check-frontend:
    cd shortener-frontend && pnpm type-check

# Run all checks (backend + frontend)
check: fmt-check clippy test type-check-frontend lint-frontend

# Run backend checks only
check-backend: fmt-check-backend clippy test-backend

# Run frontend checks only
check-frontend: fmt-check-frontend lint-frontend type-check-frontend test-frontend

# Run CI checks for frontend
ci-frontend:
    cd shortener-frontend && pnpm ci

# ============================================================================
# Docker
# ============================================================================

# Build Docker image (Debian)
docker-build:
    docker build -f docker/Dockerfile -t shortener-server:latest .

# Build Docker image (Alpine)
docker-build-alpine:
    docker build -f docker/Dockerfile.alpine -t shortener-server:alpine .

# Build frontend Docker image
docker-build-frontend:
    docker build -f docker/Dockerfile.frontend -t shortener-frontend:latest .

# Run with docker compose
docker-run:
    docker compose -f docker/docker-compose.yml up -d

# Run development environment
docker-run-dev:
    docker compose -f docker/docker-compose.dev.yml up -d

# Run frontend with docker compose
docker-run-frontend:
    docker compose -f docker/docker-compose.frontend.yml up -d

# Stop Docker containers
docker-stop:
    docker compose -f docker/docker-compose.yml down
    docker compose -f docker/docker-compose.dev.yml down
    docker compose -f docker/docker-compose.frontend.yml down

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
update: update-backend update-frontend

# Update backend dependencies
update-backend:
    cargo update

# Update frontend dependencies
update-frontend:
    cd shortener-frontend && pnpm update

# Audit dependencies for security issues
audit: audit-backend

# Audit backend dependencies
audit-backend:
    cargo audit

# Install frontend dependencies
install-frontend:
    cd shortener-frontend && pnpm install

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
    @echo "=== Backend Statistics ==="
    @echo "Lines of Rust code:"
    @find . -name '*.rs' -not -path './target/*' | xargs wc -l | tail -1
    @echo ""
    @echo "Number of Rust files:"
    @find . -name '*.rs' -not -path './target/*' | wc -l
    @echo ""
    @echo "Backend dependencies:"
    @cargo tree --depth 1
    @echo ""
    @echo "=== Frontend Statistics ==="
    @echo "Lines of TypeScript/TSX code:"
    @find shortener-frontend/src -name '*.ts' -o -name '*.tsx' | xargs wc -l | tail -1 || echo "N/A"
    @echo ""
    @echo "Number of TypeScript/TSX files:"
    @find shortener-frontend/src -name '*.ts' -o -name '*.tsx' | wc -l || echo "N/A"

# Check for outdated dependencies
outdated: outdated-backend outdated-frontend

# Check for outdated backend dependencies
outdated-backend:
    cargo outdated

# Check for outdated frontend dependencies
outdated-frontend:
    cd shortener-frontend && pnpm outdated

# Show binary sizes
sizes:
    @echo "=== Backend Binary Sizes ==="
    @ls -lh target/release/shortener-server 2>/dev/null || echo "Server not built"
    @ls -lh target/release/shortener-cli 2>/dev/null || echo "CLI not built"
    @echo ""
    @echo "=== Frontend Build Size ==="
    @du -sh shortener-frontend/dist 2>/dev/null || echo "Frontend not built"
