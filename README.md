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
- `MCP_TRANSPORT`: `stdio` (default) or `http`
- `MCP_HTTP_BIND`: Bind address used when `MCP_TRANSPORT=http` (default: `0.0.0.0:8001`)

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

### HTTP Transport

stdio remains the default. To run the server as a centrally hosted MCP server:

```bash
MCP_TRANSPORT=http \
MCP_HTTP_BIND=0.0.0.0:8001 \
WEBSURFX_BASE_URL=http://localhost:8080 \
websurfx-mcp
```

- MCP endpoint: `http://<host>:8001/mcp` (Streamable HTTP, stateless)
- Health check: `http://<host>:8001/healthz`

Point clients that support external MCP servers at the `/mcp` URL. For Open
WebUI >= 0.6.31: Settings -> Admin -> Integrations -> External Tool Servers ->
MCP (Streamable HTTP).

The HTTP server still starts if Websurfx is temporarily unreachable; it logs a
warning and `/healthz` remains available for readiness checks.

### Tool Parameters

The `websurfx_search` tool accepts only the parameters Websurfx actually honors:

- `query` (required): The search query string (max 512 characters)
- `page` (optional): Page number, 1-based (1-50)
- `safe_search` (optional): Safe search level - 0 (none) through 4 (strictest)

The tool advertises `read_only_hint = true` and `open_world_hint = true`, plus
`destructive_hint = false`, `idempotent_hint = false`, and a title. These are
MCP hints for hosts; clients are not required to honor them.

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
# Run unit tests (the live test skips itself unless enabled)
cargo test

# Run the live integration test (requires Websurfx at localhost:8080)
WEBSURFX_LIVE=1 cargo test test_search_integration
```

Set `WEBSURFX_BASE_URL` alongside `WEBSURFX_LIVE=1` to test a different instance.

The default suite is offline: JSON payloads and expected output live under
`tests/fixtures/search/` and are compiled into the tests with `include_str!`.

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
See [NOTICE](NOTICE) for upstream and fork attribution.

## Acknowledgements

Forked from [nicholasq/searxng-mcp](https://github.com/nicholasq/searxng-mcp)
(MIT, Copyright (c) 2025 Nicholas Quezada), adapted for
[Websurfx](https://github.com/neon-mmd/websurfx).

Websurfx adaptation and subsequent modifications:
Copyright (c) 2026 Mohannad (mohannad-mamo).
