#!/bin/sh
# Generate a secret key if not already set
if [ -z "$ROCKET_SECRET_KEY" ]; then
  export ENV ROCKET_SECRET_KEY=$(openssl rand -base64 32)
fi

# Extract database connection info from DATABASE_URL
export DATABASE_URL=${DATABASE_URL}

# Wait for Postgres to be ready by using the full DATABASE_URL
until diesel database reset --database-url "$DATABASE_URL"; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up - running Diesel migrations"

diesel setup --database-url "$DATABASE_URL"

# Run Diesel migrations
diesel migration run --database-url "$DATABASE_URL"

# Start the Rust application
exec "$@"