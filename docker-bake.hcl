## Docker Bake Configuration for Shortener Server
## https://docs.docker.com/build/bake/
## https://docs.docker.com/reference/cli/docker/buildx/bake/

## Special target: https://github.com/docker/metadata-action#bake-definition
target "docker-metadata-action" {}

## Common variables
variable "REGISTRY" {
    default = "docker.io"
}

variable "IMAGE_NAME" {
    default = "shortener-server"
}

variable "VERSION" {
    default = "latest"
}

## Base image configuration
target "_image" {
    inherits = ["docker-metadata-action"]
}

## Common configuration for all targets
target "_common" {
    labels = {
        "org.opencontainers.image.title" = "Shortener Server"
        "org.opencontainers.image.description" = "High-performance URL shortener service written in Rust"
        "org.opencontainers.image.source" = "https://github.com/jetsung/shortener"
        "org.opencontainers.image.documentation" = "https://github.com/jetsung/shortener/blob/main/README.md"
        "org.opencontainers.image.authors" = "Jetsung Chan <i@jetsung.com>"
        "org.opencontainers.image.licenses" = "Apache-2.0"
    }
    context = "."
    dockerfile = "./docker/Dockerfile"
    platforms = ["linux/amd64"]
    args = {
        RUST_VERSION = "1.90"
    }
}

## Demo configuration for testing (fast build)
target "_demo" {
    labels = {
        "org.opencontainers.image.title" = "Shortener Server Demo"
        "org.opencontainers.image.description" = "Demo image for testing CI/CD pipeline"
        "org.opencontainers.image.source" = "https://github.com/jetsung/shortener"
        "org.opencontainers.image.authors" = "Jetsung Chan <i@jetsung.com>"
        "org.opencontainers.image.licenses" = "Apache-2.0"
    }
    context = "."
    dockerfile = "./docker/Dockerfile.demo"
    platforms = ["linux/amd64"]
}

## Alpine-based configuration
target "_alpine" {
    inherits = ["_common"]
    dockerfile = "docker/Dockerfile.alpine"
    labels = {
        "org.opencontainers.image.title" = "Shortener Server (Alpine)"
        "org.opencontainers.image.description" = "High-performance URL shortener service written in Rust (Alpine Linux)"
    }
}

## Demo Alpine configuration for testing (fast build)
target "_demo-alpine" {
    inherits = ["_demo"]
    dockerfile = "docker/Dockerfile.demo-alpine"
    labels = {
        "org.opencontainers.image.title" = "Shortener Server Demo (Alpine)"
        "org.opencontainers.image.description" = "Demo Alpine image for testing CI/CD pipeline"
    }
}

## Default target for local development
target "default" {
    inherits = ["_common"]
    tags = [
        "shortener-server:local",
        "shortener-server:dev"
    ]
    output = ["type=docker"]
}

## Alpine default target
target "default-alpine" {
    inherits = ["_alpine"]
    tags = [
        "shortener-server:local-alpine",
        "shortener-server:dev-alpine"
    ]
    output = ["type=docker"]
}

## Development builds group
group "dev" {
    targets = ["dev-amd64", "dev-arm64"]
}

## Development build (all platforms)
target "dev" {
    inherits = ["_common", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:dev",
        "${REGISTRY}/${IMAGE_NAME}:dev-${VERSION}"
    ]
}

## Development build (amd64)
target "dev-amd64" {
    inherits = ["_common", "_image"]
    platforms = ["linux/amd64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:dev-amd64",
        "${REGISTRY}/${IMAGE_NAME}:dev-amd64-${VERSION}"
    ]
}

## Development build (arm64)
target "dev-arm64" {
    inherits = ["_common", "_image"]
    platforms = ["linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:dev-arm64",
        "${REGISTRY}/${IMAGE_NAME}:dev-arm64-${VERSION}"
    ]
}

## Alpine development builds group
group "dev-alpine" {
    targets = ["dev-alpine-amd64", "dev-alpine-arm64"]
}

## Alpine development build (all platforms)
target "dev-alpine" {
    inherits = ["_alpine", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:dev-alpine",
        "${REGISTRY}/${IMAGE_NAME}:dev-alpine-${VERSION}"
    ]
}

## Alpine development build (amd64)
target "dev-alpine-amd64" {
    inherits = ["_alpine", "_image"]
    platforms = ["linux/amd64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:dev-alpine-amd64",
        "${REGISTRY}/${IMAGE_NAME}:dev-alpine-amd64-${VERSION}"
    ]
}

## Alpine development build (arm64)
target "dev-alpine-arm64" {
    inherits = ["_alpine", "_image"]
    platforms = ["linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:dev-alpine-arm64",
        "${REGISTRY}/${IMAGE_NAME}:dev-alpine-arm64-${VERSION}"
    ]
}

