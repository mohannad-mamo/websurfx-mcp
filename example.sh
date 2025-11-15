#!/bin/bash

# Example usage script for SearXNG MCP Server
# This script demonstrates how to set up and test the SearXNG MCP server

set -e

echo "🔍 SearXNG MCP Server Example"
echo "=============================="

# Check if SearXNG is running
echo "📡 Checking if SearXNG is available at localhost:8888..."
if curl -s -f http://localhost:8888/ > /dev/null; then
    echo "✅ SearXNG is running!"
else
    echo "❌ SearXNG is not running at localhost:8888"
    echo "Please start SearXNG first. You can use Docker:"
    echo "docker run -d --name searxng -p 8888:8080 searxng/searxng"
    exit 1
fi

# Set environment variables
export SEARXNG_BASE_URL="http://localhost:8888"
export RUST_LOG="searxng_mcp=info"

echo ""
echo "🔧 Configuration:"
echo "SEARXNG_BASE_URL: $SEARXNG_BASE_URL"
echo "RUST_LOG: $RUST_LOG"

# Build the project
echo ""
echo "🔨 Building the project..."
cargo build --release

echo ""
echo "🚀 Starting SearXNG MCP Server..."
echo "The server will communicate via STDIO (JSON-RPC over stdin/stdout)"
echo "Press Ctrl+C to stop the server"
echo ""

# Start the server
cargo run --release
