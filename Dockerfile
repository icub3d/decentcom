# syntax=docker/dockerfile:1
FROM rust:latest AS builder

WORKDIR /build

# Copy the entire workspace first to ensure all member crates are present.
# We exclude the client later to avoid unnecessary bloat if needed,
# but for a simple server build, having the manifests is enough.
COPY . .

# Exclude the Tauri client crate — it needs platform GUI libs not present here.
RUN sed -i '/"client\/src-tauri"/d' Cargo.toml

# Use BuildKit cache mounts for the cargo registry and the target directory.
# This significantly speeds up builds by persisting dependencies and artifacts
# between runs, even if Cargo.toml changes (it will only rebuild what's needed).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release -p server && \
    cp target/release/server /usr/local/bin/server

# ── Runtime image ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tini && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/server /usr/local/bin/server

VOLUME ["/data"]

ENV DECENTCOM_CONFIG=/config/decentcom.toml
ENV DECENTCOM_DB=/data/decentcom.db
ENV DECENTCOM_MEDIA=/data/media

EXPOSE 8080

ENTRYPOINT ["tini", "--", "server", "--config", "/config/decentcom.toml"]
