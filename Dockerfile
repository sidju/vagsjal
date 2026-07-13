# syntax=docker/dockerfile:labs

# Stage 1: compile dependencies only (cached unless Cargo.toml/Cargo.lock changes)
FROM rust:alpine AS deps
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/vagsjal
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN SQLX_OFFLINE=true cargo build --release --target x86_64-unknown-linux-musl; \
    # Remove main crate artifacts so the real build replaces them
    rm -rf target/x86_64-unknown-linux-musl/release/vagsjal \
           target/x86_64-unknown-linux-musl/release/.fingerprint/vagsjal-* \
           target/x86_64-unknown-linux-musl/release/deps/vagsjal-* \
           target/x86_64-unknown-linux-musl/release/build/vagsjal-*

# Stage 2: build the real binary (deps already cached from stage 1)
FROM deps AS builder
COPY . .
RUN SQLX_OFFLINE=true cargo build --release --target x86_64-unknown-linux-musl
RUN touch .env

# Final image: just the binary
FROM scratch
COPY --from=builder /usr/src/vagsjal/target/x86_64-unknown-linux-musl/release/vagsjal .
COPY --from=builder /usr/src/vagsjal/.env .
COPY --from=builder /etc/ssl/certs etc/ssl/certs
USER 1000
CMD ["./vagsjal"]
