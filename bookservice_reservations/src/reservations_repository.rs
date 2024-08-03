pub use in_memory_reservations_repository::InMemoryReservationsRepository;
pub use postgres_reservations_repository::{
    PostgresReservationsRepository, PostgresReservationsRepositoryConfig,
};

use crate::api::{BookId, ReservationHistoryRecord, UserDetails, UserId, UsernameAndId};

mod in_memory_reservations_repository;
mod postgres_reservations_repository;

#[derive(Debug, thiserror::Error)]
pub enum ReservationsRepositoryError {
    #[error("Book {0} not found")]
    BookNotReserved(BookId),

    #[error("User {0} not found")]
    UserNotFound(UserId),

    #[error("Book {0} already reserved")]
    BookAlreadyReserved(BookId),

    #[error("Book {0} reserved by different user")]
    BookReservedByDifferentUser(BookId),

    #[error("Failed to deserialize book: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("DatabaseFailure failure {0}")]
    DatabaseFailure(#[from] tokio_postgres::Error),

    #[error("Other error {0}")]
    Other(String),
}

#[async_trait::async_trait]
pub trait ReservationsRepository: Send + Sync {
    async fn add_user(&self, username: UserDetails) -> Result<UserId, ReservationsRepositoryError>;

    async fn get_user(&self, id: UserId) -> Result<UserDetails, ReservationsRepositoryError>;

    async fn get_all_user_ids(&self) -> Result<Vec<UserId>, ReservationsRepositoryError>;

    async fn reserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError>;

    async fn unreserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError>;

    async fn get_all_reservations(
        &self,
        user_id: UserId,
    ) -> Result<Vec<BookId>, ReservationsRepositoryError>;

    async fn get_reservations_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<ReservationHistoryRecord>, ReservationsRepositoryError>;
}
