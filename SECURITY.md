# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.2.x   | Yes       |
| < 0.2   | No        |

Security fixes are applied to the latest release.

## Reporting a Vulnerability

Please do **not** open a public GitHub issue for security-sensitive reports.

Use GitHub's private vulnerability reporting:

https://github.com/mohannad-mamo/websurfx-mcp/security/advisories/new

Please include:

- the affected version or commit
- the transport in use (`stdio` or `http`)
- deployment details relevant to the issue
- reproduction steps or a proof of concept
- impact and any suggested fix

If private reporting is unavailable, open a minimal public issue asking for a
private contact channel. Do not include exploit details in that issue.

The maintainer will acknowledge reports as soon as practical and coordinate
disclosure after a fix is available. This is a volunteer project; no bug bounty
is offered.

## Security Model

- `websurfx-mcp` is a read-only search proxy for a Websurfx instance.
- `stdio` is the default transport and is intended for local MCP clients.
- The HTTP transport (`MCP_TRANSPORT=http`) has **no authentication**. Anyone
  who can reach `/mcp` can run searches through the configured Websurfx
  instance. Exposing it directly to the internet is a deployment choice, not a
  vulnerability by itself; run it behind an authenticating reverse proxy with
  TLS.
- The server does not store search queries or results.
- Search results, safe-search behavior, and engine availability are controlled
  by Websurfx and its configuration.

## In Scope

- The `websurfx-mcp` binary and Docker image
- MCP transport handling
- Request parameter validation
- Secret or credential exposure in logs, images, or error output
- Dependency vulnerabilities that affect this project

## Out of Scope

- Vulnerabilities in Websurfx itself; report those to the Websurfx project
- Vulnerabilities in upstream search engines
- Search result content, safe-search/blocklist behavior, or engine availability
- Deliberately exposing the unauthenticated HTTP endpoint despite the README
  warning
- Denial of service caused by Websurfx or upstream search engines

## Disclosure

Please allow reasonable time to investigate and fix the issue before public
disclosure. Reporters will be credited unless they prefer to remain anonymous.
