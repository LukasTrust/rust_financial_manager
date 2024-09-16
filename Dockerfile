# Stage 1: Build the Rust application
FROM rust:1-slim-bookworm AS build

# Install necessary dependencies, including PostgreSQL client library and headers
RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    git \
    build-essential \
    curl

# Install Diesel CLI
RUN cargo install diesel_cli --no-default-features --features postgres

# Define build argument with default value
ARG pkg=rust_financial_manager

# Create a working directory
WORKDIR /build

# Clone the specific branch from the GitHub repository
RUN git clone --branch release_test https://github.com/LukasTrust/rust_financial_manager.git .

# Install Rust dependencies
RUN cargo build --release

# Build the application
RUN cargo build --release && \
    objcopy --compress-debug-sections target/release/$pkg ./main

# Stage 2: Create a minimal runtime image
FROM debian:bookworm-slim

# Install necessary runtime libraries, including PostgreSQL client library
RUN apt-get update && \
    apt-get install -y \
    libpq5 \
    openssl \
    curl

# Install Diesel CLI in the runtime image
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    source $HOME/.cargo/env && \
    cargo install diesel_cli --no-default-features --features postgres

# Set working directory
WORKDIR /app

# Copy the binary from the build stage
COPY --from=build /build/main ./

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
