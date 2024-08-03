use paperclip::actix::web;

use crate::handlers;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(handlers::health)))
        .service(
            web::scope("/api")
                .service(web::resource("/users").route(web::get().to(handlers::get_all_users)))
                .service(
                    web::scope("/user")
                        .service(web::resource("").route(web::post().to(handlers::add_user)))
                        .service(
                            web::scope("/{user_id}")
                                .service(
                                    web::resource("/reservations")
                                        .route(web::get().to(handlers::get_all_reservations)),
                                )
                                .service(
                                    web::resource("/history")
                                        .route(web::post().to(handlers::get_reservations_history)),
                                )
                                .service(
                                    web::resource("/{book_id}")
                                        .route(web::post().to(handlers::reserve_book))
                                        .route(web::delete().to(handlers::unreserve_book)),
                                ),
                        ),
                ),
        );
}
