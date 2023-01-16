use actix_web_opentelemetry::PrometheusMetricsHandler;
use opentelemetry::sdk::{metrics::{controllers, processors, selectors}, export::metrics::aggregation};
use prometheus::{Registry, process_collector::ProcessCollector};

pub fn http_metrics_handler(registry: Registry) -> PrometheusMetricsHandler {
    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]), // Will give histogram for with resolution in n ms
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();

    let exporter = opentelemetry_prometheus::exporter(controller).with_registry(registry).init();
    PrometheusMetricsHandler::new(exporter)
}

pub fn registry() -> Registry {
    let registry = Registry::new();
    
    #[cfg(target_os = "linux")]
    let process_collector = ProcessCollector::for_self();
    let _register_result = registry.register(Box::new(process_collector));
    
    registry
}