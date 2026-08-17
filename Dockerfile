# syntax=docker/dockerfile:labs

# Build stage, compile incrementally to optimise cache usage
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/vagsjal
# Step 1: deps only
COPY Cargo.toml Cargo.lock ./
RUN mkdir src wiki templates && echo 'fn main() {}' > src/main.rs
RUN SQLX_OFFLINE=true cargo build --release --target x86_64-unknown-linux-musl
# Step 2: build.rs
COPY build.rs ./
RUN SQLX_OFFLINE=true cargo build --release --target x86_64-unknown-linux-musl
# Step 3: wiki markdown
COPY homepage.md ./
COPY wiki wiki
RUN touch homepage.md wiki/*
RUN SQLX_OFFLINE=true cargo build --release --target x86_64-unknown-linux-musl
RUN cp templates/wiki_nav_partial.html /tmp/
# Step 4: full source
RUN rm src/main.rs
COPY src src
COPY assets assets
COPY templates templates
COPY migrations migrations
COPY .sqlx .sqlx
RUN touch src/main.rs
RUN cp /tmp/wiki_nav_partial.html templates/
RUN SQLX_OFFLINE=true cargo build --release --target x86_64-unknown-linux-musl
RUN touch .env

# Final image: just the binary
FROM scratch
COPY --from=builder /usr/src/vagsjal/target/x86_64-unknown-linux-musl/release/vagsjal .
COPY --from=builder /usr/src/vagsjal/.env .
COPY --from=builder /etc/ssl/certs etc/ssl/certs
USER 1000
CMD ["./vagsjal"]
