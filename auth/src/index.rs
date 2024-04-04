use actix_web::{get, web, HttpResponse};
use reqwest::StatusCode;
use sentry::{Hub, SentryFutureExt};

use crate::{kratos_client::{KratosClient, WhoAmIRequest, LogoutBrowserRequest}, renderer::Renderer, Error};


#[tracing::instrument]
#[get("/")]
pub async fn route(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, kratos, req).bind_hub(hub).await
}

#[tracing::instrument]
pub async fn handler(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest) -> Result<HttpResponse, Error> {

    let cookie = match req
        .headers()
        .get("Cookie") {
            Some(cookie) => cookie.as_bytes(),
            None => {
                tracing::info!("no cookie, showing public view");
                let html = renderer.render("index.html")
                    .finish()?;
                return Ok(HttpResponse::Ok().body(html));
            }
        };

    let session = kratos.new_request(WhoAmIRequest)
        .cookie(cookie)
        .send()
        .await?;

    let render_builder;

    if session.status_code == StatusCode::OK {
        let logout_url = kratos.new_request(LogoutBrowserRequest)
            .cookie(cookie)
            .send()
            .await?;
        render_builder = renderer.render("home.html")
            .var("logout_url", &logout_url.body.logout_url);
    } else if session.status_code != StatusCode::UNAUTHORIZED {
        render_builder = renderer.render("index.html");
        tracing::error!("Unexpected response code from identity server!");
    } else {
        render_builder = renderer.render("index.html");
    }

    let html = render_builder.finish()?;
    Ok(HttpResponse::Ok().body(html))
}
