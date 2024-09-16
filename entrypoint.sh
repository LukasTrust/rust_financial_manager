#!/bin/sh

# Generate a secret key if not already set
if [ -z "$ROCKET_SECRET_KEY" ]; then
  export ROCKET_SECRET_KEY=$(openssl rand -base64 32)
fi

# Run Diesel migrations
# Note: Ensure diesel_cli is installed in the final runtime image or install it in this script
diesel migration run --database-url "${ROCKET_DATABASES}"

# Execute the main application
exec "$@"
