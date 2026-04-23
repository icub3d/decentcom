FROM rust:latest AS builder

WORKDIR /build

# Cache dependency compilation separately from source.
COPY Cargo.toml Cargo.lock ./
COPY shared/Cargo.toml shared/Cargo.toml
COPY server/Cargo.toml server/Cargo.toml
COPY sdk/rust/Cargo.toml sdk/rust/Cargo.toml
COPY tools/test-setup/Cargo.toml tools/test-setup/Cargo.toml
COPY tools/sdk-seed/Cargo.toml tools/sdk-seed/Cargo.toml

# Stub out lib/main so Cargo can build deps without the real sources.
RUN mkdir -p shared/src server/src sdk/rust/src tools/test-setup/src tools/sdk-seed/src && \
    echo "fn main() {}" > server/src/main.rs && \
    touch shared/src/lib.rs sdk/rust/src/lib.rs tools/test-setup/src/main.rs tools/sdk-seed/src/main.rs

# Exclude the Tauri client crate — it needs platform GUI libs not present here.
RUN sed -i '/"client\/src-tauri"/d' Cargo.toml

RUN cargo build --release -p server

# Replace stubs with real source and rebuild only the changed crate.
COPY shared/src shared/src
COPY server/src server/src
COPY server/migrations server/migrations

RUN touch server/src/main.rs && \
    cargo build --release -p server

# ── Runtime image ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/server /usr/local/bin/server

VOLUME ["/data"]

ENV DECENTCOM_CONFIG=/config/decentcom.toml
ENV DECENTCOM_DB=/data/decentcom.db
ENV DECENTCOM_MEDIA=/data/media

EXPOSE 8080

ENTRYPOINT ["server", "--config", "/config/decentcom.toml"]
