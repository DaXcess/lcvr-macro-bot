# Builder
FROM rust:1.80.1-slim AS builder

WORKDIR /app

# Add extra build dependencies here
RUN apt-get update && apt install -yqq \
    libsqlite3-dev

COPY . .

RUN cargo build --release

# Runtime
FROM debian:bookworm-slim

COPY --from=builder /app/target/release/lcvr-macros /usr/local/bin/lcvr-macros

ENTRYPOINT [ "/usr/local/bin/lcvr-macros" ]