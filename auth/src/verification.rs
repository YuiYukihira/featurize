
use actix_web::{get, web, HttpResponse, Responder};
use sentry::{SentryFutureExt, Hub};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{kratos_client::{KratosClient, VerificationFlowRequest}, renderer::Renderer, FlowUiNode, Error};


#[derive(Deserialize, Debug)]
pub struct VerifyQuery {
    flow: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationFlow {
    active: String,
    id: String,
    request_url: String,
    return_to: Option<String>,
    state: Option<String>,
    r#type: String,
    ui: VerificationFlowUi,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationFlowUi {
    action: String,
    messages: Vec<VerificationFlowUiMessage>,
    method: String,
    nodes: Vec<FlowUiNode>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationFlowUiMessage {
    id: Option<usize>,
    text: String,
    r#type: String,
}

#[tracing::instrument]
#[get("/verification")]
pub async fn route(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest, query: web::Query<VerifyQuery>) -> Result<HttpResponse, Error> {
    handler(renderer, kratos, req, query).bind_hub(Hub::current()).await
}

fn login_redirect() -> HttpResponse {
    HttpResponse::SeeOther().append_header(("Location", "/login")).finish()
}

#[tracing::instrument]
pub async fn handler(renderer: web::Data<Renderer>, kratos: web::Data<KratosClient>, req: actix_web::HttpRequest, query: web::Query<VerifyQuery>) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => Ok(login_redirect()),
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => return Ok(login_redirect())
            };
            tracing::info!("getting flow");
            let res = kratos.new_request(VerificationFlowRequest(flow_id.to_string()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            let html = renderer
                .render("verification.html")
                .var("flow", &res.body)
                .finish()?;

            Ok(HttpResponse::Ok().body(html))
        }
    }
}
