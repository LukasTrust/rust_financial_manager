#!/bin/sh
set -e

# Generate a secret key if not already set
if [ -z "$ROCKET_SECRET_KEY" ]; then
  export ROCKET_SECRET_KEY=$(openssl rand -base64 32)
fi

# Ensure DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
  echo "DATABASE_URL is not set. Exiting."
  exit 1
fi

# Wait for PostgreSQL to be ready
until diesel migration run --database-url="$DATABASE_URL"; do
  >&2 echo "Database is unavailable - sleeping"
  sleep 3
done

# Run Diesel migrations
echo "Running Diesel migrations..."
diesel setup
diesel migration run

# Execute the main application
exec "$@"
