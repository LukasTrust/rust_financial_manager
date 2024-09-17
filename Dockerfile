# Stage 1: Build the Rust application
FROM rust:1-slim-bookworm AS build

# Install necessary dependencies, including PostgreSQL client library, Diesel CLI, and headers
RUN apt-get update && \
    apt-get install -y \
    git \
    build-essential \
    libpq-dev \
    binutils \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Diesel CLI
RUN cargo install diesel_cli --no-default-features --features postgres

# Define build argument with default value
ARG pkg=rust_financial_manager

# Create a working directory
WORKDIR /build

# Clone the specific branch from the GitHub repository
RUN git clone --branch release_test https://github.com/LukasTrust/rust_financial_manager.git .

# Copy the migrations directory
COPY ./migrations /build/migrations

# Copy the .env file into the runtime container
COPY .env /app/.env

# Build the application
RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/$pkg ./main

# Stage 2: Create a minimal runtime image
FROM debian:bookworm-slim

# Install necessary runtime libraries, including PostgreSQL client library and openssl
RUN apt-get update && \
    apt-get install -y \
    libpq5 \
    postgresql-client \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the binary from the build stage
COPY --from=build /build/main ./

# Copy the Diesel CLI binary from the build stage
COPY --from=build /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# Copy the migrations directory from the build stage
COPY --from=build /build/migrations ./migrations

# Conditionally copy files if they exist
COPY --from=build /build/Rocket.toml ./static/
COPY --from=build /build/static ./static/
COPY --from=build /build/templates ./templates/

# Copy the entry point script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set environment variables for Rocket
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080

# Define the entry point
ENTRYPOINT ["/entrypoint.sh"]
CMD ["./main"]
