#!/bin/sh
set -e

# Generate a secret key if not already set
if [ -z "$ROCKET_SECRET_KEY" ]; then
  export ROCKET_SECRET_KEY=$(openssl rand -base64 32)
fi

# Wait for PostgreSQL to be ready
until PGPASSWORD=$POSTGRES_PASSWORD psql -h "postgres" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 3
done

# Run Diesel migrations
echo "Running Diesel migrations..."
diesel setup
diesel migration run

# Execute the main application
exec "$@"
