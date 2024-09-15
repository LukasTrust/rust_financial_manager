# Stage 1: Download the latest release binary
FROM debian:buster-slim AS downloader

# Install dependencies for downloading
RUN apt-get update && apt-get install -y curl ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Set GitHub release information
ENV GITHUB_REPO="your_username/rust_financial_manager"
ENV RELEASE_URL="https://github.com/$GITHUB_REPO/releases/latest/download/rust_financial_manager"

# Download the binary from the latest release
RUN curl -L $RELEASE_URL -o /usr/local/bin/rust_financial_manager && \
    chmod +x /usr/local/bin/rust_financial_manager

# Stage 2: Final Image
FROM debian:buster-slim

# Install dependencies: PostgreSQL client, OpenSSL, and CA-certificates
RUN apt-get update && apt-get install -y \
    libpq-dev openssl ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy the downloaded binary
COPY --from=downloader /usr/local/bin/rust_financial_manager /usr/local/bin/rust_financial_manager

# Copy Diesel migrations
COPY ./migrations /usr/src/app/migrations

# Create the .env file directly in the Docker image
RUN echo "DATABASE_URL=postgres://postgres:your_password@localhost/financial_manager\nROCKET_PORT=8000" > /usr/src/app/.env

# Set environment variables
ENV ROCKET_ENV=release

# Command to start the app without diesel migration run
CMD source /usr/src/app/.env && \
    ROCKET_SECRET_KEY=$(openssl rand -base64 32) ROCKET_PORT=${ROCKET_PORT:-8000} /usr/local/bin/rust_financial_manager
