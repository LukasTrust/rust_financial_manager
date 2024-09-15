# Stage 1: Build and install tools
FROM debian:buster-slim as builder

# Install build dependencies, including curl and ca-certificates
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates libpq-dev build-essential \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install Rust (required for Diesel CLI)
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add Rust to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Diesel CLI with PostgreSQL support
RUN cargo install diesel_cli --no-default-features --features postgres

# Stage 2: Final image
FROM debian:buster-slim

# Install runtime dependencies, including curl, ca-certificates, file, and others
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates libpq5 openssl postgresql-client file \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy Diesel CLI binary from the builder stage
COPY --from=builder /root/.cargo/bin/diesel /usr/local/bin/diesel

# Use hardcoded release tag to download the binary
RUN TAG="v0.1.0" && \
    RELEASE_URL="https://github.com/LukasTrust/rust_financial_manager/releases/download/${TAG}/rust_financial_manager" && \
    echo "Using release URL: ${RELEASE_URL}" && \
    curl -L ${RELEASE_URL} -o /usr/local/bin/rust_financial_manager && \
    chmod +x /usr/local/bin/rust_financial_manager && \
    echo "Binary downloaded and permissions set."

# Verify binary download and permissions
RUN echo "Checking binary..." && \
    ls -l /usr/local/bin/rust_financial_manager && \
    file /usr/local/bin/rust_financial_manager && \
    if [ -x /usr/local/bin/rust_financial_manager ]; then \
    echo "Binary is executable"; \
    else \
    echo "Binary is not executable"; \
    exit 1; \
    fi

# Copy the migrations directory into the container
COPY ./migrations /usr/src/migrations

# Add this to copy the static directory into the container
COPY ./static /usr/src/static

# Add a non-root user
RUN useradd -ms /bin/sh appuser
USER appuser

# Set working directory
WORKDIR /usr/src

# Default command
CMD ["sh", "-c", "ROCKET_SECRET_KEY=$(openssl rand -base64 32) /usr/local/bin/rust_financial_manager"]
