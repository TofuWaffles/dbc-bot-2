#!/bin/bash

# Function to check if a Docker image exists locally
image_exists() {
  local image="$1"
  if [[ "$(docker images -q "$image" 2> /dev/null)" == "" ]]; then
    return 1  # Image does not exist
  else
    return 0  # Image exists
  fi
}

# Get the image names for the services from docker-compose.yml
postgres_image=$(docker-compose config | awk '/image: / && /postgres/ {print $2}')
dbcbot=$(docker-compose config | awk '/image: / && /dbc-bot/ {print $2}')

# Check if the PostgreSQL image exists and build it if not
if ! image_exists "$postgres_image"; then
  echo "Building the PostgreSQL image..."
  docker-compose build postgres
fi

# Check if the dbc-bot image exists and build it if not
if ! image_exists "$dbcbot"; then
  echo "Building the dbc-bot image..."
  docker-compose build dbc-bot
fi

# Start the PostgreSQL service
echo "Starting PostgreSQL..."
docker-compose up -d postgres

# Wait for PostgreSQL to be fully up and running
echo "Waiting for PostgreSQL to be ready..."
until docker exec $(docker-compose ps -q postgres) pg_isready -U postgres
do
  echo "PostgreSQL is not ready yet. Retrying in 2 seconds..."
  sleep 2
done
echo "PostgreSQL is ready!"

# Start the second service
echo "Starting dbc-bot..."
docker-compose up -d dbc-bot

echo "Both services started successfully."
