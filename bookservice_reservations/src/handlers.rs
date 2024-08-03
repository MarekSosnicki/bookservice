use std::sync::Arc;

use actix_web::{Error, HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::header::LOCATION;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Data},
};

use crate::api::{BookId, ReservationHistoryRecord, UserDetails, UserId};
use crate::book_existance_checker::BookExistanceChecker;
use crate::reservations_repository::{ReservationsRepository, ReservationsRepositoryError};

impl ResponseError for ReservationsRepositoryError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            ReservationsRepositoryError::UserNotFound(book_id) => {
                HttpResponse::NotFound().body(format!("Book not found {}", book_id))
            }
            ReservationsRepositoryError::BookNotReserved(book_id) => {
                HttpResponse::Forbidden().body(format!("Book already reserved {}", book_id))
            }
            ReservationsRepositoryError::BookReservedByDifferentUser(book_id) => {
                HttpResponse::Forbidden()
                    .body(format!("Book is reserved {} by different user", book_id))
            }
            _ => HttpResponse::InternalServerError().body(self.to_string()),
        }
    }
}

#[api_v2_operation]
pub async fn health() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn add_user(
    reservations_repository: Data<Arc<dyn ReservationsRepository>>,
    details: web::Json<UserDetails>,
) -> Result<HttpResponse, Error> {
    let user_id = reservations_repository
        .add_user(details.into_inner())
        .await?;
    Ok(HttpResponse::Ok()
        .append_header((LOCATION, format!("/api/user/{}", user_id)))
        .finish())
}

#[api_v2_operation]
pub async fn get_user(
    reservations_repository: Data<Arc<dyn ReservationsRepository>>,
    user_id: web::Path<UserId>,
) -> Result<web::Json<UserDetails>, Error> {
    Ok(web::Json(
        reservations_repository
            .get_user(user_id.into_inner())
            .await?,
    ))
}

#[api_v2_operation]
pub async fn get_all_users(
    reservations_repository: Data<Arc<dyn ReservationsRepository>>,
) -> Result<web::Json<Vec<UserId>>, Error> {
    Ok(web::Json(reservations_repository.get_all_user_ids().await?))
}

#[api_v2_operation]
pub async fn get_all_reservations(
    reservations_repository: Data<Arc<dyn ReservationsRepository>>,
    user_id: web::Path<UserId>,
) -> Result<web::Json<Vec<BookId>>, Error> {
    Ok(web::Json(
        reservations_repository
            .get_all_reservations(user_id.into_inner())
            .await?,
    ))
}

#[api_v2_operation]
pub async fn get_reservations_history(
    reservations_repository: Data<Arc<dyn ReservationsRepository>>,
    user_id: web::Path<UserId>,
) -> Result<web::Json<Vec<ReservationHistoryRecord>>, Error> {
    Ok(web::Json(
        reservations_repository
            .get_reservations_history(user_id.into_inner())
            .await?,
    ))
}

#[api_v2_operation]
pub async fn reserve_book(
    book_existance_checker: Data<BookExistanceChecker>,
    reservations_repository: Data<Arc<dyn ReservationsRepository>>,
    user_and_book_id: web::Path<(UserId, BookId)>,
) -> Result<HttpResponse, Error> {
    let (user_id, book_id) = user_and_book_id.into_inner();

    let book_exists = book_existance_checker
        .check_book_existance(book_id)
        .await
        .map_err(|err| ReservationsRepositoryError::Other(err.to_string()))?;

    if book_exists {
        reservations_repository
            .reserve_book(user_id, book_id)
            .await?;
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::NotFound().body(format!("Book not found {}", book_id)))
    }
}

#[api_v2_operation]
pub async fn unreserve_book(
    reservations_repository: Data<Arc<dyn ReservationsRepository>>,
    user_and_book_id: web::Path<(UserId, BookId)>,
) -> Result<HttpResponse, Error> {
    let (user_id, book_id) = user_and_book_id.into_inner();
    reservations_repository
        .unreserve_book(user_id, book_id)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
