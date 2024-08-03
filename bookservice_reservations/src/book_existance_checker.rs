use anyhow::Context;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;

use crate::api::BookId;

pub struct BookExistanceChecker {
    book_repository_url: String,
}

impl BookExistanceChecker {
    pub fn new(book_repository_url: String) -> Self {
        Self {
            book_repository_url,
        }
    }

    pub async fn check_book_existance(&self, book_id: BookId) -> anyhow::Result<bool> {
        let reqwest_client = reqwest::Client::builder()
            .build()
            .context("Failed to build reqwest client")?;
        let client = ClientBuilder::new(reqwest_client)
            // Insert the tracing middleware
            .with(TracingMiddleware::default())
            .build();

        let response = client
            .get(&format!(
                "{}/api/book/{}",
                self.book_repository_url, book_id
            ))
            .send()
            .await
            .context("Failed to get book by id")?;

        Ok(response.status().is_success())
    }
}
