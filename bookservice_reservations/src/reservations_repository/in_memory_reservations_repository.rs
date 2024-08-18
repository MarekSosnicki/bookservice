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

impl Default for InMemoryReservationsRepository {
    fn default() -> Self {
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

#[cfg(test)]
mod tests_in_memory_reservations_repository {
    use super::*;

    #[tokio::test]
    /// Simple test to cover user management
    /// Combined into big unit test to avoid duplicate setup
    /// 1. Gets all users -expects empty
    /// 2. Creates user
    /// 3. Gets user
    /// 4. Gets all users - expects 1
    /// 5. Creates second user
    /// 6. Gets all users - expects 2
    /// 7. Gets user not existing in db to get not found
    async fn test_user_management() {
        let repository = InMemoryReservationsRepository::default();
        assert_eq!(
            repository.get_all_user_ids().await.unwrap(),
            Vec::<UserId>::default()
        );

        let user_details = UserDetails {
            username: "username".to_string(),
            favourite_tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let user_id = repository.add_user(user_details.clone()).await.unwrap();

        let user_returned = repository.get_user(user_id).await.unwrap();

        assert_eq!(user_returned, user_details);
        assert_eq!(repository.get_all_user_ids().await.unwrap(), vec![user_id]);

        let user_2_id = repository
            .add_user(UserDetails {
                username: "user2".to_string(),
                favourite_tags: vec![],
            })
            .await
            .unwrap();
        let mut all_users = repository.get_all_user_ids().await.unwrap();
        all_users.sort();
        assert_eq!(all_users, vec![user_id, user_2_id]);

        let unknown_user_id = user_2_id + 1;

        let get_unknown_user_result = repository.get_user(unknown_user_id).await;
        assert!(matches!(
            get_unknown_user_result,
            Err(ReservationsRepositoryError::UserNotFound(unknown_user_id))
        ));
    }

    #[tokio::test]
    /// Simple test to cover reservation management
    /// Combined into big unit test to avoid duplicate setup
    /// 1.Creates two users, validates reservations and history is empty
    /// 2.Reserves book
    /// 3.Lists all reservations for user
    /// 4.Creates second user
    /// 5.Tries to reserve the same book - get rejected
    /// 6.Releases reservation for the first user
    /// 7.Lists reservations
    async fn test_reservation_management() {
        let repository = InMemoryReservationsRepository::default();

        let user_1_id = repository
            .add_user(UserDetails {
                username: "user1".to_string(),
                favourite_tags: vec![],
            })
            .await
            .unwrap();
        let user_2_id = repository
            .add_user(UserDetails {
                username: "user1".to_string(),
                favourite_tags: vec![],
            })
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            Vec::<BookId>::default()
        );

        assert_eq!(
            repository
                .get_reservations_history(user_1_id)
                .await
                .unwrap(),
            Vec::<ReservationHistoryRecord>::default()
        );

        let test_book_id: BookId = 1;

        // reserve book for the user
        repository
            .reserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            vec![test_book_id]
        );
        assert_eq!(
            repository
                .get_reservations_history(user_1_id)
                .await
                .unwrap(),
            Vec::<ReservationHistoryRecord>::default()
        );

        let reserve_conflict = repository.reserve_book(user_2_id, test_book_id).await;

        assert!(matches!(
            reserve_conflict,
            Err(ReservationsRepositoryError::BookAlreadyReserved(..))
        ));

        // unreserve book for wrong user
        let unreserve_invalid_user = repository.unreserve_book(user_2_id, test_book_id).await;

        assert!(matches!(
            unreserve_invalid_user,
            Err(ReservationsRepositoryError::BookReservedByDifferentUser(..))
        ));

        // unreserve book for right user
        repository
            .unreserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            Vec::<BookId>::default()
        );

        let history = repository
            .get_reservations_history(user_1_id)
            .await
            .unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].book_id, test_book_id);

        // reserve again to check if properly release
        repository
            .reserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        // Reserve other book
        let other_book_id: BookId = 10;
        repository
            .reserve_book(user_1_id, other_book_id)
            .await
            .unwrap();

        let mut two_reservations = repository.get_all_reservations(user_1_id).await.unwrap();
        two_reservations.sort();
        assert_eq!(two_reservations, vec![test_book_id, other_book_id]);

        // Unreserve to see if can have the same book twice separately
        repository
            .unreserve_book(user_1_id, test_book_id)
            .await
            .unwrap();

        assert_eq!(
            repository.get_all_reservations(user_1_id).await.unwrap(),
            vec![other_book_id]
        );

        let history = repository
            .get_reservations_history(user_1_id)
            .await
            .unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].book_id, test_book_id);
        assert_eq!(history[1].book_id, test_book_id);
    }
}
