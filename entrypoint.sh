#!/bin/sh
# Generate a secret key if not already set
if [ -z "$ROCKET_SECRET_KEY" ]; then
  export ROCKET_SECRET_KEY=$(openssl rand -base64 32)
fi

# Load the .env file from /app directory to set environment variables
if [ -f /app/.env ]; then
  set -o allexport
  . /app/.env
  set +o allexport
fi

echo "POSTGRES_USER: $POSTGRES_USER"
echo "POSTGRES_PASSWORD: $POSTGRES_PASSWORD"
echo "POSTGRES_DB: $POSTGRES_DB"

# Build the DATABASE_URL dynamically
DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres/${POSTGRES_DB}"

# Output the DATABASE_URL for debugging
echo "DATABASE_URL is: $DATABASE_URL"

# Run Diesel migrations using DATABASE_URL
diesel migration run --database-url $DATABASE_URL

# Start the Rust application
exec "$@"
