use actix_web::{HttpResponse};
use actix_web::http::Error;

pub async fn health() -> Result<HttpResponse, Error> {
    println!("Healthy!");
    Ok(HttpResponse::Ok().finish())
}
