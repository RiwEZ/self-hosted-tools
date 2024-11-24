# build stage
FROM rust:1.82-alpine3.19 as builder

# This is important, see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /app

COPY . .

RUN cargo build --release

RUN strip target/release/cloudflare-ddns

FROM alpine:3.19

RUN apk add --no-cache libgcc

COPY --from=builder /app/target/release/cloudflare-ddns .

ENTRYPOINT ["/cloudflare-ddns"]
