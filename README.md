# SearXNG MCP Server

A Model Context Protocol (MCP) server that provides web search capabilities through [SearXNG](https://github.com/searxng/searxng). This server allows MCP clients like Raycast, Zed, and OpenCode to perform web searches using your own SearXNG instance.

## Just get to point and tell me how to run this thing!

There are recommended ways to run this mcp server:
1. SearXNG server running in a container and mcp server running on the host.
 - `$ docker/searxng_run.sh`
 - `$ cargo install --path .`
2. SearXNG server running in a container and mcp server running in a container.
 - `$ docker/searxng_run.sh`
 - `$ ./mcp_build.sh`
 - `$ ./mcp_start.sh`

If you want anything else, you are on your own.
Read the scripts **FIRST** before running them in case you need to make changes!

## Features

- **Web Search**: Search the web using SearXNG's aggregated results from multiple search engines

## Prerequisites

- [Rust](https://rustup.rs/)
- Docker (if you plan on running SearXNG in a container)

### Quick Setup

For an interactive setup with docker, run the provided setup script:

```bash
./setup.sh
```

Note: This script will map host port 8080 to container port 8080 for SearXNG. It also creates a searxng network so that if you decide to run the mcp server in a docker container, it will be able to communicate with the SearXNG container. If you do not like this configuration, then make the appropriate changes to the script.

This script provides an interactive menu that will:
- Check Docker availability
- Build the SearXNG Docker image
- Start the SearXNG container
- Build the MCP Docker image

It does NOT start the MCP server container because that should be done in the configuration of your MCP client.

### Manual Docker Setup

If you prefer to set up manually with Docker, you can reference these scripts:

- **[mcp_build.sh](./mcp_build.sh)** - Shows how to build the MCP server image
- **[docker/searxng_run.sh](./docker/searxng_run.sh)** - Shows how to build and run the SearXNG server container

Basic manual setup:
```bash
# Build and run SearXNG
cd docker/
docker build -t searxng-server .
docker run -d --name searxng-server --network searxng -v "searxng_config:/etc/searxng/" -v "searxng_data:/var/cache/searxng/" searxng-server
cd ..

# Build MCP client
docker build -t searxng-mcp .
```

## Installation

1. Clone this repository:
```bash
git clone https://github.com/nicholasq/searxng-mcp
cd searxng-mcp
```

2. Install the project:
```bash
cargo install --path .
```

## Configuration

Optional environment variables:
- `SEARXNG_BASE_URL` The url to your SearXNG instance (default: "http://localhost:8080")
- `SEARXNG_TIMEOUT_SECONDS`: HTTP timeout in seconds (default: 30)
- `SEARXNG_MAX_RESULTS`: Maximum number of results to display (default: 20)
- `SEARXNG_VALIDATION_TIMEOUT_SECONDS`: Validation timeout in seconds (default: 10)

## Usage

### With Zed

1. Add the server to your Zed configuration file:

**macOS/Linux**: `~/.config/zed/settings.json`

```json
{
  "mcpServers": {
    "searxng": {
      "source": "custom",
      "env": {
        "SEARXNG_BASE_URL": "http://localhost:8080"
      },
      "command": "searxng-mcp",
      "args": []
    }
  }
}
```

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

Ask Zed:
- "Search for 'rust programming language'"
- "Search for recent news about climate change in the past month"
- "Find images of cats using safe search"
- "Search for 'machine learning' in academic sources"

## Development

### Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests (requires SearXNG at localhost:8080)
cargo test -- --ignored
```

### Project Structure

```
src/
├── main.rs          # MCP server implementation
├── config.rs        # Configuration management
├── search.rs        # SearXNG API client
└── errors.rs        # Error types and handling
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

Forked from [nicholasq/searxng-mcp](https://github.com/nicholasq/searxng-mcp)
(MIT, Copyright (c) 2025 Nicholas Quezada), adapted for
[Websurfx](https://github.com/neon-mmd/websurfx).
