use anyhow::{bail, Context};
use reqwest::header::LOCATION;
use reqwest::StatusCode;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;

use crate::api::{BookId, ReservationHistoryRecord, UserDetails, UserId};

pub struct BookServiceReservationsClient {
    url: String,
    client: ClientWithMiddleware,
}

impl BookServiceReservationsClient {
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

    /// Calls POST /api/user endpoint
    /// Returns user_id of added user in response
    pub async fn add_user(&self, user_details: UserDetails) -> anyhow::Result<UserId> {
        let response = self
            .client
            .post(format!("{}/api/user", self.url))
            .json(&user_details)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to add user {}", error)
        }

        let location_header = response
            .headers()
            .get(LOCATION)
            .context("No location header")?;

        location_header
            .to_str()
            .context("Failed to convert header to str")?
            .strip_prefix("/api/user/")
            .context("Invalid location header")?
            .parse()
            .context("Failed to parse user id")
    }

    /// Calls GET /api/user/{user_id} endpoint
    /// Returns user details if user was present
    /// None if user was not in the repository
    /// and error in case of any other failure
    pub async fn get_user(&self, user_id: UserId) -> anyhow::Result<Option<UserDetails>> {
        let response = self
            .client
            .get(format!("{}/api/user/{}", self.url, user_id))
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else if response.status().is_success() {
            Ok(Some(response.json().await?))
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to get user {}", error)
        }
    }

    /// Calls GET /api/users endpoint
    pub async fn list_users(&self) -> anyhow::Result<Vec<UserId>> {
        let response = self
            .client
            .get(format!("{}/api/users", self.url))
            .send()
            .await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to list books {}", error)
        }
    }

    /// Calls POST /api/user/{user_id}/reservation/{book_id} endpoint
    /// Returns true if successful and false if failed to reserve
    pub async fn reserve_book(&self, book_id: BookId, user_id: UserId) -> anyhow::Result<bool> {
        let response = self
            .client
            .post(format!(
                "{}/api/user/{}/reservation/{}",
                self.url, user_id, book_id
            ))
            .send()
            .await?;

        if response.status() == StatusCode::FORBIDDEN {
            Ok(false)
        } else if response.status().is_success() {
            Ok(true)
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to get user {}", error)
        }
    }

    /// Calls DELETE /api/user/{user_id}/reservation/{book_id} endpoint
    /// Returns true if successful and false if failed to unreserve
    pub async fn unreserve_book(&self, book_id: BookId, user_id: UserId) -> anyhow::Result<bool> {
        let response = self
            .client
            .delete(format!(
                "{}/api/user/{}/reservation/{}",
                self.url, user_id, book_id
            ))
            .send()
            .await?;

        if response.status() == StatusCode::FORBIDDEN {
            Ok(false)
        } else if response.status().is_success() {
            Ok(true)
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to get user {}", error)
        }
    }

    /// Calls GET /api/user/{user_id}/reservations endpoint
    pub async fn list_reservations(&self, user_id: UserId) -> anyhow::Result<Vec<BookId>> {
        let response = self
            .client
            .get(format!("{}/api/user/{}/reservations", self.url, user_id))
            .send()
            .await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: String = response.json().await.unwrap_or_default();
            bail!("Failed to list books {}", error)
        }
    }

    /// Calls GET /api/user/{user_id}/history endpoint
    pub async fn history(&self, user_id: UserId) -> anyhow::Result<Vec<ReservationHistoryRecord>> {
        let response = self
            .client
            .get(format!("{}/api/user/{}/history", self.url, user_id))
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
