use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

pub type UserId = i32;
pub type BookId = i32;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Apiv2Schema)]
pub struct UserDetails {
    pub username: String,
    pub favourite_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Apiv2Schema)]
pub struct ReservationHistoryRecord {
    pub book_id: BookId,
    pub unreserved_at: i64,
}
