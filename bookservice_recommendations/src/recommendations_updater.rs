use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use bookservice_repository::api::BookDetails;
use bookservice_repository::client::BookServiceRepositoryClient;
use bookservice_reservations::api::{BookId, ReservationHistoryRecord, UserId};
use bookservice_reservations::client::BookServiceReservationsClient;

use crate::api::Recommendations;
use crate::recommendations::{CoefficientsStorage, RecommendationsEngine};

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

    pub async fn start(&self) -> anyhow::Result<()> {
        let user_ids = self.book_service_reservations_client.list_users().await?;
        let book_ids = self.book_service_repository_client.list_books().await?;

        let mut book_id_to_details: HashMap<BookId, BookDetails> = Default::default();
        for book_id in book_ids {
            if let Some(details) = self
                .book_service_repository_client
                .get_book(book_id.book_id)
                .await?
            {
                book_id_to_details.insert(book_id.book_id, details);
            } else {
                tracing::warn!("Failed to get details for book {}", book_id.book_id);
            }
        }

        let mut user_id_to_reservations: HashMap<BookId, Vec<BookId>> = Default::default();
        let mut user_id_to_history: HashMap<BookId, Vec<ReservationHistoryRecord>> =
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
        {
            let mut storage = self.coefficients_storage.lock();
            storage.update_storage(&user_id_to_history, &book_id_to_details)?;

            self.recommendations_engine
                .write()
                .update_recommendations_for_users(
                    &storage,
                    &user_id_to_reservations,
                    &user_id_to_history,
                )?;
        }

        Ok(())
    }
}
