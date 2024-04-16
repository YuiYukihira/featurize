use actix_web::{get, web, HttpResponse};
use reqwest::{Method, StatusCode};
use sentry::{Hub, SentryFutureExt};
use serde::{Deserialize, Serialize};

use crate::{
    kratos_client::{GenericError, KratosClient, KratosRedirectType, KratosRequestType, Yes},
    renderer::{self, Renderer},
    Error,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct SettingsFlow {
    active: Option<String>,
    // TODO: continue_with: Vec<ContinueWith>,
    expires_at: String,
    id: String,
    // TODO: identity: Identity
    issued_at: String,
    request_url: String,
    return_to: Option<String>,
    state: String,
    r#type: String,
    ui: UiContainer,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiContainer {
    action: String,
    #[serde(default)]
    messages: Vec<UiText>,
    method: String,
    nodes: Vec<UiNode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiText {
    context: Option<serde_json::Value>,
    id: i64,
    text: String,
    r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiNode {
    attributes: serde_json::Value,
    group: String,
    #[serde(default)]
    messages: Vec<UiText>,
    meta: UiNodeMeta,
    r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UiNodeMeta {
    label: Option<UiText>,
}

#[derive(Debug)]
pub struct SettingsFlowRequest(pub String);

impl KratosRequestType for SettingsFlowRequest {
    const PATH: &'static str = "self-service/settings/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Result<SettingsFlow, GenericError<serde_json::Value>>;
    type NeedsCookie = Yes;

    fn build_req(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.query(&[("id", &self.0)])
    }

    fn construct_response(
        status_code: reqwest::StatusCode,
        body: serde_json::Value,
    ) -> Result<Self::ResponseType, crate::Error> {
        match status_code {
            StatusCode::GONE => Ok(Err(serde_json::from_value(body)?)),
            _ => Ok(Ok(serde_json::from_value(body)?)),
        }
    }
}

#[derive(Debug)]
pub struct SettingsRedirect;

impl KratosRedirectType for SettingsRedirect {
    const PATH: &'static str = "self-service/settings/browser";
}

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
