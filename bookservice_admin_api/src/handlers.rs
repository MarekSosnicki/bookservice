use std::sync::Arc;

use actix_web::{HttpResponse, web};
use actix_web::http::Error;
use actix_web::web::Data;

use crate::api::{BookDetails, BookDetailsPatch, BookId, GetAllBooksResponse};
use crate::books_repository::{BookRepository, BookRepositoryError};

pub async fn health() -> Result<HttpResponse, Error> {
    println!("Healthy!");
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_all_books(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
) -> HttpResponse {
    println!("GET ALL BOOKS");
    match books_repository.list_books() {
        Ok(books) => HttpResponse::Ok().json(GetAllBooksResponse { books }),
        Err(err) => {
            tracing::error!("Get all books failed {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    details: web::Json<BookDetails>,
) -> HttpResponse {
    match books_repository.add_book(details.into_inner()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => {
            tracing::error!("Add book failed {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    book_id: web::Path<BookId>,
    patch: web::Json<BookDetailsPatch>,
) -> HttpResponse {
    match books_repository.update_book(book_id.into_inner(), patch.into_inner()) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(BookRepositoryError::NotFound(_)) => HttpResponse::NotFound().finish(),
        Err(err) => {
            tracing::error!("Update book failed {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_book(
    books_repository: Data<Arc<dyn BookRepository + Send + Sync>>,
    book_id: web::Path<BookId>,
) -> HttpResponse {
    match books_repository.get_book(book_id.into_inner()) {
        Ok(book_details) => HttpResponse::Ok().json(book_details),
        Err(BookRepositoryError::NotFound(_)) => HttpResponse::NotFound().finish(),
        Err(err) => {
            tracing::error!("Get book failed {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
