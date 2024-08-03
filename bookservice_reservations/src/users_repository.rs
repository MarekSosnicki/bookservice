use anyhow::Context;
use tokio_postgres::{Client, NoTls};

#[async_trait::async_trait]
pub trait ReservationsRepository: Send + Sync {}

pub struct InMemoryReservationsRepository {}

impl InMemoryReservationsRepository {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait::async_trait]
impl ReservationsRepository for InMemoryReservationsRepository {}

pub struct PostgresReservationsRepositoryConfig {
    pub hostname: String,
    pub username: String,
    pub password: String,
}

pub struct PostgresReservationsRepository {
    client: Client,
}

impl PostgresReservationsRepository {
    pub async fn init(config: PostgresReservationsRepositoryConfig) -> anyhow::Result<Self> {
        let connection_str = format!(
            "postgresql://{}:{}@{}",
            config.username, config.password, config.hostname
        );
        tracing::info!("Postgres connection_str: {}", connection_str);
        let (client, connection) = tokio_postgres::connect(&connection_str, NoTls)
            .await
            .context("Failed to start postgres")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        client
            .batch_execute(
                "
        CREATE TABLE IF NOT EXISTS books (
            id              SERIAL PRIMARY KEY,
            params          JSONB
            )
        ",
            )
            .await
            .context("Failed to setup table")?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl ReservationsRepository for PostgresReservationsRepository {}
