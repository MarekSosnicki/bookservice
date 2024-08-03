use std::sync::Arc;

use actix_web::Error;
use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use actix_web::web::Data;
use paperclip::actix::{
    api_v2_operation,
    web::{self},
};

use crate::api::{BookDetails, BookDetailsPatch, BookId, GetAllBooksResponse};
use crate::books_repository::{BookRepository, BookRepositoryError};

#[api_v2_operation]
pub async fn health() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn get_all_books(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
) -> Result<HttpResponse, Error> {
    Ok(match books_repository.list_books().await {
        Ok(books) => HttpResponse::Ok().json(GetAllBooksResponse { books }),
        Err(err) => {
            tracing::error!("Get all books failed {}", err);
            HttpResponse::InternalServerError().finish()
        }
    })
}

#[api_v2_operation]
pub async fn add_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    details: web::Json<BookDetails>,
) -> Result<HttpResponse, Error> {
    Ok(
        match books_repository.add_book(details.into_inner()).await {
            Ok(book_id) => HttpResponse::Ok()
                .append_header((LOCATION, format!("/api/book/{}", book_id)))
                .finish(),
            Err(err) => {
                tracing::error!("Add book failed {}", err);
                HttpResponse::InternalServerError().finish()
            }
        },
    )
}

#[api_v2_operation]
pub async fn update_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    book_id: web::Path<BookId>,
    patch: web::Json<BookDetailsPatch>,
) -> Result<HttpResponse, Error> {
    Ok(
        match books_repository
            .update_book(book_id.into_inner(), patch.into_inner())
            .await
        {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(BookRepositoryError::NotFound(_)) => HttpResponse::NotFound().finish(),
            Err(err) => {
                tracing::error!("Update book failed {}", err);
                HttpResponse::InternalServerError().finish()
            }
        },
    )
}

#[api_v2_operation]
pub async fn get_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    book_id: web::Path<BookId>,
) -> Result<HttpResponse, Error> {
    Ok(
        match books_repository.get_book(book_id.into_inner()).await {
            Ok(book_details) => HttpResponse::Ok().json(book_details),
            Err(BookRepositoryError::NotFound(_)) => HttpResponse::NotFound().finish(),
            Err(err) => {
                tracing::error!("Get book failed {}", err);
                HttpResponse::InternalServerError().finish()
            }
        },
    )
}

#[cfg(test)]
mod handler_tests {
    // TODO: Add client for api
    // TODO: Add tests for handler
}
