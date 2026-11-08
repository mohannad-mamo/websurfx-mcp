# syntax=docker/dockerfile:1

FROM rust:alpine AS builder

# ring (used by rustls) needs a C toolchain and perl to build.
RUN apk add --no-cache build-base perl

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM alpine:3

RUN apk add --no-cache ca-certificates \
    && addgroup -S websurfx \
    && adduser -S -G websurfx websurfx

WORKDIR /app

COPY --from=builder /src/target/release/websurfx-mcp /usr/local/bin/websurfx-mcp
COPY LICENSE NOTICE /usr/share/licenses/websurfx-mcp/

USER websurfx

# stdio is the default so existing MCP client configs keep working.
ENV MCP_TRANSPORT=stdio
EXPOSE 8001

ENTRYPOINT ["websurfx-mcp"]
