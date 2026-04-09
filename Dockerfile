# ── Stage 1: Build ──
FROM rust:1.87-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Cache dependencies by building a dummy project first
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

RUN groupadd -r pockerra && useradd -r -g pockerra -s /sbin/nologin pockerra

COPY --from=builder /app/target/release/pockerra-backend /usr/local/bin/pockerra-backend

USER pockerra

EXPOSE 3000

CMD ["pockerra-backend"]
