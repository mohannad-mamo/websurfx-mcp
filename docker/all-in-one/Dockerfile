# syntax=docker/dockerfile:1

ARG WEBSURFX_TAG=v1.31.1
ARG WEBSURFX_REPO=https://github.com/neon-mmd/websurfx
ARG S6_OVERLAY_VERSION=3.2.3.2

FROM rust:1.94.1-alpine AS websurfx-builder

ARG WEBSURFX_TAG
ARG WEBSURFX_REPO

# Websurfx vendors LuaJIT and builds C code, so it needs the full toolchain.
RUN apk add --no-cache \
    alpine-sdk musl-dev g++ make libcrypto3 libressl-dev libressl-static perl build-base

WORKDIR /src
RUN wget -q -O /tmp/websurfx.tar.gz \
        "${WEBSURFX_REPO}/archive/refs/tags/${WEBSURFX_TAG}.tar.gz" \
    && mkdir -p websurfx \
    && tar -xzf /tmp/websurfx.tar.gz -C websurfx --strip-components=1

WORKDIR /src/websurfx
RUN RUSTFLAGS="-C link-arg=-lgcc" cargo build --release --locked

FROM rust:1.94.1-alpine AS mcp-builder

RUN apk add --no-cache build-base perl

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM alpine:3

ARG TARGETARCH
ARG S6_OVERLAY_VERSION=3.2.3.2

RUN case "$TARGETARCH" in \
        amd64) S6_ARCH=x86_64 ;; \
        arm64) S6_ARCH=aarch64 ;; \
        *) echo "Unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && apk add --no-cache ca-certificates xz \
    && wget -q -O /tmp/s6-noarch.tar.xz \
        "https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz" \
    && wget -q -O /tmp/s6-arch.tar.xz \
        "https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-${S6_ARCH}.tar.xz" \
    && tar -C / -Jxpf /tmp/s6-noarch.tar.xz \
    && tar -C / -Jxpf /tmp/s6-arch.tar.xz \
    && rm -f /tmp/s6-*.tar.xz \
    && apk del xz \
    && addgroup -S websurfx \
    && adduser -S -G websurfx websurfx \
    && mkdir -p \
        /home/websurfx \
        /opt/websurfx \
        /etc/xdg/websurfx \
        /usr/share/licenses/websurfx \
        /usr/share/licenses/websurfx-mcp \
        /usr/share/src \
    && chown websurfx:websurfx /home/websurfx

WORKDIR /opt/websurfx

# Websurfx (AGPL-3.0) and its corresponding source for this exact tag.
COPY --from=websurfx-builder /src/websurfx/target/release/websurfx /usr/local/bin/websurfx
COPY --from=websurfx-builder /src/websurfx/public /opt/websurfx/public
COPY --from=websurfx-builder /src/websurfx/LICENSE /usr/share/licenses/websurfx/LICENSE-AGPL-3.0
COPY --from=websurfx-builder /tmp/websurfx.tar.gz /usr/share/src/websurfx-source.tar.gz

# websurfx-mcp (MIT).
COPY --from=mcp-builder /src/target/release/websurfx-mcp /usr/local/bin/websurfx-mcp
COPY LICENSE NOTICE /usr/share/licenses/websurfx-mcp/
COPY docker/all-in-one/THIRD_PARTY.txt /usr/share/licenses/THIRD_PARTY.txt

COPY docker/all-in-one/config.lua /etc/xdg/websurfx/config.lua
COPY docker/all-in-one/allowlist.txt /etc/xdg/websurfx/allowlist.txt
COPY docker/all-in-one/blocklist.txt /etc/xdg/websurfx/blocklist.txt
COPY docker/all-in-one/s6/ /etc/s6-overlay/s6-rc.d/
COPY docker/all-in-one/user-bundles.d/ /etc/s6-overlay/user-bundles.d/

ENV MCP_TRANSPORT=http \
    MCP_HTTP_BIND=0.0.0.0:8001 \
    WEBSURFX_BASE_URL=http://127.0.0.1:8080 \
    S6_BEHAVIOUR_IF_STAGE2_FAILS=2 \
    S6_LOGGING=0

EXPOSE 8080 8001

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD wget -qO- http://127.0.0.1:8001/healthz >/dev/null \
        && wget -qO- http://127.0.0.1:8080/ >/dev/null \
        || exit 1

ENTRYPOINT ["/init"]