## Release builds group (for CI/CD - builds both debian and alpine)
group "release" {
    targets = ["release-debian", "release-alpine"]
}

## CI Release build - Debian-based (main/latest tag)
target "release" {
    inherits = ["_common", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:latest"
    ]
}

## CI Release build - Alpine variant
target "release-alpine" {
    inherits = ["_alpine", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:alpine"
    ]
}

## Release build (Debian-based, multi-platform) - with debian tags
target "release-debian" {
    inherits = ["_common", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:latest",
        "${REGISTRY}/${IMAGE_NAME}:${VERSION}",
        "${REGISTRY}/${IMAGE_NAME}:debian",
        "${REGISTRY}/${IMAGE_NAME}:debian-${VERSION}"
    ]
}

## Release build (Debian, amd64 only)
target "release-debian-amd64" {
    inherits = ["_common", "_image"]
    platforms = ["linux/amd64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:${VERSION}-amd64",
        "${REGISTRY}/${IMAGE_NAME}:debian-${VERSION}-amd64"
    ]
}

## Release build (Debian, arm64 only)
target "release-debian-arm64" {
    inherits = ["_common", "_image"]
    platforms = ["linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:${VERSION}-arm64",
        "${REGISTRY}/${IMAGE_NAME}:debian-${VERSION}-arm64"
    ]
}

## Release build (Alpine, amd64 only)
target "release-alpine-amd64" {
    inherits = ["_alpine", "_image"]
    platforms = ["linux/amd64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:alpine-${VERSION}-amd64"
    ]
}

## Release build (Alpine, arm64 only)
target "release-alpine-arm64" {
    inherits = ["_alpine", "_image"]
    platforms = ["linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:alpine-${VERSION}-arm64"
    ]
}

## All builds group (for CI/CD)
group "all" {
    targets = ["release-debian", "release-alpine"]
}

## Demo release targets (for testing CI/CD)
target "release-demo" {
    inherits = ["_demo", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:latest"
    ]
}

target "release-demo-alpine" {
    inherits = ["_demo-alpine", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:alpine"
    ]
}

## ============================================================================
## Frontend Configuration
## ============================================================================

variable "FRONTEND_IMAGE_NAME" {
    default = "shortener-frontend"
}

## Common configuration for frontend targets
target "_frontend_common" {
    labels = {
        "org.opencontainers.image.title" = "Shortener Frontend"
        "org.opencontainers.image.description" = "Modern URL shortener service frontend"
        "org.opencontainers.image.source" = "https://github.com/jetsung/shortener"
        "org.opencontainers.image.documentation" = "https://github.com/jetsung/shortener/blob/main/README.md"
        "org.opencontainers.image.authors" = "Jetsung Chan <i@jetsung.com>"
        "org.opencontainers.image.licenses" = "Apache-2.0"
    }
    context = "."
    dockerfile = "./docker/Dockerfile.frontend"
    platforms = ["linux/amd64"]
}

## Default target for local development
target "frontend-default" {
    inherits = ["_frontend_common"]
    tags = [
        "shortener-frontend:local",
        "shortener-frontend:dev"
    ]
    output = ["type=docker"]
}

## Development builds group
group "frontend-dev" {
    targets = ["frontend-dev-amd64", "frontend-dev-arm64"]
}

## Development build (all platforms)
target "frontend-dev" {
    inherits = ["_frontend_common", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:dev",
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:dev-${VERSION}"
    ]
}

## Development build (amd64)
target "frontend-dev-amd64" {
    inherits = ["_frontend_common", "_image"]
    platforms = ["linux/amd64"]
    tags = [
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:dev-amd64",
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:dev-amd64-${VERSION}"
    ]
}

## Development build (arm64)
target "frontend-dev-arm64" {
    inherits = ["_frontend_common", "_image"]
    platforms = ["linux/arm64"]
    tags = [
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:dev-arm64",
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:dev-arm64-${VERSION}"
    ]
}

## CI Release build (main/latest tag)
target "frontend-release" {
    inherits = ["_frontend_common", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:latest",
    ]
}

## Release build (amd64 only)
target "frontend-release-amd64" {
    inherits = ["_frontend_common", "_image"]
    platforms = ["linux/amd64"]
    tags = [
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:${VERSION}-amd64"
    ]
}

## Release build (arm64 only)
target "frontend-release-arm64" {
    inherits = ["_frontend_common", "_image"]
    platforms = ["linux/arm64"]
    tags = [
        "${REGISTRY}/${FRONTEND_IMAGE_NAME}:${VERSION}-arm64"
    ]
}
