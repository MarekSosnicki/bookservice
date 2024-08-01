use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use serde_json::json;

use crate::api;
use crate::api::{BookDetails, BookId, BookTitleAndId};

pub trait BooksRepository {
    /// Adds book to repository, returns an id assigned to the book
    fn add_book(&self, details: BookDetails) -> anyhow::Result<BookId>;
    /// Updates book in the repository, returns true if book was updated and false if it was not found
    fn update_book(&self, book_id: BookId, patch: api::BookDetailsPatch) -> anyhow::Result<bool>;
    /// Retrieves details of the book from repository
    fn get_book(&self, book_id: BookId) -> anyhow::Result<Option<BookDetails>>;
    /// Lists all books in the repository
    fn list_books(&self) -> anyhow::Result<Vec<BookTitleAndId>>;
}

pub struct InMemoryBooksRepository {
    book_sequence_generator: AtomicU64,
    books: parking_lot::RwLock<HashMap<BookId, BookDetails>>,
}

impl InMemoryBooksRepository {
    pub fn new() -> Self {
        Self {
            book_sequence_generator: Default::default(),
            books: Default::default(),
        }
    }
}

impl BooksRepository for InMemoryBooksRepository {
    fn add_book(&self, details: api::BookDetails) -> anyhow::Result<BookId> {
        let id = self.book_sequence_generator.fetch_add(1, Ordering::Relaxed);
        self.books.write().insert(id, details);
        Ok(id)
    }

    fn update_book(&self, book_id: BookId, patch: api::BookDetailsPatch) -> anyhow::Result<bool> {
        let mut locked_books = self.books.write();
        if let Some(book) = locked_books.get_mut(&book_id) {
            let mut result_book = json!(book);
            json_patch::merge(&mut result_book, &json!(patch));
            let result_book: BookDetails =
                serde_json::from_value(result_book).context("Failed to recreate patched book")?;
            *book = result_book;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_book(&self, book_id: BookId) -> anyhow::Result<Option<BookDetails>> {
        Ok(self.books.read().get(&book_id).cloned())
    }

    fn list_books(&self) -> anyhow::Result<Vec<BookTitleAndId>> {
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
