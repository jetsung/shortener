## Docker Bake Configuration for Shortener
## https://docs.docker.com/build/bake/
## https://docs.docker.com/reference/cli/docker/buildx/bake/

## Special target: https://github.com/docker/metadata-action#bake-definition
target "docker-metadata-action" {}

## Common variables
variable "REGISTRY" {
    default = "docker.io"
}

variable "VERSION" {
    default = "latest"
}

## Rust toolchain for all backend builder stages (rust:${RUST_VERSION}-alpine)
variable "RUST_VERSION" {
    default = "1.98"
}

## Shared OCI labels; only title/description differ per image
function "oci_labels" {
    params = [title, description]
    result = {
        "org.opencontainers.image.title" = title
        "org.opencontainers.image.description" = description
        "org.opencontainers.image.source" = "https://github.com/jetsung/shortener"
        "org.opencontainers.image.documentation" = "https://github.com/jetsung/shortener/blob/main/README.md"
        "org.opencontainers.image.authors" = "Jetsung Chan <i@jetsung.com>"
        "org.opencontainers.image.licenses" = "Apache-2.0"
    }
}

## Release tags: :latest (+ :${VERSION} unless it would duplicate :latest)
function "release_tags" {
    params = [image]
    result = VERSION == "latest" ? ["${REGISTRY}/${image}:latest"] : ["${REGISTRY}/${image}:latest", "${REGISTRY}/${image}:${VERSION}"]
}

## Dev tags: :dev + :dev-${VERSION}
function "dev_tags" {
    params = [image]
    result = ["${REGISTRY}/${image}:dev", "${REGISTRY}/${image}:dev-${VERSION}"]
}

## Per-arch release tag: :${VERSION}-${arch}
function "arch_tags" {
    params = [image, arch]
    result = ["${REGISTRY}/${image}:${VERSION}-${arch}"]
}

## Per-arch dev tags: :dev-${arch} + :dev-${arch}-${VERSION}
function "dev_arch_tags" {
    params = [image, arch]
    result = ["${REGISTRY}/${image}:dev-${arch}", "${REGISTRY}/${image}:dev-${arch}-${VERSION}"]
}

## ============================================================================
## Backend (distroless: built on rust:alpine, runtime is scratch)
## ============================================================================

variable "IMAGE_NAME" {
    default = "shortener-server"
}

## Common configuration for all backend targets
target "_common" {
    inherits = ["docker-metadata-action"]
    labels = oci_labels("Shortener Server", "High-performance URL shortener service written in Rust")
    context = "."
    dockerfile = "./docker/Dockerfile.backend"
    platforms = ["linux/amd64"]
    args = {
        RUST_VERSION = "${RUST_VERSION}"
    }
}

## Default target for local development
target "default" {
    inherits = ["_common"]
    tags = [
        "${IMAGE_NAME}:local",
        "${IMAGE_NAME}:dev"
    ]
    output = ["type=docker"]
}

## Development builds group
group "dev" {
    targets = ["dev-amd64", "dev-arm64"]
}

