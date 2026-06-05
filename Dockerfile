FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev perl make
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/hermes_newsletter_script /usr/local/bin/hermes_newsletter_script

ENTRYPOINT ["/usr/local/bin/hermes_newsletter_script"]
