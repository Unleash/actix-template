FROM alpine:3.20.3

WORKDIR /app
COPY target/aarch64-unknown-linux-musl/release/actix-template /app/
EXPOSE 1337
ENTRYPOINT ["/app/actix-template"]
