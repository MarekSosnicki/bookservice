use actix_web::{App, HttpServer};
use bookservice_admin_api::app_config::config_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| App::new().configure(config_app))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
