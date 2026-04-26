# ─── 빌드 단계 ───────────────────────────────────────
# 최신 안정 Rust (edition2024 지원 필요)
FROM rust:1-slim AS builder

WORKDIR /app

# 의존성 캐시 활용을 위해 Cargo.toml만 먼저 복사
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/collector*

# 실제 소스 복사 후 빌드
COPY src ./src
RUN cargo build --release

# ─── 런타임 단계 ─────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates tzdata && \
    rm -rf /var/lib/apt/lists/*

ENV TZ=Asia/Seoul

WORKDIR /app
COPY --from=builder /app/target/release/collector /app/collector

CMD ["/app/collector"]
