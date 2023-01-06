# actix-template
Template repo for writing web-apps with Actix inside Unleash.

Defaults to running at all interfaces on port 1337.
Two endpoints will be available on
* http://localhost:1337/internal-backstage/metrics 
* http://localhost:1337/internal-backstage/health


# What this repo does
* Shows a way to setup an actix server with correct logging and prometheus metrics for all endpoints
* Shows a way to build a rust project with caches and docker image being built.
* Use Clap to parse command line arguments. This will auto-generate a `--help` command for the binary if you're wondering what options there are
* Sets up a github action clippy action which will give codequality warnings on https://github.com/Unleash/actix-template/security/code-scanning
* Sets up a github action which runs tests, then builds the binary and then a docker image and uploads to aws ECR
* Show a way to host TLS with custom SSL certificates


# What this repo aims to do in the future
* Configure opentelemetry to post traces to JAEGER or other Opentelemetry compatible endpoints


# Interesting documentation
* https://actix.rs/ - main webpage of actix
* https://github.com/paperclip-rs/paperclip - Paperclip (OpenAPI documentation in Rust)
* https://tokio.rs/ - Asynchronous runtime for Rust
* https://clap.rs - Clap - Command line parser for Rust.

# What this repo does not do
* Configure paperclip for openapi documentation. We'll make a separate repo for how to do this
* Setup database migration or database connections. We have an ambition for making this example as well.
* Asynchronous tasks / scheduled tasks.
