use actix_web::{get, web, HttpResponse, Responder};
use sentry::{Breadcrumb, Hub, SentryFutureExt};
use serde::Deserialize;
use tera::Tera;

use crate::{Error, Flow, renderer::Renderer, kratos_client::{KratosClient, RegistrationBrowser, RegistrationFlowRequest}};


#[derive(Deserialize, Debug)]
pub struct RegisterQuery {
    flow: Option<String>
}

#[tracing::instrument]
#[get("/registration")]
pub async fn route(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest, query: web::Query<RegisterQuery>) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, kratos, req, query).bind_hub(hub).await
}

#[tracing::instrument]
pub async fn handler(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest, query: web::Query<RegisterQuery>) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => {
            tracing::info!("redirecting to login flow");
            Ok(kratos.redirect(RegistrationBrowser))
        },
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => {
                    tracing::error!("No CSRF token found!");
                    return Ok(kratos.redirect(RegistrationBrowser));
                }
            };
            tracing::info!("getting flow");
            let res = kratos.new_request(RegistrationFlowRequest(flow_id.to_string()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            let html = renderer
                .render("register.html")
                .var("flow", &res.body)
                .finish()?;

            Ok(HttpResponse::Ok().body(html))
        }
    }
}
