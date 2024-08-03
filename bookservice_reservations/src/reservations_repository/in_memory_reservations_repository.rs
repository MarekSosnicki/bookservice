use std::collections::HashMap;
use std::sync::atomic::AtomicI32;

use crate::api::ReservationHistoryRecord;
use crate::reservations_repository::{
    BookId, ReservationsRepository, ReservationsRepositoryError, UserDetails, UserId, UsernameAndId,
};

pub struct InMemoryReservationsRepository {
    users: parking_lot::Mutex<HashMap<UserId, UserDetails>>,
    reservations: parking_lot::Mutex<HashMap<UserId, BookId>>,
    history: parking_lot::Mutex<HashMap<UserId, (BookId, u64)>>,
    user_sequence_generator: AtomicI32,
}

impl InMemoryReservationsRepository {
    pub fn new() -> Self {
        Self {
            users: Default::default(),
            reservations: Default::default(),
            history: Default::default(),
            user_sequence_generator: Default::default(),
        }
    }
}
#[async_trait::async_trait]
impl ReservationsRepository for InMemoryReservationsRepository {
    async fn add_user(
        &self,
        user_data: UserDetails,
    ) -> Result<UserId, ReservationsRepositoryError> {
        todo!()
    }

    async fn get_user(&self, id: UserId) -> Result<UserDetails, ReservationsRepositoryError> {
        todo!()
    }

    async fn get_all_users(
        &self,
        username: String,
    ) -> Result<Vec<UsernameAndId>, ReservationsRepositoryError> {
        todo!()
    }

    async fn reserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        todo!()
    }

    async fn unreserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        todo!()
    }

    async fn get_all_reservations(
        &self,
        user_id: UserId,
    ) -> Result<Vec<BookId>, ReservationsRepositoryError> {
        todo!()
    }

    async fn get_reservations_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<ReservationHistoryRecord>, ReservationsRepositoryError> {
        todo!()
    }
}
