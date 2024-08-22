use std::time::UNIX_EPOCH;

use anyhow::Context;
use serde_json::json;
use tokio_postgres::{Client, NoTls, Statement};
use tokio_postgres::error::SqlState;

use crate::api::ReservationHistoryRecord;
use crate::reservations_repository::{
    BookId, ReservationsRepository, ReservationsRepositoryError, UserDetails, UserId,
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
            params          JSONB
            )
        ",
            )
            .await
            .context("Failed to setup users table")?;

        client
            .batch_execute(
                "
        CREATE TABLE IF NOT EXISTS reservations (
            book_id              INTEGER NOT NULL UNIQUE,
            user_id              INTEGER NOT NULL
            )
        ",
            )
            .await
            .context("Failed to setup reservations table")?;

        client
            .batch_execute(
                "
        CREATE TABLE IF NOT EXISTS history (
            book_id              INTEGER NOT NULL,
            user_id              INTEGER NOT NULL,
            unreserved_at        BIGINT
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
        let stmt: Statement = self
            .client
            .prepare("INSERT INTO users (params) VALUES ($1) RETURNING id")
            .await?;

        let rows = self.client.query(&stmt, &[&json!(user_data)]).await?;

        let user_id: UserId = rows
            .first()
            .ok_or_else(|| ReservationsRepositoryError::Other("Id not returned".to_string()))?
            .try_get(0)?;

        Ok(user_id)
    }

    async fn get_user(&self, id: UserId) -> Result<UserDetails, ReservationsRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare("SELECT params FROM users WHERE id = ($1)")
            .await?;

        let rows = self.client.query(&stmt, &[&id]).await?;

        let details: serde_json::Value = rows
            .first()
            .ok_or_else(|| ReservationsRepositoryError::UserNotFound(id))?
            .try_get(0)?;

        Ok(serde_json::from_value(details)?)
    }

    async fn get_all_user_ids(&self) -> Result<Vec<UserId>, ReservationsRepositoryError> {
        let stmt: Statement = self.client.prepare("SELECT id FROM users").await?;
        let rows = self.client.query(&stmt, &[]).await?;
        rows.iter().map(|row| Ok(row.try_get(0)?)).collect()
    }

    async fn reserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare(
                "INSERT INTO reservations (book_id, user_id) VALUES ($1, $2) RETURNING user_id",
            )
            .await?;

        let rows = self.client.query(&stmt, &[&book_id, &user_id]).await;

        match rows {
            Ok(rows) if rows.is_empty() => {
                Err(ReservationsRepositoryError::BookAlreadyReserved(book_id))
            }
            Ok(_) => Ok(()),
            Err(err)
                if err
                    .as_db_error()
                    // This is unique constraint validation error
                    .map(|db_err| db_err.code() == &SqlState::from_code("23505"))
                    .unwrap_or_default() =>
            {
                Err(ReservationsRepositoryError::BookAlreadyReserved(book_id))
            }
            Err(other_err) => Err(other_err.into()),
        }
    }

    async fn unreserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare(
                "DELETE FROM reservations WHERE book_id = $1 AND user_id = $2 RETURNING book_id",
            )
            .await?;

        let rows = self.client.query(&stmt, &[&book_id, &user_id]).await?;

        if rows.is_empty() {
            Err(ReservationsRepositoryError::BookNotReservedOrReservedByDifferentUser(book_id))
        } else {
            let stmt: Statement = self
                .client
                .prepare(
                    "INSERT INTO history (book_id, user_id, unreserved_at) VALUES ($1, $2, $3)",
                )
                .await?;

            self.client
                .execute(
                    &stmt,
                    &[
                        &book_id,
                        &user_id,
                        &(std::time::SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64),
                    ],
                )
                .await?;

            Ok(())
        }
    }

    async fn get_all_reservations(
        &self,
        user_id: UserId,
    ) -> Result<Vec<BookId>, ReservationsRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare("SELECT book_id FROM reservations WHERE user_id = $1")
            .await?;
        let rows = self.client.query(&stmt, &[&user_id]).await?;
        rows.iter().map(|row| Ok(row.try_get(0)?)).collect()
    }

    async fn get_reservations_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<ReservationHistoryRecord>, ReservationsRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare("SELECT book_id, unreserved_at FROM history WHERE user_id = $1")
            .await?;

        let rows = self.client.query(&stmt, &[&user_id]).await?;

        rows.iter()
            .map(|row| {
                let book_id = row.try_get(0)?;
                let unreserved_at: i64 = row.try_get(1)?;

                Ok(ReservationHistoryRecord {
                    book_id,
                    unreserved_at,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests_postgres_reservations_repository {
    use serial_test::file_serial;
    use testcontainers::{ContainerAsync, GenericImage, ImageExt};
    use testcontainers::core::IntoContainerPort;
    use testcontainers::runners::AsyncRunner;

    use super::*;

    async fn start_postgres_container_and_init_repo(
    ) -> (ContainerAsync<GenericImage>, PostgresReservationsRepository) {
        let _pg_container = GenericImage::new("postgres", "latest")
            .with_mapped_port(5432, 5432.tcp())
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .start()
            .await
            .expect("Failed to start postgres");

        for _ in 0..10 {
            if let Ok(repo) =
                PostgresReservationsRepository::init(PostgresReservationsRepositoryConfig {
                    hostname: "127.0.0.1".to_string(),
                    username: "postgres".to_string(),
                    password: "postgres".to_string(),
                })
                .await
            {
                return (_pg_container, repo);
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
        panic!("Failed to setup postgres container")
    }

    #[tokio::test]
    #[file_serial(key, path => "../.pgtestslock")]
    /// Simple test to cover user management
    /// Combined into big unit test to avoid duplicate setup
    /// 1. Gets all users -expects empty
    /// 2. Creates user
    /// 3. Gets user
    /// 4. Gets all users - expects 1
    /// 5. Creates second user
    /// 6. Gets all users - expects 2
    /// 7. Gets user not existing in db to get not found
    async fn test_user_management() {
        let (_container, repository) = start_postgres_container_and_init_repo().await;
        assert_eq!(
            repository.get_all_user_ids().await.unwrap(),
            Vec::<UserId>::default()
        );

        let user_details = UserDetails {
            username: "username".to_string(),
            favourite_tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let user_id = repository.add_user(user_details.clone()).await.unwrap();

        let user_returned = repository.get_user(user_id).await.unwrap();

        assert_eq!(user_returned, user_details);
        assert_eq!(repository.get_all_user_ids().await.unwrap(), vec![user_id]);

        let user_2_id = repository
            .add_user(UserDetails {
                username: "user2".to_string(),
                favourite_tags: vec![],
            })
            .await
            .unwrap();
        let mut all_users = repository.get_all_user_ids().await.unwrap();
        all_users.sort();
        assert_eq!(all_users, vec![user_id, user_2_id]);

        let unknown_user_id = user_2_id + 1;

        let get_unknown_user_result = repository.get_user(unknown_user_id).await;
        assert!(matches!(
            get_unknown_user_result,
            Err(ReservationsRepositoryError::UserNotFound(..))
        ));
    }

    #[tokio::test]
    #[file_serial(key, path => "../.pgtestslock")]
    /// Simple test to cover reservation management
    /// Combined into big unit test to avoid duplicate setup
    /// 1.Creates two users, validates reservations and history is empty
    /// 2.Reserves book
    /// 3.Lists all reservations for user
    /// 4.Creates second user
    /// 5.Tries to reserve the same book - get rejected
    /// 6.Releases reservation for the first user
    /// 7.Lists reservations
    async fn test_reservation_management() {
        let (_container, repository) = start_postgres_container_and_init_repo().await;

        let user_1_id = repository
            .add_user(UserDetails {
                username: "user1".to_string(),
                favourite_tags: vec![],
            })
            .await
            .unwrap();
        let user_2_id = repository
            .add_user(UserDetails {
                username: "user1".to_string(),
                favourite_tags: vec![],
            })
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            Vec::<BookId>::default()
        );

        assert_eq!(
            repository
                .get_reservations_history(user_1_id)
                .await
                .unwrap(),
            Vec::<ReservationHistoryRecord>::default()
        );

        let test_book_id: BookId = 123;

        // reserve book for the user
        repository
            .reserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            vec![test_book_id]
        );
        assert_eq!(
            repository
                .get_reservations_history(user_1_id)
                .await
                .unwrap(),
            Vec::<ReservationHistoryRecord>::default()
        );

        let reserve_conflict = repository.reserve_book(user_2_id, test_book_id).await;

        assert!(matches!(
            reserve_conflict,
            Err(ReservationsRepositoryError::BookAlreadyReserved(..))
        ));

        // unreserve book for wrong user
        let unreserve_invalid_user = repository.unreserve_book(user_2_id, test_book_id).await;

        assert!(matches!(
            unreserve_invalid_user,
            Err(ReservationsRepositoryError::BookNotReservedOrReservedByDifferentUser(..))
        ));

        // unreserve book for right user
        repository
            .unreserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            Vec::<BookId>::default()
        );

        let history = repository
            .get_reservations_history(user_1_id)
            .await
            .unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].book_id, test_book_id);

        // reserve again to check if properly release
        repository
            .reserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        // Reserve other book
        let other_book_id: BookId = 5553;
        repository
            .reserve_book(user_1_id, other_book_id)
            .await
            .unwrap();

        let mut two_reservations = repository.get_all_reservations(user_1_id).await.unwrap();
        two_reservations.sort();
        assert_eq!(two_reservations, vec![test_book_id, other_book_id]);

        // Unreserve to see if can have the same book twice separately
        repository
            .unreserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            vec![other_book_id]
        );

        let history = repository
            .get_reservations_history(user_1_id)
            .await
            .unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].book_id, test_book_id);
        assert_eq!(history[1].book_id, test_book_id);
    }
}
