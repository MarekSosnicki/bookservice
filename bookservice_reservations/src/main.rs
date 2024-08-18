use std::env;
use std::sync::Arc;

use actix_web::{App, HttpServer};
use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::TokioCurrentThread;
use paperclip::actix::{OpenApiExt, web};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;

use bookservice_reservations::app_config::config_app;
use bookservice_reservations::book_existance_checker::BookExistanceChecker;
use bookservice_reservations::reservations_repository::{
    InMemoryReservationsRepository, PostgresReservationsRepository,
    PostgresReservationsRepositoryConfig, ReservationsRepository,
};

// Based on https://github.com/LukeMathWalker/tracing-actix-web/blob/main/examples/opentelemetry/src/main.rs#L15
fn init_telemetry() {
    let app_name = "bookservice_reservations";

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_telemetry();
    println!("starting HTTP server at http://localhost:8080");

    let use_in_memory_db = env::var("USE_IN_MEMORY_DB")
        .map(|value| value.to_lowercase() == "true")
        .unwrap_or_default();
    let pg_hostname = env::var("DB_HOST").unwrap_or("127.0.0.1".to_string());
    let pg_username = env::var("DB_USERNAME").unwrap_or("postgres".to_string());
    let pg_password = env::var("DB_PASSWORD").unwrap_or("postgres".to_string());
    let bookservice_repository_url =
        env::var("BOOKSERVICE_REPOSITORY_URL").unwrap_or("http://localhost:8080".to_string());

    let books_repository: Arc<dyn ReservationsRepository> = if use_in_memory_db {
        Arc::new(InMemoryReservationsRepository::default())
    } else {
        Arc::new(
            PostgresReservationsRepository::init(PostgresReservationsRepositoryConfig {
                hostname: pg_hostname,
                username: pg_username,
                password: pg_password,
            })
            .await
            .expect("Failed to init postgres"),
        )
    };

    HttpServer::new(move || {
        App::new()
            .wrap_api()
            .app_data(web::Data::new(books_repository.clone()))
            .app_data(web::Data::new(BookExistanceChecker::new(
                bookservice_repository_url.clone(),
            )))
            .wrap(TracingLogger::default())
            .configure(config_app)
            .with_json_spec_at("/apispec/v2")
            .build()
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
