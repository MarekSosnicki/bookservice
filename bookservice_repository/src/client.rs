use anyhow::{bail, Context};
use reqwest::header::LOCATION;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;

use crate::api::{BookDetails, BookId};

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
}
