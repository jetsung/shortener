.PHONY: help build build-alpine run run-dev stop clean logs test docker-push cross-build cross-build-server cross-build-cli docs docs-serve docs-build tag-server tag-frontend

# Default target
help:
	@echo "Available targets:"
	@echo ""
	@echo "Docker targets:"
	@echo "  build         - Build Docker image (Debian-based)"
	@echo "  build-alpine  - Build Docker image (Alpine-based, smaller)"
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
	@echo "Git targets:"
	@echo "  tag-server     - Create signed git tag for server (usage: make tag-server VERSION=0.1.2)"
	@echo "  tag-frontend   - Create signed git tag for frontend (usage: make tag-frontend VERSION=0.1.2)"

# Build Docker image (Debian-based)
build:
	docker build -f docker/Dockerfile -t shortener-server:latest .

# Build Docker image (Alpine-based)
build-alpine:
	docker build -f docker/Dockerfile.alpine -t shortener-server:alpine .

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
	docker build -f docker/Dockerfile --target builder -t shortener-test .
	docker run --rm shortener-test cargo test

# Push to registry (customize REGISTRY variable)
REGISTRY ?= docker.io/yourusername
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
	@echo "Installing documentation dependencies..."
	@pip install -q -r docs/requirements.txt
	@echo "Starting documentation server at http://127.0.0.1:8000"
	@mkdocs serve

docs-build:
	@echo "Installing documentation dependencies..."
	@pip install -q -r docs/requirements.txt
	@echo "Building documentation..."
	@mkdocs build
	@echo "Documentation built to site/"

# Git tag targets
VERSION ?= 0.1.0

tag-server:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make tag-server VERSION=0.1.2"; \
		exit 1; \
	fi
	@TAG_NAME="shortener-server-v$(VERSION)"; \
	echo "Creating signed tag: $$TAG_NAME"; \
	git tag -s $$TAG_NAME -m "Release $$TAG_NAME"; \
	echo "Tag created successfully. Push with: git push origin $$TAG_NAME"

tag-frontend:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION is required. Usage: make tag-frontend VERSION=0.1.2"; \
		exit 1; \
	fi
	@TAG_NAME="shortener-frontend-v$(VERSION)"; \
	echo "Creating signed tag: $$TAG_NAME"; \
	git tag -s $$TAG_NAME -m "Release $$TAG_NAME"; \
	echo "Tag created successfully. Push with: git push origin $$TAG_NAME"
