# Multi-stage build for shortener-server (Distroless-based for minimal image size)
# Stage 1: Build the application
FROM rust:1.90-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY shortener-server/Cargo.toml ./shortener-server/
COPY shortener-cli/Cargo.toml ./shortener-cli/
COPY shortener-common/Cargo.toml ./shortener-common/

# Copy source code
COPY shortener-server/src ./shortener-server/src
COPY shortener-cli/src ./shortener-cli/src
COPY shortener-common/src ./shortener-common/src

# Copy configuration file (needed at build time for include_str! macro)
RUN mkdir -p ./config
COPY config/config.toml ./config/config.toml

# Copy benches (required for Cargo.toml validation, but won't be compiled)
COPY shortener-server/benches ./shortener-server/benches

# Build for release (skip benches and tests)
RUN cargo build --release -p shortener-server --bins

# Stage 2: Create the runtime image using distroless
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy the binary from builder
COPY --from=builder /app/target/release/shortener-server /usr/local/bin/shortener-server

# Copy configuration file
COPY config/config.docker.toml /app/config.toml

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config.toml

# Run the binary
CMD ["/usr/local/bin/shortener-server"]
