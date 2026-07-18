# Kaniscope AI Code Review — GitHub Action (Docker container action).
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
# Cache dependencies against a stub main, then build the real binary.
COPY Cargo.toml ./
COPY Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release || true
COPY . .
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
# CA roots for outbound HTTPS (OpenRouter + GitHub API).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kaniscope-action /usr/local/bin/kaniscope-action
ENTRYPOINT ["/usr/local/bin/kaniscope-action"]
