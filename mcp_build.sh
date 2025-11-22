#!/bin/bash

set -e

# Build the searxng-mcp image (rebuild if it exists)
echo "Building searxng-mcp image..."
docker build -t searxng-mcp .
