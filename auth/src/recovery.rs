use actix_web::{get, web, HttpResponse};
use sentry::{Hub, SentryFutureExt};
use serde::Deserialize;

use crate::{
    kratos_client::{KratosClient, RecoveryBrowser, RecoveryFlowRequest},
    renderer::Renderer,
    Error,
};

#[derive(Deserialize, Debug)]
pub struct RecoveryQuery {
    flow: Option<String>,
}

#[tracing::instrument]
#[get("/recovery")]
pub async fn route(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<RecoveryQuery>,
) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, kratos, req, query).bind_hub(hub).await
}

#[tracing::instrument]
async fn handler(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<RecoveryQuery>,
) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => {
            tracing::info!("redirecting to recovery flow");
            Ok(kratos.redirect(RecoveryBrowser))
        }
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => {
                    tracing::error!("No CSRF token found!");
                    return Ok(kratos.redirect(RecoveryBrowser));
                }
            };
            tracing::info!("getting flow");
            let res = kratos
                .new_request(RecoveryFlowRequest(flow_id.to_owned()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            Ok(renderer
                .render("recovery.html")
                .var("flow", &res.body)
                .ok()
                .finish()?)
        }
    }
}
