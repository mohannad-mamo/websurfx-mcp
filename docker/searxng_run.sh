#!/bin/bash
set -e

# Create network if it doesn't exist
docker network create searxng 2>/dev/null || true

# Stop and remove existing container if it exists
if docker ps -a --format 'table {{.Names}}' | grep -q "^searxng-server$"; then
  echo "Stopping and removing existing searxng-server container..."
  docker stop searxng-server 2>/dev/null || true
  docker rm searxng-server 2>/dev/null || true
fi

# Run the Docker container
echo "Starting searxng-server container..."
docker run -d --name searxng-server \
  --network searxng \
  -v "searxng_config:/etc/searxng/" \
  -v "searxng_data:/var/cache/searxng/" \
  searxng-server

echo "searxng-server started successfully"
