use paperclip::actix::web;

use crate::handlers;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(handlers::health)))
        .service(
            web::scope("/api").service(
                web::resource("/recommendations/{user_id}")
                    .route(web::get().to(handlers::get_recommendations_for_user)),
            ),
        );
}
