use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

use bookservice_reservations::api::BookId;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq, Apiv2Schema)]
/// A set of book recommendations for user, contains only book ids that were never reserved by user before
pub struct Recommendations {
    /// Up to 4 most popular books that were not yet reserved by user
    pub most_popular: Vec<BookId>,
    /// Up to 4 most popular books of the author that the user already reserved a book from
    /// The priority is to take books of different authors
    pub author_match: Vec<BookId>,
    /// Up to 4 most popular book of the authors with the highest comparison score and never reserved before by the user
    pub new_author_match: Vec<BookId>,
    // TODO: Add tag match
    // /// Among the books with similar tag matching score, up to 4 will be randomly selected
    // pub wild_tags_matches: Vec<BookId>,
}
