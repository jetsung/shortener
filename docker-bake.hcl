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
        "org.opencontainers.image.source" = "https://github.com/yourusername/shortener"
        "org.opencontainers.image.documentation" = "https://github.com/yourusername/shortener/blob/main/README.md"
        "org.opencontainers.image.authors" = "Your Name <your@email.com>"
        "org.opencontainers.image.licenses" = "MIT"
    }
    context = "."
    dockerfile = "docker/Dockerfile"
    platforms = ["linux/amd64"]
    args = {
        RUST_VERSION = "1.90"
    }
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

## Release builds group
group "release" {
    targets = ["release-debian", "release-alpine"]
}

## Release build (Debian-based, multi-platform)
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

## Release build (Alpine-based, multi-platform)
target "release-alpine" {
    inherits = ["_alpine", "_image"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = [
        "${REGISTRY}/${IMAGE_NAME}:alpine",
        "${REGISTRY}/${IMAGE_NAME}:alpine-${VERSION}"
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
