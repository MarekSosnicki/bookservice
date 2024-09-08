use std::collections::HashMap;
use std::sync::Arc;

use futures_util::stream::StreamExt;
use itertools::Itertools;
use opentelemetry_sdk::util::tokio_interval_stream;
use parking_lot::{Mutex, RwLock};

use bookservice_repository::api::BookDetails;
use bookservice_repository::client::BookServiceRepositoryClient;
use bookservice_reservations::api::{BookId, ReservationHistoryRecord, UserId};
use bookservice_reservations::client::BookServiceReservationsClient;

use crate::api::Recommendations;
use crate::recommendations::{CoefficientsStorage, RecommendationsEngine};

const INTERVAL_SECONDS: u64 = 10;
const UPDATE_ALL_BOOK_DETAILS_EVERY_INTERVAL: i32 = 200;
const USERS_SPLIT: i32 = 10;

#[derive(Clone)]
pub struct RecommendationsProvider {
    recommendations_engine: Arc<RwLock<RecommendationsEngine>>,
}

impl RecommendationsProvider {
    pub fn get_recommendations_for_user(&self, user_id: UserId) -> Recommendations {
        self.recommendations_engine
            .read()
            .get_recommendations_for_user(user_id)
    }
}

pub struct RecommendationsUpdater {
    coefficients_storage: Arc<Mutex<CoefficientsStorage>>,
    recommendations_engine: Arc<RwLock<RecommendationsEngine>>,
    book_service_repository_client: BookServiceRepositoryClient,
    book_service_reservations_client: BookServiceReservationsClient,
}

impl RecommendationsUpdater {
    pub fn new(
        book_service_repository_url: &str,
        book_service_reservations_url: &str,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            coefficients_storage: Arc::new(Default::default()),
            recommendations_engine: Arc::new(Default::default()),
            book_service_repository_client: BookServiceRepositoryClient::new(
                book_service_repository_url,
            )?,
            book_service_reservations_client: BookServiceReservationsClient::new(
                book_service_reservations_url,
            )?,
        })
    }
    pub fn provider(&self) -> RecommendationsProvider {
        RecommendationsProvider {
            recommendations_engine: self.recommendations_engine.clone(),
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let mut periodic_updater =
            tokio_interval_stream(std::time::Duration::from_secs(INTERVAL_SECONDS));
        let mut interval_no = 0;
        let mut processed_users_to_last_updated: HashMap<UserId, std::time::Instant> =
            Default::default();

        while periodic_updater.next().await.is_some() {
            tracing::info!("Recommendations tick no {}", interval_no);

            // Every tick get all users
            let user_ids = self.book_service_reservations_client.list_users().await?;

            // Process the users that were never processed every tick
            let mut users_to_process = user_ids
                .iter()
                .filter(|uid| !processed_users_to_last_updated.contains_key(uid))
                .cloned()
                .collect_vec();

            // In constant intervals, take a group of users and update them
            if interval_no % (UPDATE_ALL_BOOK_DETAILS_EVERY_INTERVAL / USERS_SPLIT) == 0 {
                let user_modulo =
                    interval_no / (UPDATE_ALL_BOOK_DETAILS_EVERY_INTERVAL / USERS_SPLIT);
                tracing::info!("Processing users modulo {}", user_modulo);
                for user_id in processed_users_to_last_updated.keys() {
                    if user_id & user_modulo == 0 {
                        users_to_process.push(*user_id);
                    }
                }
            }
            let (user_id_to_reservations, user_id_to_history) =
                self.fetch_user_reservations_data(users_to_process).await?;

            // Every UPDATE_ALL_BOOK_DETAILS_EVERY_INTERVAL ticks process all books
            let book_ids_to_process = if interval_no == 0 {
                self.book_service_repository_client
                    .list_books()
                    .await?
                    .into_iter()
                    .map(|id_and_title| id_and_title.book_id)
                    .collect_vec()
            } else {
                // Otherwise process only books from user reservations and history
                user_id_to_reservations
                    .values()
                    .flatten()
                    .cloned()
                    .chain(user_id_to_history.values().flatten().map(|r| r.book_id))
                    .unique()
                    .collect_vec()
            };

            let mut book_id_to_details: HashMap<BookId, BookDetails> = Default::default();
            for book_id in book_ids_to_process {
                if let Some(details) = self
                    .book_service_repository_client
                    .get_book(book_id)
                    .await?
                {
                    book_id_to_details.insert(book_id, details);
                } else {
                    tracing::warn!("Failed to get details for book {}", book_id);
                }
            }

            self.update(
                &user_id_to_reservations,
                &user_id_to_history,
                &book_id_to_details,
            )
            .await?;

            let now = std::time::Instant::now();
            for (user_id, _) in user_id_to_reservations.iter() {
                processed_users_to_last_updated.insert(*user_id, now);
            }

            interval_no = (interval_no + 1) % UPDATE_ALL_BOOK_DETAILS_EVERY_INTERVAL;
        }
        Ok(())
    }

    async fn fetch_user_reservations_data(
        &self,
        user_ids: Vec<UserId>,
    ) -> anyhow::Result<(
        HashMap<UserId, Vec<BookId>>,
        HashMap<UserId, Vec<ReservationHistoryRecord>>,
    )> {
        let mut user_id_to_reservations: HashMap<UserId, Vec<BookId>> = Default::default();
        let mut user_id_to_history: HashMap<UserId, Vec<ReservationHistoryRecord>> =
            Default::default();
        for user_id in user_ids {
            let history = self
                .book_service_reservations_client
                .history(user_id)
                .await?;
            user_id_to_history.insert(user_id, history);
            let reservations = self
                .book_service_reservations_client
                .list_reservations(user_id)
                .await?;
            user_id_to_reservations.insert(user_id, reservations);
        }
        Ok((user_id_to_reservations, user_id_to_history))
    }

    async fn update(
        &self,
        user_id_to_reservations: &HashMap<UserId, Vec<BookId>>,
        user_id_to_history: &HashMap<UserId, Vec<ReservationHistoryRecord>>,
        book_id_to_details: &HashMap<BookId, BookDetails>,
    ) -> anyhow::Result<()> {
        let mut storage = self.coefficients_storage.lock();
        storage.update_storage(user_id_to_history, book_id_to_details)?;

        self.recommendations_engine
            .write()
            .update_recommendations_for_users(
                &storage,
                user_id_to_reservations,
                user_id_to_history,
            )?;
        Ok(())
    }
}
