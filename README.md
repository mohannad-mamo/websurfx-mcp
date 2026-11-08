# Websurfx MCP Server

A Model Context Protocol (MCP) server that provides web search capabilities through [Websurfx](https://github.com/neon-mmd/websurfx). This server allows MCP clients like Raycast, Zed, and OpenCode to perform web searches using your own Websurfx instance.

> **Breaking change:** the tool is now named `websurfx_search` (previously
> `searxng_search`) and the base URL environment variable is now
> `WEBSURFX_BASE_URL` (previously `SEARXNG_BASE_URL`). Update your MCP client
> configuration accordingly.

## Features

- **Web Search**: Search the web using Websurfx's aggregated results from multiple search engines

## Prerequisites

- [Rust](https://rustup.rs/)
- A running [Websurfx](https://github.com/neon-mmd/websurfx) instance (default: `http://localhost:8080`)

## Installation

1. Clone this repository:
```bash
git clone https://github.com/mohannad-mamo/websurfx-mcp
cd websurfx-mcp
```

2. Install the project:
```bash
cargo install --path .
```

## Configuration

Optional environment variables:

- `WEBSURFX_BASE_URL`: The URL of your Websurfx instance (default: `"http://localhost:8080"`)
- `WEBSURFX_TIMEOUT_SECONDS`: HTTP timeout in seconds (default: 30)
- `WEBSURFX_MAX_RESULTS`: Maximum number of results to display (default: 20)
- `WEBSURFX_VALIDATION_TIMEOUT_SECONDS`: Validation timeout in seconds (default: 10)

## Usage

### With Zed

1. Add the server to your Zed configuration file:

**macOS/Linux**: `~/.config/zed/settings.json`

```json
{
  "mcpServers": {
    "websurfx": {
      "source": "custom",
      "env": {
        "WEBSURFX_BASE_URL": "http://localhost:8080"
      },
      "command": "websurfx-mcp",
      "args": []
    }
  }
}
```

### Tool Parameters

The `websurfx_search` tool accepts only the parameters Websurfx actually honors:

- `query` (required): The search query string (max 512 characters)
- `page` (optional): Page number, 1-based (1-50)
- `safe_search` (optional): Safe search level - 0 (none) through 4 (strictest)

#### Unsupported by Websurfx

Websurfx v1.31.1 silently ignores `categories`, `engines`, `language`, and
`time_range`, as well as the SearXNG-style `format`, `pageno`, and `safesearch`
parameter names. This server therefore does not advertise them. Configure
engines, categories, and language on the Websurfx server itself.

The server sends only `q`, `json=true`, `page`, and `safesearch`.

### Failure Reporting

Websurfx can return no results because an upstream engine failed, a query was
blocked by safe search, or no engines are enabled. Instead of reporting those
as an ordinary empty result, the tool appends an `Engine warnings` list and/or
a notice explaining the status returned by Websurfx.

### Example Queries

Ask Zed:

- "Search for 'rust programming language'"
- "Find recent news about climate change"
- "Search for 'machine learning' tutorials"

## Development

### Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests (requires Websurfx at localhost:8080)
cargo test -- --ignored
```

### Project Structure

```
src/
├── main.rs          # MCP server implementation
├── config.rs        # Configuration management
├── search.rs        # Websurfx API client
└── errors.rs        # Error types and handling
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

Forked from [nicholasq/searxng-mcp](https://github.com/nicholasq/searxng-mcp)
(MIT, Copyright (c) 2025 Nicholas Quezada), adapted for
[Websurfx](https://github.com/neon-mmd/websurfx).
