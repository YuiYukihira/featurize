use actix_web::{get, web, HttpResponse};
use sentry::{Hub, SentryFutureExt};
use serde::Deserialize;

use crate::{Error, kratos_client::{KratosClient, LoginBrowser, LoginFlowRequest}, renderer::Renderer};


#[derive(Deserialize, Debug)]
pub struct LoginQuery {
    flow: Option<String>
}

#[tracing::instrument]
#[get("/login")]
pub async fn route(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest, query: web::Query<LoginQuery>) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, kratos, req, query).bind_hub(hub).await
}

#[tracing::instrument]
pub async fn handler(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest, query: web::Query<LoginQuery>) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => {
            tracing::info!("redirecting to login flow");
            Ok(kratos.redirect(LoginBrowser))
        },
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => {
                    tracing::error!("No CSRF token found!");
                    return Ok(kratos.redirect(LoginBrowser));
                }
            };
            tracing::info!("getting flow");
            let res = kratos.new_request(LoginFlowRequest(flow_id.to_string()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            let html = renderer
                .render("login.html")
                .var("flow", &res.body)
                .finish()?;

            Ok(HttpResponse::Ok().body(html))
        }
    }
}
