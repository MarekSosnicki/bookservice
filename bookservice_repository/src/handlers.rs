use std::sync::Arc;

use actix_web::{Error, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use actix_web::web::Data;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use crate::api::{BookDetails, BookDetailsPatch, BookId, BookTitleAndId};
use crate::books_repository::{BookRepository, BookRepositoryError};

#[api_v2_operation]
pub async fn health() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

impl ResponseError for BookRepositoryError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            BookRepositoryError::NotFound(book_id) => {
                HttpResponse::NotFound().body(format!("Book not found {}", book_id))
            }
            _ => HttpResponse::InternalServerError().body(self.to_string()),
        }
    }
}

#[api_v2_operation]
pub async fn get_all_books(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
) -> Result<Json<Vec<BookTitleAndId>>, Error> {
    Ok(Json(books_repository.list_books().await?))
}

#[api_v2_operation]
pub async fn add_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    details: web::Json<BookDetails>,
) -> Result<HttpResponse, Error> {
    let book_id = books_repository.add_book(details.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .append_header((LOCATION, format!("/api/book/{}", book_id)))
        .finish())
}

#[api_v2_operation]
pub async fn update_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    book_id: web::Path<BookId>,
    patch: web::Json<BookDetailsPatch>,
) -> Result<HttpResponse, Error> {
    books_repository
        .update_book(book_id.into_inner(), patch.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn get_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    book_id: web::Path<BookId>,
) -> Result<web::Json<BookDetails>, Error> {
    Ok(Json(books_repository.get_book(book_id.into_inner()).await?))
}

#[cfg(test)]
mod handler_tests {
    // TODO: Add client for api
    // TODO: Add tests for handler
}
