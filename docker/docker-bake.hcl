## Docker Bake Configuration for Shortener
## https://docs.docker.com/build/bake/
## https://docs.docker.com/reference/cli/docker/buildx/bake/
##
## 唯一交付镜像：docker/Dockerfile（nginx + shortener-server 统一镜像）。

## Special target: https://github.com/docker/metadata-action#bake-definition
target "docker-metadata-action" {}

## Common variables
variable "REGISTRY" {
    default = "docker.io"
}

variable "VERSION" {
    default = "latest"
}

## Rust toolchain for the backend builder stage (rust:${RUST_VERSION}-alpine)
variable "RUST_VERSION" {
    default = "1.98"
}

## Shared OCI labels
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
## Shortener unified image (nginx:alpine runtime; nginx serves the frontend
## and proxies /api/* + short codes to the in-container backend at
## 127.0.0.1:8080 — see docker/nginx-aio.conf)
## ============================================================================

variable "IMAGE_NAME" {
    default = "shortener"
}

## Common configuration for all targets
target "_common" {
    inherits = ["docker-metadata-action"]
    labels = oci_labels("Shortener", "Unified URL shortener image with frontend (nginx) and backend (shortener-server) written in Rust")
    context = "."
    dockerfile = "./docker/Dockerfile"
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
