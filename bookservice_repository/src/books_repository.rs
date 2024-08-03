pub use in_memory_books_repository::InMemoryBookRepository;
pub use postgres_books_repository::{PostgresBooksRepository, PostgresBooksRepositoryConfig};

use crate::api;
use crate::api::{BookDetails, BookId, BookTitleAndId};

mod in_memory_books_repository;
mod postgres_books_repository;

#[derive(thiserror::Error, Debug)]
pub enum BookRepositoryError {
    // #[error("an unspecified internal error occurred: {0}")]
    // FailedToProcess(String),
    #[error("Book {0} not found")]
    NotFound(BookId),

    #[error("Failed to deserialize book: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("DatabaseFailure failure {0}")]
    DatabaseFailure(#[from] tokio_postgres::Error),

    #[error("Other error {0}")]
    Other(String),
}

#[async_trait::async_trait]
pub trait BookRepository {
    /// Adds book to repository, returns an id assigned to the book
    async fn add_book(&self, details: BookDetails) -> Result<BookId, BookRepositoryError>;
    /// Updates book in the repository, returns true if book was updated and false if it was not found
    async fn update_book(
        &self,
        book_id: BookId,
        patch: api::BookDetailsPatch,
    ) -> Result<bool, BookRepositoryError>;
    /// Retrieves details of the book from repository
    async fn get_book(&self, book_id: BookId) -> Result<BookDetails, BookRepositoryError>;
    /// Lists all books in the repository
    async fn list_books(&self) -> Result<Vec<BookTitleAndId>, BookRepositoryError>;
}
