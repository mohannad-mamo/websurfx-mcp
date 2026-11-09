# Websurfx MCP Server

<p align="center">
  <a href="https://github.com/mohannad-mamo/websurfx-mcp/actions/workflows/ci.yml">
    <img alt="CI" src="https://github.com/mohannad-mamo/websurfx-mcp/actions/workflows/ci.yml/badge.svg">
  </a>
  <a href="https://github.com/mohannad-mamo/websurfx-mcp/releases">
    <img alt="Release" src="https://img.shields.io/github/v/release/mohannad-mamo/websurfx-mcp?style=flat-square">
  </a>
  <a href="https://hub.docker.com/r/mohmamo/websurfx-mcp">
    <img alt="Docker pulls" src="https://img.shields.io/docker/pulls/mohmamo/websurfx-mcp?style=flat-square">
  </a>
  <a href="https://hub.docker.com/r/mohmamo/websurfx-all-in-one">
    <img alt="All-in-one pulls" src="https://img.shields.io/docker/pulls/mohmamo/websurfx-all-in-one?style=flat-square">
  </a>
  <a href="LICENSE">
    <img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue?style=flat-square">
  </a>
  <a href="docker/all-in-one/THIRD_PARTY.txt">
    <img alt="All-in-one: AGPL-3.0" src="https://img.shields.io/badge/all--in--one-AGPL--3.0-orange?style=flat-square">
  </a>
  <img alt="Rust 1.85+" src="https://img.shields.io/badge/rust-1.85%2B-blue?style=flat-square">
  <img alt="linux/amd64 and linux/arm64" src="https://img.shields.io/badge/platform-linux%2Famd64%20%7C%20linux%2Farm64-lightgrey?style=flat-square">
</p>

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

Two multi-arch images (`linux/amd64`, `linux/arm64`) are published to both GHCR
and Docker Hub:

| Image | Bundles | GHCR | Docker Hub |
| --- | --- | --- | --- |
| `websurfx-mcp` | MCP server only; point it at an existing Websurfx | `ghcr.io/mohannad-mamo/websurfx-mcp` | `mohmamo/websurfx-mcp` |
| `websurfx-all-in-one` | Websurfx `v1.31.1` + MCP server | `ghcr.io/mohannad-mamo/websurfx-all-in-one` | `mohmamo/websurfx-all-in-one` |

### Tags

Standalone:

| Tag | Meaning |
| --- | --- |
| `edge` | Latest build from `main` |
| `sha-<shortsha>` | Exact commit |
| `0.3.1`, `0.3`, `latest` | Release tags |

All-in-one:

| Tag | Meaning |
| --- | --- |
| `edge` | Latest build from `main` |
| `sha-<shortsha>` | Exact commit |
| `0.3.1-ws1.31.1`, `0.3-ws1.31.1`, `latest` | Release tags |

Pull from either registry:

```bash
docker pull ghcr.io/mohannad-mamo/websurfx-mcp:0.3.1
docker pull mohmamo/websurfx-mcp:0.3.1

docker pull ghcr.io/mohannad-mamo/websurfx-all-in-one:0.3.1-ws1.31.1
docker pull mohmamo/websurfx-all-in-one:0.3.1-ws1.31.1
```

Use `:edge` for the latest development build, or a release tag for a stable
version.

### Standalone image

Use this when you already run Websurfx somewhere.

#### stdio

```bash
IMAGE=ghcr.io/mohannad-mamo/websurfx-mcp:0.3.1
# or: IMAGE=mohmamo/websurfx-mcp:0.3.1

docker run --rm -i \
  -e WEBSURFX_BASE_URL=http://host.docker.internal:8080 \
  "$IMAGE"
```

#### HTTP

```bash
IMAGE=ghcr.io/mohannad-mamo/websurfx-mcp:0.3.1
# or: IMAGE=mohmamo/websurfx-mcp:0.3.1

docker run --rm -p 8001:8001 \
  -e MCP_TRANSPORT=http \
  -e MCP_AUTH_TOKEN=replace-with-a-long-random-token \
  -e WEBSURFX_BASE_URL=http://host.docker.internal:8080 \
  "$IMAGE"
```

Verify it is up with `curl http://localhost:8001/healthz`.

On Linux, add `--add-host=host.docker.internal:host-gateway` to reach a
Websurfx instance running directly on the host.

#### Build locally

```bash
docker build -t websurfx-mcp .
```

The Dockerfile is architecture-neutral. To build for both common platforms
(requires Buildx; `--push` publishes a multi-arch manifest):

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t ghcr.io/mohannad-mamo/websurfx-mcp:0.3.1 \
  --push .
```

The runtime image is Alpine-based, runs as a non-root user, and includes
`ca-certificates`, `LICENSE`, and `NOTICE` under
`/usr/share/licenses/websurfx-mcp/`.

### All-in-one image (experimental)

The all-in-one image bundles Websurfx `v1.31.1` with the MCP server.

```bash
docker run --rm -p 8080:8080 -p 8001:8001 \
  -e MCP_AUTH_TOKEN=replace-with-a-long-random-token \
  ghcr.io/mohannad-mamo/websurfx-all-in-one:0.3.1-ws1.31.1
