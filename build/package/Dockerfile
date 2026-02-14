FROM rust:1.76-slim-bookworm AS builder

WORKDIR /app
# Install build dependencies for ssh2 (libssh2, openssl)
RUN apt-get update && apt-get install -y libssh2-1-dev libssl-dev pkg-config

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# Install runtime dependencies
RUN apt-get update && apt-get install -y libssh2-1 openssl ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /root/
COPY --from=builder /app/target/release/tacoshell .
CMD ["./tacoshell"]

