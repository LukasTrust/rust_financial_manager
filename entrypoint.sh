#!/bin/sh
# Generate a secret key if not already set
if [ -z "$ROCKET_SECRET_KEY" ]; then
  export ENV ROCKET_SECRET_KEY=$(openssl rand -base64 32)
fi

# Load the .env file to set environment variables, including DATABASE_URL
if [ -f .env ]; then
  export $(cat .env | grep -v '^#' | xargs)
fi

# Wait for Postgres to be ready by using the DATABASE_URL
until diesel database reset --database-url "$DATABASE_URL"; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up - running Diesel migrations"

# Run Diesel migrations using DATABASE_URL
diesel migration run --database-url "$DATABASE_URL"

# Start the Rust application
exec "$@"