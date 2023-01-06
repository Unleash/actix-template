FROM alpine:3.17.0

WORKDIR /app
COPY target/aarch64-unknown-linux-musl/release/actix-template /app/
EXPOSE 1337
ENTRYPOINT ["/app/actix-template"]
