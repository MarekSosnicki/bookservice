use actix_web::Error;
use actix_web::HttpResponse;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use bookservice_reservations::api::UserId;

use crate::api::Recommendations;
use crate::recommendations_updater::RecommendationsProvider;

#[api_v2_operation]
pub async fn health() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[api_v2_operation]
pub async fn get_recommendations_for_user(
    recommendations_provider: web::Data<RecommendationsProvider>,
    user_id: web::Path<UserId>,
) -> Result<Json<Recommendations>, Error> {
    Ok(Json(
        recommendations_provider.get_recommendations_for_user(user_id.into_inner()),
    ))
}

#[cfg(test)]
mod handler_tests {
    // TODO: Add client for api
    // TODO: Add tests for handler
}
