# Contributing

Thanks for contributing to `websurfx-mcp`.

## Scope

This server targets Websurfx. SearXNG-only parameters such as `categories`,
`engines`, `language`, `time_range`, `format`, and `pageno` are intentionally
not advertised because Websurfx ignores them. Configure engines, categories, and
language on the Websurfx server itself.

## Development Setup

Prerequisites:

- Rust 1.85 or newer
- A running Websurfx instance for live testing (optional)
- Docker and Buildx for container changes

```bash
git clone https://github.com/mohannad-mamo/websurfx-mcp
cd websurfx-mcp
cargo build
```

## Before Submitting

Run all three:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

For transport changes, also manually verify:

```bash
MCP_TRANSPORT=http \
MCP_HTTP_BIND=127.0.0.1:8001 \
WEBSURFX_BASE_URL=http://localhost:8080 \
cargo run

curl http://127.0.0.1:8001/healthz
```

For Docker changes:

```bash
docker build -t websurfx-mcp .
docker buildx build --platform linux/amd64,linux/arm64 -t websurfx-mcp .
```

## Tests

- The default test suite is offline and must stay that way.
- Fixtures live in `tests/fixtures/search/`.
- Add or update fixtures when changing response parsing or output formatting.
- Live tests must remain gated behind `WEBSURFX_LIVE=1`:

```bash
WEBSURFX_LIVE=1 \
WEBSURFX_BASE_URL=http://localhost:8080 \
cargo test test_search_integration
```

## Code Guidelines

- Keep changes focused and consistent with the surrounding style.
- Do not add crates without a clear reason.
- Keep the tool schema aligned with what Websurfx actually honors.
- Preserve UTF-8-safe truncation and the plain-text tool output format.
- Keep `LICENSE` verbatim. Update `NOTICE` or README acknowledgements when
  attribution changes.
- Do not push to the `upstream` remote (`nicholasq/searxng-mcp`).

## Pull Requests

- Fork the repository and open a PR against `main`.
- Describe the problem, the change, and how you tested it.
- Keep PRs focused; split unrelated refactors.
- Link related issues where applicable.
- Report security issues through `SECURITY.md`, not a public PR or issue.

## Commit Messages

Use clear, imperative messages, for example:

- `fix: send page instead of pageno`
- `feat: add streamable HTTP transport`
- `test: add fixture for engine errors`
- `docs: clarify HTTP security model`

## License

By contributing, you agree that your contributions are licensed under the MIT
License.
