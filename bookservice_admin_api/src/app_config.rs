use actix_web::web;
use crate::handlers;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("health")
            .route(web::get().to(handlers::health))
    );
}
