# Websurfx MCP Server

[![CI](https://github.com/mohannad-mamo/websurfx-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/mohannad-mamo/websurfx-mcp/actions/workflows/ci.yml)

A Model Context Protocol (MCP) server that provides web search capabilities through [Websurfx](https://github.com/neon-mmd/websurfx). This server allows MCP clients like Raycast, Zed, and OpenCode to perform web searches using your own Websurfx instance.

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

## Docker

Build locally:

```bash
docker build -t websurfx-mcp .
```

The Dockerfile is architecture-neutral. To build for both common platforms
(requires Buildx; `--push` publishes a multi-arch manifest):

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t ghcr.io/mohannad-mamo/websurfx-mcp:0.2.0 \
  --push .
```

The runtime image is Alpine-based, runs as a non-root user, and includes
`ca-certificates`, `LICENSE`, and `NOTICE` under
`/usr/share/licenses/websurfx-mcp/`.

### stdio

```bash
docker run --rm -i \
  -e WEBSURFX_BASE_URL=http://host.docker.internal:8080 \
  websurfx-mcp
```

### HTTP

```bash
docker run --rm -p 8001:8001 \
  -e MCP_TRANSPORT=http \
  -e WEBSURFX_BASE_URL=http://host.docker.internal:8080 \
  websurfx-mcp
```

Verify it is up with `curl http://localhost:8001/healthz`.

On Linux, add `--add-host=host.docker.internal:host-gateway` to reach a
Websurfx instance running directly on the host.

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

## Security

The HTTP transport does **not** provide authentication. Anyone who can reach
`/mcp` can run searches through your Websurfx instance.

- Bind to loopback (`MCP_HTTP_BIND=127.0.0.1:8001`) when the client runs on the
  same machine.
- If the server must be reachable over a network, put it behind a reverse proxy
  that enforces authentication and TLS. Do not publish port `8001` directly to
  the internet.
- Docker's `-p 8001:8001` listens on all host interfaces by default.
- Websurfx's own rate limits and safe-search settings still apply.

Use stdio for local clients unless you specifically need the HTTP transport.

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

For example, a DuckDuckGo failure appears as `- duckduckgo: EmptyResultSet` or
`- duckduckgo: RequestError`, rather than silently returning only results from
other engines.

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

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and pull request
guidelines. For security issues, follow [SECURITY.md](SECURITY.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
See [NOTICE](NOTICE) for upstream and fork attribution.

## Acknowledgements

Forked from [nicholasq/searxng-mcp](https://github.com/nicholasq/searxng-mcp)
(MIT, Copyright (c) 2025 Nicholas Quezada), adapted for
[Websurfx](https://github.com/neon-mmd/websurfx).

Websurfx adaptation and subsequent modifications:
Copyright (c) 2026 Mohannad (mohannad-mamo).
