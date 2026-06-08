# syntax=docker/dockerfile:labs

# Build in one container
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/vagsjal
COPY . .
RUN SQLX_OFFLINE=true cargo install --path . --target x86_64-unknown-linux-musl
RUN touch .env

# Create a container holding only the built binary
FROM scratch
COPY --from=builder /usr/local/cargo/bin/vagsjal .
COPY --from=builder /usr/src/vagsjal/.env .
COPY --from=builder /etc/ssl/certs etc/ssl/certs
USER 1000
CMD ["./vagsjal"]
