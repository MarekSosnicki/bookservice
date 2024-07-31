use actix_web::http::Error;
use actix_web::HttpResponse;

pub async fn health() -> Result<HttpResponse, Error> {
    println!("Healthy!");
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_all_books() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn add_book() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn update_book() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_book() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}
