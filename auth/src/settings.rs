use actix_web::{get, web, HttpResponse};
use sentry::{Hub, SentryFutureExt};
use serde::Deserialize;

use crate::{
    kratos_client::{KratosClient, SettingsFlowRequest, SettingsRedirect},
    renderer::Renderer,
    Error,
};

#[derive(Debug, Deserialize)]
pub struct SettingsQuery {
    flow: Option<String>,
}

#[tracing::instrument]
#[get("/settings")]
pub async fn route(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<SettingsQuery>,
) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, kratos, req, query).bind_hub(hub).await
}

#[tracing::instrument]
pub async fn handler(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<SettingsQuery>,
) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => {
            tracing::info!("redirecting to settings flow");
            Ok(kratos.redirect(SettingsRedirect))
        }
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => {
                    tracing::error!("No CSRF token found!");
                    return Ok(kratos.redirect(SettingsRedirect));
                }
            };
            tracing::info!("getting flow");
            let res = kratos
                .new_request(SettingsFlowRequest(flow_id.to_owned()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            match res.body {
                Ok(res) => Ok(renderer
                    .render("settings.html")
                    .var("flow", &res)
                    .ok()
                    .finish()?),
                Err(_) => {
                    tracing::info!("flow expired! redirecting");
                    Ok(kratos.redirect(SettingsRedirect))
                }
            }
        }
    }
}
