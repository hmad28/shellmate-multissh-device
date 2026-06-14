FROM rust:1.82-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/shellmate-sync-server /usr/local/bin/
EXPOSE 8080
ENV RUST_LOG=info
CMD ["shellmate-sync-server"]
