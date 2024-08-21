use anyhow::{bail, Context};
use reqwest::header::LOCATION;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;

use crate::api::{BookDetails, BookDetailsPatch, BookId, BookTitleAndId};

pub struct BookServiceRepositoryClient {
    url: String,
    client: ClientWithMiddleware,
}

impl BookServiceRepositoryClient {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let reqwest_client = reqwest::Client::builder()
            .build()
            .context("Failed to build reqwest client")?;
        let client = ClientBuilder::new(reqwest_client)
            // Insert the tracing middleware
            .with(TracingMiddleware::default())
            .build();

        Ok(Self {
            url: url.to_string(),
            client,
        })
    }

    /// Calls POST /api/book endpoint
    /// Returns book_id of added book in response
    pub async fn add_book(&self, book_details: BookDetails) -> anyhow::Result<BookId> {
        let response = self
            .client
            .post(format!("{}/api/book", self.url))
            .json(&book_details)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to add book {}", error)
        }

        let location_header = response
            .headers()
            .get(LOCATION)
            .context("No location header")?;

        location_header
            .to_str()
            .context("Failed to convert header to str")?
            .strip_prefix("/api/book/")
            .context("Invalid location header")?
            .parse()
            .context("Failed to parse book id")
    }

    /// Calls GET /api/book/{book_id} endpoint
    /// Returns book details if book was present
    /// None if book was not in the repository
    /// and error in case of any other failure
    pub async fn get_book(&self, book_id: BookId) -> anyhow::Result<Option<BookDetails>> {
        let response = self
            .client
            .get(format!("{}/api/book/{}", self.url, book_id))
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else if response.status().is_success() {
            Ok(Some(response.json().await?))
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to get book {}", error)
        }
    }

    /// Calls PATCH /api/book/{book_id} endpoint
    pub async fn update_book(
        &self,
        book_id: BookId,
        patch: BookDetailsPatch,
    ) -> anyhow::Result<()> {
        let response = self
            .client
            .patch(format!("{}/api/book/{}", self.url, book_id))
            .json(&patch)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to update book {}", error)
        }
    }

    /// Calls GET /api/books endpoint
    pub async fn list_books(&self) -> anyhow::Result<Vec<BookTitleAndId>> {
        let response = self
            .client
            .get(format!("{}/api/books", self.url))
            .send()
            .await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to list books {}", error)
        }
    }
}
