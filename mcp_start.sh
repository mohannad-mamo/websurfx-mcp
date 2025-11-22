#!/bin/bash

set -e

# Create network if it doesn't exist
docker network create searxng 2>/dev/null || true

# Stop and remove existing container if it exists
if docker ps -a --format 'table {{.Names}}' | grep -q "^searxng-mcp$"; then
  echo "Stopping and removing existing searxng-mcp container..."
  docker stop searxng-mcp 2>/dev/null || true
  docker rm searxng-mcp 2>/dev/null || true
fi

# Run the Docker container in foreground
echo "Starting searxng-mcp container..."
docker run \
  --name searxng-mcp \
  --network searxng \
  -e SEARXNG_BASE_URL=http://searxng-server:8080 \
  -i \
  searxng-mcp
