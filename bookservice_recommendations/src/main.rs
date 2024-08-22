// Based on https://github.com/LukeMathWalker/tracing-actix-web/blob/main/examples/opentelemetry/src/main.rs#L15
#[cfg(feature = "server")]
fn init_telemetry() {
    use opentelemetry::global;
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use opentelemetry_sdk::runtime::TokioCurrentThread;
    use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::{EnvFilter, Registry};

    let app_name = "bookservice_repository";

    // Start a new Jaeger trace pipeline.
    // Spans are exported in batch - recommended setup for a production application.
    global::set_text_map_propagator(TraceContextPropagator::new());
    #[allow(deprecated)]
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(app_name)
        .install_batch(TokioCurrentThread)
        .expect("Failed to install OpenTelemetry tracer.");

    // Filter based on level - trace, debug, info, warn, error
    // Tunable via `RUST_LOG` env variable
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    // Create a `tracing` layer using the Jaeger tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    // Create a `tracing` layer to emit spans as structured logs to stdout
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);
    // Combined them all together in a `tracing` subscriber
    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.")
}

#[cfg(feature = "server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    use bookservice_repository::app_config::config_app;
    use paperclip::actix::OpenApiExt;
    use tracing_actix_web::TracingLogger;

    init_telemetry();
    println!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap_api()
            .wrap(TracingLogger::default())
            .configure(config_app)
            .with_json_spec_at("/apispec/v2")
            .build()
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
