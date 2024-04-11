// Featurize, the FOSS feature flagging
// Copyright (C) 2024  Lucy Ekaterina
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use actix_web::{get, web, HttpResponse};
use sentry::{Hub, SentryFutureExt};
use serde::{Deserialize, Serialize};


use crate::{
    kratos_client::{KratosClient, VerificationFlowRequest},
    renderer::Renderer,
    Error, FlowUiNode,
};

#[derive(Deserialize, Debug)]
pub struct VerifyQuery {
    flow: Option<String>,
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
    nodes: Vec<FlowUiNode>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationFlowUiMessage {
    id: Option<usize>,
    text: String,
    r#type: String,
}

#[tracing::instrument]
#[get("/verification")]
pub async fn route(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<VerifyQuery>,
) -> Result<HttpResponse, Error> {
    handler(renderer, kratos, req, query)
        .bind_hub(Hub::current())
        .await
}

fn login_redirect() -> HttpResponse {
    HttpResponse::SeeOther()
        .append_header(("Location", "/login"))
        .finish()
}

#[tracing::instrument]
pub async fn handler(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<VerifyQuery>,
) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => Ok(login_redirect()),
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => return Ok(login_redirect()),
            };
            tracing::info!("getting flow");
            let res = kratos
                .new_request(VerificationFlowRequest(flow_id.to_string()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            Ok(renderer
                .render("verification.html")
                .var("flow", &res.body)
                .ok()
                .finish()?)
        }
    }
}
