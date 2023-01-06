mod cli;
use std::{fs::File, io::BufReader, path::PathBuf};

use actix_tls::accept::rustls::reexports::ServerConfig;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use actix_web_opentelemetry::{PrometheusMetricsHandler, RequestMetricsBuilder, RequestTracing};
use clap::Parser;
use opentelemetry::{
    global,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
    },
};
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde_json::json;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn configure_tls(
    server_cert: PathBuf,
    server_key: PathBuf,
) -> Result<ServerConfig, anyhow::Error> {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();
    let mut cert_file = BufReader::new(File::open(server_cert.as_path())?);
    let mut key_file = BufReader::new(File::open(server_key.as_path())?);
    let cert_chain = certs(&mut cert_file)?
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_file)?
        .into_iter()
        .map(PrivateKey)
        .collect();
    config
        .with_single_cert(cert_chain, keys.remove(0))
        .map_err(|e| anyhow::Error::new(e))
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let args = cli::ServerArgs::parse();

    let logger = tracing_subscriber::fmt::layer();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    let collector = Registry::default().with(logger).with(env_filter);
    // Initialize tracing
    tracing::subscriber::set_global_default(collector).unwrap();
    let metrics_handler = {
        let controller = controllers::basic(
            processors::factory(
                selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]), // Will give histogram for with resolution in n ms
                aggregation::cumulative_temporality_selector(),
            )
            .with_memory(true),
        )
        .build();

        let exporter = opentelemetry_prometheus::exporter(controller).init();
        PrometheusMetricsHandler::new(exporter)
    };
    let meter = global::meter("actix_web");
    let request_metrics = RequestMetricsBuilder::new().build(meter);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(request_metrics.clone())
            .wrap(middleware::Logger::default().exclude("/internal-backstage")) // Will log all requests to server except requests hitting /internal-backstage/ subpaths
            .service(
                web::scope("/internal-backstage")
                    .service(
                        web::resource("/metrics").route(web::get().to(metrics_handler.clone())),
                    )
                    .service(web::resource("/health").route(web::get().to(|| async {
                        HttpResponse::Ok().json(json!({
                            "status": "OK"
                        }))
                    }))),
            )
    });
    let server = if let (Some(server_cert), Some(server_key)) =
        (args.ssl.server_cert, args.ssl.server_key)
    {
        if !server_cert.exists() {
            panic!("server_cert {:#?} does not exist", server_cert);
        }
        if !server_key.exists() {
            panic!("server_key {:#?} does not exist", server_key);
        }
        let tls_config = configure_tls(server_cert, server_key).expect("Failed to configure TLS");
        server.bind_rustls((args.interface.clone(), args.ssl.ssl_port), tls_config)
    } else {
        server.bind((args.interface.clone(), args.port))
    }
    .unwrap_or_else(|_| {
        panic!("Could not bind to {}:{}", args.interface, args.port);
    })
    .shutdown_timeout(5); // Graceful shutdown waits for existing connections for up to n seconds

    tokio::select! {
        _ = server.run() => {
            info!("Server received shutdown signal and is shutting down. Bye bye!");
        }
    }

    Ok(())
}
