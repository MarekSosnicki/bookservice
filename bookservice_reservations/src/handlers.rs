use actix_web::{Error, HttpResponse};
use paperclip::actix::api_v2_operation;

#[api_v2_operation]
pub async fn health() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn get_all_users() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn add_user() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn get_all_reservations() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn get_reservations_history() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn reserve_book() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn unreserve_book() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}
