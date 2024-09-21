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

# Output the DATABASE_URL for debugging
echo "DATABASE_URL is: $DB_URL"

echo "Running database migrations..."
diesel migration run --database-url $DB_URL

# Start the Rust application
exec "$@"