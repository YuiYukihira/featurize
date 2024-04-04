use actix_web::{get, web, HttpResponse, HttpResponseBuilder, Responder};
use sentry::{Hub, SentryFutureExt};
use serde::Deserialize;
use tera::Tera;

use crate::{kratos_client::{ErrorsRequest, KratosClient}, renderer::{self, Renderer}, Error};

#[derive(Deserialize, Debug)]
pub struct ErrorQuery {
    id: String,
}

#[derive(Deserialize)]
pub struct AuthError {
    error: ErrorMessage
}

#[derive(Deserialize)]
pub struct ErrorMessage {
    code: u16,
    message: String,
    reason: String,
}

#[tracing::instrument]
#[get("/error")]
pub async fn route(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, query: web::Query<ErrorQuery>) -> Result<HttpResponse, Error> {
    handler(renderer, kratos, query).bind_hub(Hub::current()).await
}

#[tracing::instrument]
pub async fn handler(renderer: web::Data<Renderer>, auth_config: web::Data<KratosClient>, query: web::Query<ErrorQuery>) -> Result<HttpResponse, Error> {
    let error = auth_config.new_request(ErrorsRequest(query.id.clone()))
        .send()
        .await?;

    let html = renderer
        .render("error.html")
        .var("msg", &error.body.error.message)
        .var("reason", &error.body.error.reason)
        .finish()?;

    Ok(HttpResponseBuilder::new(error.body.error.code.try_into().expect("error code was not a valid HTTP status code")).body(html))
}
