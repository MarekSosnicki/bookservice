use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};

use serde_json::json;

use crate::api;
use crate::api::{BookDetails, BookId, BookTitleAndId};
use crate::books_repository::{BookRepository, BookRepositoryError};

pub struct InMemoryBookRepository {
    book_sequence_generator: AtomicI32,
    books: parking_lot::RwLock<HashMap<BookId, BookDetails>>,
}

impl Default for InMemoryBookRepository {
    fn default() -> Self {
        let result = Self {
            book_sequence_generator: Default::default(),
            books: Default::default(),
        };
        result
    }
}

#[async_trait::async_trait]
impl BookRepository for InMemoryBookRepository {
    async fn add_book(&self, details: api::BookDetails) -> Result<BookId, BookRepositoryError> {
        let id = self.book_sequence_generator.fetch_add(1, Ordering::Relaxed);
        self.books.write().insert(id, details);
        Ok(id)
    }

    async fn update_book(
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

    async fn get_book(&self, book_id: BookId) -> Result<BookDetails, BookRepositoryError> {
        self.books
            .read()
            .get(&book_id)
            .cloned()
            .ok_or(BookRepositoryError::NotFound(book_id))
    }

    async fn list_books(&self) -> Result<Vec<BookTitleAndId>, BookRepositoryError> {
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

#[cfg(test)]
mod in_memory_book_repository_tests {
    use crate::api::{BookDetails, BookDetailsPatch, BookTitleAndId};
    use crate::books_repository::{BookRepository, BookRepositoryError, InMemoryBookRepository};

    #[tokio::test]
    /// Tests if add_book and get_book work correctly
    /// for the sake of not starting container multiple times it tests everything in one testcase
    async fn test_add_book_and_get_it() {
        let repo = InMemoryBookRepository::default();

        let not_existing_book_id = 20000;
        let book_not_found = repo.get_book(not_existing_book_id).await;
        assert!(matches!(
            book_not_found,
            Err(BookRepositoryError::NotFound(..))
        ));

        let book_details = BookDetails {
            title: "xx".to_string(),
            authors: vec!["www".to_string()],
            publisher: "".to_string(),
            description: "".to_string(),
            tags: vec!["tag tag".to_string()],
        };
        let id = repo
            .add_book(book_details.clone())
            .await
            .expect("Failed to add book");

        let details = repo.get_book(id).await.expect("Failed to get book");
        assert_eq!(details, book_details);
    }

    #[tokio::test]
    /// Tests if list_books works correctly
    /// for the sake of not starting container multiple times it tests everything in one testcase
    async fn test_add_books_and_list_them() {
        let repo = InMemoryBookRepository::default();

        let list = repo.list_books().await.expect("Failed to list books");
        assert_eq!(list, vec![]);

        let book1_details = BookDetails {
            title: "title1".to_string(),
            authors: vec!["www".to_string()],
            publisher: "".to_string(),
            description: "".to_string(),
            tags: vec!["tag tag".to_string()],
        };

        let book2_details = BookDetails {
            title: "title2".to_string(),
            ..book1_details.clone()
        };

        let id_1 = repo
            .add_book(book1_details.clone())
            .await
            .expect("Failed to add book");

        let list = repo.list_books().await.expect("Failed to list books");

        assert_eq!(
            list,
            vec![BookTitleAndId {
                book_id: id_1,
                title: "title1".to_string(),
            },]
        );

        let id_2 = repo
            .add_book(book2_details.clone())
            .await
            .expect("Failed to add book");

        let mut list = repo.list_books().await.expect("Failed to list books");

        list.sort_by_key(|i| i.book_id);

        assert_eq!(
            list,
            vec![
                BookTitleAndId {
                    book_id: id_1,
                    title: "title1".to_string(),
                },
                BookTitleAndId {
                    book_id: id_2,
                    title: "title2".to_string(),
                }
            ]
        );
    }

    #[tokio::test]
    /// Tests if add_book and get_book work correctly
    /// for the sake of not starting container multiple times it tests everything in one testcase
    async fn test_add_book_patch_and_get_it() {
        let repo = InMemoryBookRepository::default();
        let not_existing_book = 2000;
        let result = repo
            .update_book(not_existing_book, BookDetailsPatch::default())
            .await
            .expect("Failed to update");
        // false means nothing to update
        assert!(!result);

        let book_details = BookDetails {
            title: "xx".to_string(),
            authors: vec!["sss".to_string()],
            publisher: "aaad".to_string(),
            description: "ewqeweq".to_string(),
            tags: vec!["tag tag".to_string()],
        };
        let id = repo
            .add_book(book_details.clone())
            .await
            .expect("Failed to add book");

        let patch_title_only = BookDetailsPatch {
            title: Some("patchedTitle".to_string()),
            ..BookDetailsPatch::default()
        };
        let patch_result = repo
            .update_book(id, patch_title_only)
            .await
            .expect("Failed to patch");
        assert!(patch_result);

        let expected_with_patch_title = BookDetails {
            title: "patchedTitle".to_string(),
            ..book_details.clone()
        };
        assert_eq!(repo.get_book(id).await.unwrap(), expected_with_patch_title);

        let patch_all_fields = BookDetailsPatch {
            title: Some("patchedTitle".to_string()),
            authors: Some(vec!["a".to_string(), "b".to_string()]),
            publisher: Some("c".to_string()),
            description: Some("d".to_string()),
            tags: Some(vec!["e".to_string(), "w".to_string()]),
        };
        let patch_result = repo
            .update_book(id, patch_all_fields)
            .await
            .expect("Failed to patch");
        assert!(patch_result);

        let expected_after_patch = BookDetails {
            title: "patchedTitle".to_string(),
            authors: vec!["a".to_string(), "b".to_string()],
            publisher: "c".to_string(),
            description: "d".to_string(),
            tags: vec!["e".to_string(), "w".to_string()],
        };

        assert_eq!(repo.get_book(id).await.unwrap(), expected_after_patch);
    }
}
