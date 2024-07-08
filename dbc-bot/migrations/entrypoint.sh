#!/bin/sh
set -e

# Wait for PostgreSQL to be ready
until pg_isready -h "localhost" -U "$POSTGRES_USER" -d "$POSTGRES_DB"; do
  echo "Waiting for PostgreSQL..."
  sleep 2
done

# Apply SQL scripts
for f in /docker-entrypoint-initdb.d/*.sql; do
  echo "Running migration $f..."
  psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f "$f"
done

# Start PostgreSQL server
exec docker-entrypoint.sh postgres
