use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

pub type BookId = i32;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Apiv2Schema)]
/// Struct containing book id and title
pub struct BookTitleAndId {
    pub book_id: BookId,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Apiv2Schema)]
/// Struct representing details of the book
pub struct BookDetails {
    pub title: String,
    pub authors: Vec<String>,
    pub publisher: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Apiv2Schema)]
/// Struct representing a patch to book details. Allows to specify only a few fields and patch the current details
pub struct BookDetailsPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}
