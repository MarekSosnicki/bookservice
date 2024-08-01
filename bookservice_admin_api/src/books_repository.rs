use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use crate::api;
use crate::api::{BookDetails, BookId, BookTitleAndId};

#[derive(thiserror::Error, Debug)]
pub enum BookRepositoryError {
    // #[error("an unspecified internal error occurred: {0}")]
    // FailedToProcess(String),
    #[error("Book {0} not found")]
    NotFound(BookId),

    #[error("Failed to recreate patched book: {0}")]
    PatchFailed(#[from] serde_json::Error),
}

pub trait BookRepository {
    /// Adds book to repository, returns an id assigned to the book
    fn add_book(&self, details: BookDetails) -> Result<BookId, BookRepositoryError>;
    /// Updates book in the repository, returns true if book was updated and false if it was not found
    fn update_book(
        &self,
        book_id: BookId,
        patch: api::BookDetailsPatch,
    ) -> Result<bool, BookRepositoryError>;
    /// Retrieves details of the book from repository
    fn get_book(&self, book_id: BookId) -> Result<BookDetails, BookRepositoryError>;
    /// Lists all books in the repository
    fn list_books(&self) -> Result<Vec<BookTitleAndId>, BookRepositoryError>;
}

pub struct InMemoryBookRepository {
    book_sequence_generator: AtomicU64,
    books: parking_lot::RwLock<HashMap<BookId, BookDetails>>,
}

impl InMemoryBookRepository {
    pub fn new() -> Self {
        let result = Self {
            book_sequence_generator: Default::default(),
            books: Default::default(),
        };

        result
            .add_book(BookDetails {
                title: "aaa".to_string(),
                authors: vec!["bbb".to_string()],
                publisher: "ccc".to_string(),
                description: "ddd".to_string(),
                tags: vec!["eee".to_string()],
            })
            .unwrap();

        result
            .add_book(BookDetails {
                title: "1aaa".to_string(),
                authors: vec!["1bbb".to_string()],
                publisher: "1ccc".to_string(),
                description: "1ddd".to_string(),
                tags: vec!["1eee".to_string()],
            })
            .unwrap();

        result
    }
}

impl BookRepository for InMemoryBookRepository {
    fn add_book(&self, details: api::BookDetails) -> Result<BookId, BookRepositoryError> {
        let id = self.book_sequence_generator.fetch_add(1, Ordering::Relaxed);
        self.books.write().insert(id, details);
        Ok(id)
    }

    fn update_book(
        &self,
        book_id: BookId,
        patch: api::BookDetailsPatch,
    ) -> Result<bool, BookRepositoryError> {
        let mut locked_books = self.books.write();
        if let Some(book) = locked_books.get_mut(&book_id) {
            let mut result_book = json!(book);
            json_patch::merge(&mut result_book, &json!(patch));
            let result_book: BookDetails = serde_json::from_value(result_book)?;
            *book = result_book;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_book(&self, book_id: BookId) -> Result<BookDetails, BookRepositoryError> {
        self.books
            .read()
            .get(&book_id)
            .cloned()
            .ok_or(BookRepositoryError::NotFound(book_id))
    }

    fn list_books(&self) -> Result<Vec<BookTitleAndId>, BookRepositoryError> {
        Ok(self
            .books
            .read()
            .iter()
            .map(|(&book_id, details)| BookTitleAndId {
                book_id,
                title: details.title.clone(),
            })
            .collect())
    }
}

// pub struct PostgresBooksRepository {}
//
// impl PostgresBooksRepository {
//     pub fn init() -> Self {
//         Self
//     }
// }
