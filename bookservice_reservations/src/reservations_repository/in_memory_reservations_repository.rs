use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::UNIX_EPOCH;

use crate::api::ReservationHistoryRecord;
use crate::reservations_repository::{
    BookId, ReservationsRepository, ReservationsRepositoryError, UserDetails, UserId,
};

pub struct InMemoryReservationsRepository {
    users: parking_lot::RwLock<HashMap<UserId, UserDetails>>,
    reservations: parking_lot::RwLock<HashMap<BookId, UserId>>,
    history: parking_lot::RwLock<HashMap<UserId, Vec<ReservationHistoryRecord>>>,
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
        let id = self.user_sequence_generator.fetch_add(1, Ordering::Relaxed);
        self.users.write().insert(id, user_data);
        Ok(id)
    }

    async fn get_user(&self, id: UserId) -> Result<UserDetails, ReservationsRepositoryError> {
        let locked_users = self.users.read();

        locked_users
            .get(&id)
            .cloned()
            .ok_or_else(|| ReservationsRepositoryError::UserNotFound(id))
    }

    async fn get_all_user_ids(&self) -> Result<Vec<UserId>, ReservationsRepositoryError> {
        Ok(self.users.read().keys().cloned().collect())
    }

    async fn reserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        let mut reservations_lock = self.reservations.write();

        match reservations_lock.entry(book_id) {
            Entry::Occupied(_) => Err(ReservationsRepositoryError::BookAlreadyReserved(book_id)),
            Entry::Vacant(entry) => {
                entry.insert(user_id);
                Ok(())
            }
        }
    }

    async fn unreserve_book(
        &self,
        user_id: UserId,
        book_id: BookId,
    ) -> Result<(), ReservationsRepositoryError> {
        let mut reservations_lock = self.reservations.write();

        match reservations_lock.entry(book_id) {
            Entry::Occupied(occupied) => {
                if occupied.get() == &user_id {
                    occupied.remove();
                    self.history.write().entry(user_id).or_default().push(
                        ReservationHistoryRecord {
                            book_id,
                            unreserved_at: std::time::SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        },
                    );
                    Ok(())
                } else {
                    Err(ReservationsRepositoryError::BookReservedByDifferentUser(
                        book_id,
                    ))
                }
            }
            Entry::Vacant(_) => Err(ReservationsRepositoryError::BookNotReserved(book_id)),
        }
    }

    async fn get_all_reservations(
        &self,
        user_id: UserId,
    ) -> Result<Vec<BookId>, ReservationsRepositoryError> {
        Ok(self
            .reservations
            .read()
            .iter()
            .filter(|(_, &uid)| user_id == uid)
            .map(|(book_id, _)| *book_id)
            .collect())
    }

    async fn get_reservations_history(
        &self,
        user_id: UserId,
    ) -> Result<Vec<ReservationHistoryRecord>, ReservationsRepositoryError> {
        Ok(self
            .history
            .read()
            .get(&user_id)
            .cloned()
            .unwrap_or_default())
    }
}
