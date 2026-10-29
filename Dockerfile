FROM rust:1.88-slim AS builder

WORKDIR /usr/src/yt2obsidian

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
COPY prompts ./prompts

RUN touch src/main.rs && cargo build --release


FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y python3 ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && chmod a+rx /usr/local/bin/yt-dlp

WORKDIR /app

COPY --from=builder /usr/src/yt2obsidian/target/release/yt2obsidian /usr/local/bin/yt2obsidian

VOLUME ["/vault"]

ENV VAULT_PATH=/vault
ENV OLLAMA_MODEL=qwen2.5:7b

ENTRYPOINT ["yt2obsidian"]