```

Or use Docker Hub:

```bash
docker run --rm -p 8080:8080 -p 8001:8001 \
  -e MCP_AUTH_TOKEN=replace-with-a-long-random-token \
  mohmamo/websurfx-all-in-one:0.3.1-ws1.31.1
```

- Web UI: `http://localhost:8080`
- MCP: `http://localhost:8001/mcp`
- Health: `http://localhost:8001/healthz`

#### Build locally

```bash
docker build -f docker/all-in-one/Dockerfile -t websurfx-all-in-one .
```

The combined image contains MIT-licensed `websurfx-mcp` and AGPL-3.0-licensed
Websurfx. The exact Websurfx source tarball and AGPL license are included at
`/usr/share/src/websurfx-source.tar.gz` and
`/usr/share/licenses/websurfx/LICENSE-AGPL-3.0`; see
`/usr/share/licenses/THIRD_PARTY.txt`.

## Configuration

| Variable | Default | Description |
| --- | --- | --- |
| `WEBSURFX_BASE_URL` | `http://localhost:8080` | Base URL of the Websurfx instance. |
| `WEBSURFX_TIMEOUT_SECONDS` | `30` | HTTP timeout for search requests. Must be greater than 0. |
| `WEBSURFX_MAX_RESULTS` | `20` | Maximum results included in the tool output. Must be greater than 0. |
| `WEBSURFX_VALIDATION_TIMEOUT_SECONDS` | `10` | Timeout for the startup/health validation request. Must be greater than 0. |
| `MCP_TRANSPORT` | `stdio` | `stdio` or `http`. |
| `MCP_HTTP_BIND` | `0.0.0.0:8001` | Bind address for HTTP mode. |
| `MCP_AUTH_TOKEN` | unset | Optional bearer token required on `/mcp` in HTTP mode. Ignored by stdio. |

### Transport Modes

| Mode | Best for | Authentication |
| --- | --- | --- |
| `stdio` | Local MCP clients launched by the host | Not used |
| `http` | Shared or remote MCP clients | Optional bearer token via `MCP_AUTH_TOKEN` |

### Startup Behavior

- `stdio` validates `WEBSURFX_BASE_URL` at startup and exits if Websurfx is unreachable.
- `http` logs a warning if Websurfx is unreachable but still starts, so `/healthz` can come up.
- Configuration is read at startup; restart the process after changing environment variables.

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
MCP_AUTH_TOKEN=replace-with-a-long-random-token \
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

### Authentication

HTTP mode is **unauthenticated by default**. Set `MCP_AUTH_TOKEN` to require a
bearer token on `/mcp`. `/healthz` always remains public for readiness checks.

Generate and export a token:

```bash
export MCP_AUTH_TOKEN="$(openssl rand -hex 32)"
```

Run the server with it:

```bash
MCP_TRANSPORT=http \
MCP_HTTP_BIND=0.0.0.0:8001 \
MCP_AUTH_TOKEN="$MCP_AUTH_TOKEN" \
WEBSURFX_BASE_URL=http://localhost:8080 \
websurfx-mcp
```

With Docker:

```bash
docker run --rm -p 8001:8001 \
  -e MCP_TRANSPORT=http \
  -e MCP_AUTH_TOKEN="$MCP_AUTH_TOKEN" \
  -e WEBSURFX_BASE_URL=http://host.docker.internal:8080 \
  ghcr.io/mohannad-mamo/websurfx-mcp:0.3.1
```

Clients must send:

```http
Authorization: Bearer <token>
```

Example request:

```bash
curl -X POST http://localhost:8001/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H "Authorization: Bearer $MCP_AUTH_TOKEN" \
  --data '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"curl","version":"0"}}}'
```

Behavior:

- `/mcp` returns `401` when the token is missing or wrong.
- `/healthz` stays public.
- Token comparison is constant-time.
- If `MCP_AUTH_TOKEN` is unset, no authentication is required.
- The token is read at startup; restart the process after changing it.
- The `stdio` transport ignores `MCP_AUTH_TOKEN`.

#### Kubernetes Secret

```bash
kubectl -n apps create secret generic websurfx-mcp-auth \
  --from-literal=MCP_AUTH_TOKEN="$(openssl rand -hex 32)"
```

Read the token back for your client configuration:

```bash
kubectl -n apps get secret websurfx-mcp-auth \
  -o jsonpath='{.data.MCP_AUTH_TOKEN}' | base64 -d
```

Reference it from the Deployment:

```yaml
env:
  - name: MCP_AUTH_TOKEN
    valueFrom:
      secretKeyRef:
        name: websurfx-mcp-auth
        key: MCP_AUTH_TOKEN
```

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

## Security

- The HTTP transport is unauthenticated unless `MCP_AUTH_TOKEN` is set.
- Bind to loopback (`MCP_HTTP_BIND=127.0.0.1:8001`) when the client runs on the
  same machine.
- If the server must be reachable over a network, use TLS and preferably a
  reverse proxy. Bearer tokens sent over plain HTTP are visible on the network.
- Docker's `-p 8001:8001` listens on all host interfaces by default.
- In the all-in-one image, keep the Websurfx Web UI port `8080` internal. The
  Web UI has no authentication.
- Websurfx's own rate limits and safe-search settings still apply.

Use stdio for local clients unless you specifically need the HTTP transport.

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
