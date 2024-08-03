use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};

use anyhow::Context;
use serde_json::json;
use tokio_postgres::{Client, NoTls, Statement};

use crate::api;
use crate::api::{BookDetails, BookDetailsPatch, BookId, BookTitleAndId};
use crate::books_repository::BookRepositoryError::Other;

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

pub struct InMemoryBookRepository {
    book_sequence_generator: AtomicI32,
    books: parking_lot::RwLock<HashMap<BookId, BookDetails>>,
}

impl InMemoryBookRepository {
    pub fn new() -> Self {
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

pub struct PostgresBooksRepository {
    client: Client,
}

pub struct PostgresBooksRepositoryConfig {
    pub hostname: String,
    pub username: String,
    pub password: String,
}

impl PostgresBooksRepository {
    pub async fn init(config: PostgresBooksRepositoryConfig) -> anyhow::Result<Self> {
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
impl BookRepository for PostgresBooksRepository {
    async fn add_book(&self, details: BookDetails) -> Result<BookId, BookRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare("INSERT INTO books (params) VALUES ($1) RETURNING id")
            .await?;

        let rows = self.client.query(&stmt, &[&json!(details)]).await?;

        let book_id: BookId = rows
            .first()
            .ok_or_else(|| BookRepositoryError::Other("Id not returned".to_string()))?
            .try_get(0)?;

        Ok(book_id)
    }

    async fn update_book(
        &self,
        book_id: BookId,
        patch: BookDetailsPatch,
    ) -> Result<bool, BookRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare("UPDATE books SET params = params || ($1)::JSONB WHERE id = ($2) RETURNING id")
            .await?;

        let rows = self.client.query(&stmt, &[&json!(patch), &book_id]).await?;
        Ok(rows.len() > 0)
    }

    async fn get_book(&self, book_id: BookId) -> Result<BookDetails, BookRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare("SELECT params FROM books WHERE id = ($1)")
            .await?;

        let rows = self.client.query(&stmt, &[&book_id]).await?;

        let details: serde_json::Value = rows
            .first()
            .ok_or_else(|| BookRepositoryError::NotFound(book_id))?
            .try_get(0)?;

        Ok(serde_json::from_value(details)?)
    }

    async fn list_books(&self) -> Result<Vec<BookTitleAndId>, BookRepositoryError> {
        let stmt: Statement = self
            .client
            .prepare("SELECT id, params->'title' FROM books")
            .await?;

        let rows = self.client.query(&stmt, &[]).await?;

        rows.iter()
            .map(|row| {
                let book_id = row.try_get(0)?;
                let title_json: serde_json::Value = row.try_get(1)?;

                Ok(BookTitleAndId {
                    book_id,
                    title: title_json
                        .as_str()
                        .ok_or_else(|| Other("Title is not string".to_string()))?
                        .to_string(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use testcontainers::{ContainerAsync, GenericImage, ImageExt};
    use testcontainers::core::IntoContainerPort;
    use testcontainers::runners::AsyncRunner;

    use crate::api::{BookDetails, BookDetailsPatch, BookTitleAndId};
    use crate::books_repository::{
        BookRepository, BookRepositoryError, PostgresBooksRepository, PostgresBooksRepositoryConfig,
    };

    async fn start_postgres_container_and_init_repo(
    ) -> (ContainerAsync<GenericImage>, PostgresBooksRepository) {
        let _pg_container = GenericImage::new("postgres", "latest")
            .with_mapped_port(5432, 5432.tcp())
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .start()
            .await
            .expect("Failed to start postgres");

        for _ in 0..10 {
            if let Ok(repo) = PostgresBooksRepository::init(PostgresBooksRepositoryConfig {
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
    #[serial]
    /// Tests if add_book and get_book work correctly
    /// for the sake of not starting container multiple times it tests everything in one testcase
    async fn test_add_book_and_get_it() {
        let (_container, repo) = start_postgres_container_and_init_repo().await;

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
    #[serial]
    /// Tests if list_books works correctly
    /// for the sake of not starting container multiple times it tests everything in one testcase
    async fn test_add_books_and_list_them() {
        let (_container, repo) = start_postgres_container_and_init_repo().await;

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

        let list = repo.list_books().await.expect("Failed to list books");

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
    #[serial]
    /// Tests if add_book and get_book work correctly
    /// for the sake of not starting container multiple times it tests everything in one testcase
    async fn test_add_book_patch_and_get_it() {
        let (_container, repo) = start_postgres_container_and_init_repo().await;
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