## Development build (all platforms)
target "dev" {
    inherits = ["_common"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = dev_tags(IMAGE_NAME)
}

## Development build (amd64)
target "dev-amd64" {
    inherits = ["_common"]
    platforms = ["linux/amd64"]
    tags = dev_arch_tags(IMAGE_NAME, "amd64")
}

## Development build (arm64)
target "dev-arm64" {
    inherits = ["_common"]
    platforms = ["linux/arm64"]
    tags = dev_arch_tags(IMAGE_NAME, "arm64")
}

## Release builds group (for CI/CD)
group "release-all" {
    targets = ["release"]
}

## Release build (multi-platform)
target "release" {
    inherits = ["_common"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = release_tags(IMAGE_NAME)
}

## Release build (amd64 only)
target "release-amd64" {
    inherits = ["_common"]
    platforms = ["linux/amd64"]
    tags = arch_tags(IMAGE_NAME, "amd64")
}

## Release build (arm64 only)
target "release-arm64" {
    inherits = ["_common"]
    platforms = ["linux/arm64"]
    tags = arch_tags(IMAGE_NAME, "arm64")
}

## ============================================================================
## Frontend (built on node:alpine, runtime is nginx:alpine)
## ============================================================================

variable "FRONTEND_IMAGE_NAME" {
    default = "shortener-frontend"
}

## Common configuration for frontend targets
target "_frontend_common" {
    inherits = ["docker-metadata-action"]
    labels = oci_labels("Shortener Frontend", "Modern URL shortener service frontend")
    context = "."
    dockerfile = "./docker/Dockerfile.frontend"
    platforms = ["linux/amd64"]
}

## Default target for local development
target "frontend-default" {
    inherits = ["_frontend_common"]
    tags = [
        "${FRONTEND_IMAGE_NAME}:local",
        "${FRONTEND_IMAGE_NAME}:dev"
    ]
    output = ["type=docker"]
}

## Development builds group
group "frontend-dev" {
    targets = ["frontend-dev-amd64", "frontend-dev-arm64"]
}

## Development build (all platforms)
target "frontend-dev" {
    inherits = ["_frontend_common"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = dev_tags(FRONTEND_IMAGE_NAME)
}

## Development build (amd64)
target "frontend-dev-amd64" {
    inherits = ["_frontend_common"]
    platforms = ["linux/amd64"]
    tags = dev_arch_tags(FRONTEND_IMAGE_NAME, "amd64")
}

## Development build (arm64)
target "frontend-dev-arm64" {
    inherits = ["_frontend_common"]
    platforms = ["linux/arm64"]
    tags = dev_arch_tags(FRONTEND_IMAGE_NAME, "arm64")
}

## Release build (multi-platform)
target "frontend-release" {
    inherits = ["_frontend_common"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = release_tags(FRONTEND_IMAGE_NAME)
}

## Release build (amd64 only)
target "frontend-release-amd64" {
    inherits = ["_frontend_common"]
    platforms = ["linux/amd64"]
    tags = arch_tags(FRONTEND_IMAGE_NAME, "amd64")
}

## Release build (arm64 only)
target "frontend-release-arm64" {
    inherits = ["_frontend_common"]
    platforms = ["linux/arm64"]
    tags = arch_tags(FRONTEND_IMAGE_NAME, "arm64")
}

## ============================================================================
## All-In-One (frontend + backend in a single image)
## nginx (port 80) serves the frontend and proxies /api/* + short codes to
## the in-container backend at 127.0.0.1:8080 (see docker/nginx-aio.conf)
## ============================================================================

variable "AIO_IMAGE_NAME" {
    default = "shortener"
}

## Common configuration for AIO targets
target "_aio_common" {
    inherits = ["docker-metadata-action"]
    labels = oci_labels("Shortener All-In-One", "All-in-one image with frontend and backend for URL shortener")
    context = "."
    dockerfile = "./docker/Dockerfile"
    platforms = ["linux/amd64"]
    args = {
        RUST_VERSION = "${RUST_VERSION}"
    }
}

## Default target for local development
target "aio-default" {
    inherits = ["_aio_common"]
    tags = [
        "${AIO_IMAGE_NAME}:local",
        "${AIO_IMAGE_NAME}:dev"
    ]
    output = ["type=docker"]
}

## Development builds group
group "aio-dev" {
    targets = ["aio-dev-amd64", "aio-dev-arm64"]
}

## Development build (all platforms)
target "aio-dev" {
    inherits = ["_aio_common"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = dev_tags(AIO_IMAGE_NAME)
}

## Development build (amd64)
target "aio-dev-amd64" {
    inherits = ["_aio_common"]
    platforms = ["linux/amd64"]
    tags = dev_arch_tags(AIO_IMAGE_NAME, "amd64")
}

## Development build (arm64)
target "aio-dev-arm64" {
    inherits = ["_aio_common"]
    platforms = ["linux/arm64"]
    tags = dev_arch_tags(AIO_IMAGE_NAME, "arm64")
}

## Release builds group (for CI/CD)
group "aio-release-all" {
    targets = ["aio-release"]
}

## Release build (multi-platform)
target "aio-release" {
    inherits = ["_aio_common"]
    platforms = ["linux/amd64", "linux/arm64"]
    tags = release_tags(AIO_IMAGE_NAME)
}

## Release build (amd64 only)
target "aio-release-amd64" {
    inherits = ["_aio_common"]
    platforms = ["linux/amd64"]
    tags = arch_tags(AIO_IMAGE_NAME, "amd64")
}

## Release build (arm64 only)
target "aio-release-arm64" {
    inherits = ["_aio_common"]
    platforms = ["linux/arm64"]
    tags = arch_tags(AIO_IMAGE_NAME, "arm64")
}
