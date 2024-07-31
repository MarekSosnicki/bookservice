use actix_web::web;

use crate::handlers;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("health").route(web::get().to(handlers::health)))
        .service(
            web::scope("/api")
                .service(
                    web::resource("books")
                        .route(web::get())
                        .to(handlers::get_all_books),
                )
                .service(
                    web::scope("/{book_id}").service(
                        web::resource("")
                            .route(web::get().to(handlers::get_book))
                            .route(web::post().to(handlers::add_book))
                            .route(web::patch().to(handlers::update_book)),
                    ),
                ),
        );
}
