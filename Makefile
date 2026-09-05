.PHONY: help build build-aio run run-dev stop clean logs test docker-push cross-build cross-build-server cross-build-cli docs docs-serve docs-build version bump-version tag-server tag-frontend

# Default target
help:
	@echo "Available targets:"
	@echo ""
	@echo "Docker targets:"
	@echo "  build         - Build Docker image (Debian-based)"
	@echo "  build-aio     - Build All-In-One Docker image (frontend + backend)"
	@echo "  run           - Run with docker-compose (PostgreSQL + Redis)"
	@echo "  run-dev       - Run with docker-compose.dev (SQLite + Redis)"
	@echo "  run-mysql     - Run with docker-compose (MySQL + Redis)"
	@echo "  stop          - Stop all containers"
	@echo "  clean         - Stop and remove all containers and volumes"
	@echo "  logs          - Show logs from all containers"
	@echo "  logs-server   - Show logs from server only"
	@echo "  test          - Run tests in Docker"
	@echo "  docker-push   - Push image to registry"
	@echo ""
	@echo "Cross-compilation targets:"
	@echo "  cross-build        - Build all packages for all targets"
	@echo "  cross-build-server - Build server for all targets"
	@echo "  cross-build-cli    - Build CLI for all targets"
	@echo ""
	@echo "Documentation targets:"
	@echo "  docs           - Install docs dependencies and serve locally"
	@echo "  docs-serve     - Serve documentation locally"
	@echo "  docs-build     - Build documentation to site/"
	@echo ""
	@echo "Version targets:"
	@echo "  version        - Show versions in Cargo.toml / openapi.yml / package.json"
	@echo "  bump-version   - Sync version across them (usage: make bump-version VERSION=$(VERSION))"
	@echo ""
	@echo "Git targets:"
	@echo "  tag-server     - Create signed git tag for server (usage: make tag-server VERSION=$(VERSION))"
	@echo "  tag-frontend   - Create signed git tag for frontend (usage: make tag-frontend VERSION=$(VERSION))"

# Build Docker image (Debian-based)
build:
	docker build -f docker/Dockerfile.backend -t shortener-server:latest .

# Build All-In-One Docker image (frontend + backend)
build-aio:
	docker build -f docker/Dockerfile -t shortener:latest .

# Run with PostgreSQL and Redis
run:
	docker compose -f docker/docker-compose.yml up -d

# Run development environment with SQLite
run-dev:
	docker compose -f docker/docker-compose.dev.yml up -d

# Run with MySQL instead of PostgreSQL
run-mysql:
	docker compose -f docker/docker-compose.yml --profile mysql up -d

# Stop all containers
stop:
	docker compose -f docker/docker-compose.yml down
	docker compose -f docker/docker-compose.dev.yml down

# Clean up everything
clean:
	docker compose -f docker/docker-compose.yml down -v
	docker compose -f docker/docker-compose.dev.yml down -v
	docker system prune -f

# Show logs
logs:
	docker compose -f docker/docker-compose.yml logs -f

# Show server logs only
logs-server:
	docker compose -f docker/docker-compose.yml logs -f shortener-server

# Run tests in Docker
test:
	docker build -f docker/Dockerfile.backend --target builder -t shortener-test .
	docker run --rm shortener-test cargo test

# Push to registry (customize REGISTRY variable; defaults to match CI ghcr.io)
REGISTRY ?= ghcr.io/jetsung
TAG ?= latest

docker-push: build
	docker tag shortener-server:latest $(REGISTRY)/shortener-server:$(TAG)
	docker push $(REGISTRY)/shortener-server:$(TAG)

# Cross-compilation targets
cross-build:
	./scripts/build-cross.sh --all

cross-build-server:
	./scripts/build-cross.sh --server

cross-build-cli:
	./scripts/build-cross.sh --cli

# Documentation targets
docs: docs-serve

docs-serve:
	@command -v zensical >/dev/null 2>&1 || { command -v uv >/dev/null 2>&1 && uv tool install -q zensical || pip install -q zensical; }
	@echo "Starting documentation server at http://127.0.0.1:8000"
	@zensical serve

docs-build:
	@command -v zensical >/dev/null 2>&1 || { command -v uv >/dev/null 2>&1 && uv tool install -q zensical || pip install -q zensical; }
	@echo "Building documentation..."
	@zensical build --clean
	@echo "Documentation built to site/"

# Version targets
# 默认取 Cargo.toml 的版本，可用 make <target> VERSION=x.y.z 覆盖
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

# Show versions across Cargo.toml / openapi.yml / shortener-frontend/package.json
version:
	./scripts/bump-version.sh

# Sync version across Cargo.toml / openapi.yml / shortener-frontend/package.json
bump-version:
	./scripts/bump-version.sh $(VERSION)

# Git tag targets
tag-server:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make tag-server VERSION=0.2.0"; \
		exit 1; \
	fi
	@TAG_NAME="shortener-server-v$(VERSION)"; \
	echo "Creating signed tag: $$TAG_NAME"; \
	git tag -s $$TAG_NAME -m "Release $$TAG_NAME"; \
	echo "Tag created successfully. Push with: git push origin $$TAG_NAME"

tag-frontend:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make tag-frontend VERSION=0.2.0"; \
		exit 1; \
	fi
	@TAG_NAME="shortener-frontend-v$(VERSION)"; \
	echo "Creating signed tag: $$TAG_NAME"; \
	git tag -s $$TAG_NAME -m "Release $$TAG_NAME"; \
	echo "Tag created successfully. Push with: git push origin $$TAG_NAME"
