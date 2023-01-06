mod cli;
use actix_web::{middleware, web, App, HttpResponse, HttpResponseBuilder, HttpServer};
use actix_web_opentelemetry::{PrometheusMetricsHandler, RequestMetricsBuilder, RequestTracing};
use clap::Parser;
use opentelemetry::{
    global,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
    },
};
use serde_json::json;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

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
    })
    .bind((args.interface.clone(), args.port.clone()))
    .expect(format!("Could not bind to {}:{}", args.interface, args.port).as_str())
    .shutdown_timeout(5); // Graceful shutdown waits for existing connections for up to n seconds

    tokio::select! {
        _ = server.run() => {
            info!("Server received shutdown signal and is shutting down. Bye bye!");
        }
    }

    Ok(())
}
