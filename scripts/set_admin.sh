#!/bin/bash

# Script to set admin status for a user
# Usage: ./set_admin.sh your-email@example.com

if [ -z "$1" ]; then
    echo "Usage: $0 <email>"
    echo "Example: $0 admin@example.com"
    exit 1
fi

EMAIL="$1"

# Load database URL from .env if available
if [ -f ".env" ]; then
    export $(grep -v '^#' .env | xargs)
fi

if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL not set"
    echo "Please set DATABASE_URL environment variable or add it to .env file"
    exit 1
fi

echo "Setting admin status for user: $EMAIL"

psql "$DATABASE_URL" -c "
UPDATE users 
SET is_admin = true 
WHERE email = '$EMAIL';
"

if [ $? -eq 0 ]; then
    echo "✅ Successfully set admin status for $EMAIL"
    echo "User will need to log out and log back in for changes to take effect."
else
    echo "❌ Failed to update user. Make sure the user exists and database is accessible."
    exit 1
fi
