#!/bin/sh

# Function to URL encode special characters
urlencode() {
  # urlencode function for special characters in the user/password
  echo "$1" | sed -e 's/%/%25/g' \
                  -e 's/ /%20/g' \
                  -e 's/!/%21/g' \
                  -e 's/#/%23/g' \
                  -e 's/\$/%24/g' \
                  -e 's/&/%26/g' \
                  -e "s/'/%27/g" \
                  -e 's/(/%28/g' \
                  -e 's/)/%29/g' \
                  -e 's/*/%2A/g' \
                  -e 's/+/%2B/g' \
                  -e 's/,/%2C/g' \
                  -e 's/:/%3A/g' \
                  -e 's/;/%3B/g' \
                  -e 's/=/%3D/g' \
                  -e 's/?/%3F/g' \
                  -e 's/@/%40/g' \
                  -e 's/\[/%5B/g' \
                  -e 's/\]/%5D/g'
}

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

# Log the environment variables for debugging
echo "POSTGRES_USER: $POSTGRES_USER"
echo "POSTGRES_PASSWORD: $POSTGRES_PASSWORD"
echo "POSTGRES_DB: $POSTGRES_DB"

# URL encode the POSTGRES_USER and POSTGRES_PASSWORD
ENCODED_USER=$(urlencode "$POSTGRES_USER")
ENCODED_PASSWORD=$(urlencode "$POSTGRES_PASSWORD")

PROTOCOL="postgres://"
USER_PASS="${ENCODED_USER}:${ENCODED_PASSWORD}"
HOST_PORT="@postgres:5432"
DB_NAME="/${POSTGRES_DB}"

# Concatenate the full DATABASE_URL
DATABASE_URL="${PROTOCOL}${USER_PASS}${HOST_PORT}${DB_NAME}"

# Output the DATABASE_URL for debugging
echo "DATABASE_URL is: $DATABASE_URL"

# Run Diesel migrations using DATABASE_URL
diesel migration run --database-url $DATABASE_URL

# Start the Rust application
exec "$@"
