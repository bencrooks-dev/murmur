# Murmur relay — build + minimal runtime image. Deploy to any host that gives you
# a public TLS endpoint (Fly.io, Render, Railway, a VPS behind Caddy/nginx). The
# platform terminates https/wss and proxies to the relay's ws on :8787.
FROM rust:1-slim AS build
WORKDIR /app
COPY . .
RUN cargo build --release --bin murmur-server

FROM debian:stable-slim
RUN useradd -m murmur
USER murmur
COPY --from=build /app/target/release/murmur-server /usr/local/bin/murmur-server
EXPOSE 8787
CMD ["murmur-server"]
