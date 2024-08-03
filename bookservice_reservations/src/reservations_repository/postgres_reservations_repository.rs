use anyhow::Context;
use tokio_postgres::{Client, NoTls};

use crate::api::ReservationHistoryRecord;
use crate::reservations_repository::{
    BookId, ReservationsRepository, ReservationsRepositoryError, UserDetails, UserId, UsernameAndId,
};

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
        CREATE TABLE IF NOT EXISTS users (
            id              SERIAL PRIMARY KEY,
            username        VARCHAR
            )
        ",
            )
            .await
            .context("Failed to setup users table")?;

        client
            .batch_execute(
                "
        CREATE TABLE IF NOT EXISTS reservations (
            book_id              SERIAL PRIMARY KEY,
            user_id              integer
            )
        ",
            )
            .await
            .context("Failed to setup reservations table")?;

        client
            .batch_execute(
                "
        CREATE TABLE IF NOT EXISTS history (
            book_id              SERIAL PRIMARY KEY,
            user_id              integer,
            unreserved_at        timestamp
            )
        ",
            )
            .await
            .context("Failed to setup reservations table")?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl ReservationsRepository for PostgresReservationsRepository {
    async fn add_user(
        &self,
        user_data: UserDetails,
    ) -> Result<UserId, ReservationsRepositoryError> {
        todo!()
    }

    async fn get_user(&self, id: UserId) -> Result<UserDetails, ReservationsRepositoryError> {
        todo!()
    }

    async fn get_all_users(
        &self,
        username: String,
    ) -> Result<Vec<UsernameAndId>, ReservationsRepositoryError> {
        todo!()
    }

    async fn reserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        todo!()
    }

    async fn unreserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        todo!()
    }

    async fn get_all_reservations(
        &self,
        user_id: UserId,
    ) -> Result<Vec<BookId>, ReservationsRepositoryError> {
        todo!()
    }

    async fn get_reservations_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<ReservationHistoryRecord>, ReservationsRepositoryError> {
        todo!()
    }
}
