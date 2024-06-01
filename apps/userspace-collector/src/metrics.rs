use axum::{routing::get, Router};
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder, TextEncoder};

pub fn init_metrics() {
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus::default_registry().clone())
        .build()
        .unwrap();

    let meter_provider = SdkMeterProvider::builder().with_reader(exporter).build();
    global::set_meter_provider(meter_provider.clone());

    tokio::spawn(listen_for_prometheus_scrape());
}

async fn listen_for_prometheus_scrape() {
    let app = Router::new().route("/metrics", get(metrics_handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn metrics_handler() -> impl axum::response::IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    axum::response::Response::builder()
        .header("Content-Type", encoder.format_type())
        .body::<axum::body::Body>(buffer.into())
        .unwrap()
}
