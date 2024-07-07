# Use Rust version 1.79.0, latest on my machine as of 07.07.24
FROM lukemathwalker/cargo-chef:latest-rust-1.79.0 as chef
# `cd` into our app - this folder will be created by Docker if it doesn't already exist
WORKDIR /app
# Install required system dependencies for our linking config
RUN apt update && apt install lld clang -y

FROM chef as planner
# Copy all files from our working environment to our Docker image
COPY . .
# Computes a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same, all layers should be cached
COPY . .
# Set SQLX to offline mode to read from prepared metadata
ENV SQLX_OFFLINE true
# Build the project
RUN cargo build --release --bin zero2prod

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates when establishing HTTPS connection
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Copy the compiled binary from the builder environment to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need to copy configuration file at runtime
COPY configuration configuration
# Set environment to production
ENV APP_ENVIRONMENT production
# When `docker run` executes, launch the binary
ENTRYPOINT ["./zero2prod"]
