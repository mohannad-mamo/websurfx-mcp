# SearXNG MCP Server

A Model Context Protocol (MCP) server that provides web search capabilities through SearXNG. This server allows AI assistants like Claude Desktop to perform web searches using your own SearXNG instance.

## Features

- **Web Search**: Search the web using SearXNG's aggregated results from multiple search engines
- **Flexible Parameters**: Support for categories, engines, language, pagination, time range, and safe search
- **Error Handling**: Robust error handling with user-friendly messages
- **Async Performance**: Built with Tokio for high-performance async operations

## Prerequisites

- Rust 1.70 or higher
- A running SearXNG instance (see [SearXNG Installation](https://docs.searxng.org/admin/installation.html))

## Installation

1. Clone this repository:
```bash
git clone <your-repo-url>
cd searxng-mcp
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

Set the required environment variable:

```bash
export SEARXNG_BASE_URL="http://localhost:8888"
```

Optional environment variables:
- `SEARXNG_TIMEOUT_SECONDS`: HTTP timeout in seconds (default: 30)
- `SEARXNG_MAX_RESULTS`: Maximum number of results to display (default: 20)
- `RUST_LOG`: Logging level (default: "searxng_mcp=info,rmcp=warn")

## Usage

### With Claude Desktop

1. Add the server to your Claude Desktop configuration file:

**macOS/Linux**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "searxng": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/absolute/path/to/searxng-mcp",
      "env": {
        "SEARXNG_BASE_URL": "http://localhost:8888"
      }
    }
  }
}
```

2. Restart Claude Desktop

3. The `searxng_search` tool will be available for use.

### Tool Parameters

The `searxng_search` tool accepts the following parameters:

- `query` (required): The search query string
- `categories` (optional): Comma-separated list of categories (e.g., "general,news,images")
- `engines` (optional): Comma-separated list of specific engines to use
- `language` (optional): Language code for results (e.g., "en", "fr", "de")
- `page` (optional): Page number for pagination (starts at 1)
- `time_range` (optional): Time filter - "day", "month", or "year"
- `safe_search` (optional): Safe search level - 0 (none), 1 (moderate), 2 (strict)

### Example Queries

Ask Claude:
- "Search for 'rust programming language'"
- "Search for recent news about climate change in the past month"
- "Find images of cats using safe search"
- "Search for 'machine learning' in academic sources"

## Development

### Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests (requires SearXNG at localhost:8888)
cargo test --ignored
```

### Project Structure

```
src/
├── main.rs          # MCP server implementation
├── config.rs        # Configuration management
├── search.rs        # SearXNG API client
└── errors.rs        # Error types and handling
```

### Key Components

- **SearxngServer**: Main MCP server struct implementing the ServerHandler trait
- **SearxngClient**: HTTP client for interacting with SearXNG API
- **SearchParams**: Request parameters for search queries
- **SearchResponse**: Parsed response from SearXNG

## Error Handling

The server provides comprehensive error handling for:
- Missing or invalid environment variables
- Network connectivity issues
- Invalid search parameters
- SearXNG API errors
- JSON parsing errors

## Logging

Logs are written to stderr to avoid interfering with MCP communication. Set `RUST_LOG` environment variable to control logging level:

```bash
export RUST_LOG="searxng_mcp=debug,rmcp=info"
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## SearXNG Setup

If you don't have SearXNG running, you can set it up using Docker:

```bash
# Using Docker
docker run -d \
  --name searxng \
  -p 8888:8080 \
  -v "${PWD}/searxng:/etc/searxng" \
  searxng/searxng

# Or using Docker Compose
version: '3.7'
services:
  searxng:
    image: searxng/searxng
    container_name: searxng
    ports:
      - "8888:8080"
    volumes:
      - ./searxng:/etc/searxng
    environment:
      - SEARXNG_BASE_URL=http://localhost:8888/
```

For more advanced SearXNG setup options, refer to the [official SearXNG documentation](https://docs.searxng.org/).

## Troubleshooting

### Common Issues

1. **"SearXNG server URL not configured"**
   - Ensure `SEARXNG_BASE_URL` environment variable is set
   - Verify the URL is accessible

2. **"Failed to connect to SearXNG server"**
   - Check if SearXNG is running and accessible
   - Verify firewall settings
   - Test the URL in a browser

3. **Tool not showing up in Claude Desktop**
   - Verify the configuration file path is correct
   - Check the JSON syntax
   - Restart Claude Desktop completely
   - Check Claude Desktop logs: `~/Library/Logs/Claude/mcp*.log`

4. **Build failures**
   - Ensure Rust 1.70+ is installed
   - Run `cargo clean` and retry
   - Check for dependency conflicts

### Debug Mode

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug cargo run
```
