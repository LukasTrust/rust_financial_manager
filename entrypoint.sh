#!/bin/sh
# Generate a secret key if not already set
if [ -z "$ROCKET_SECRET_KEY" ]; then
  export ENV ROCKET_SECRET_KEY=$(openssl rand -base64 32)
fi

# Start the Rust application
exec "$@"