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
use actix_web::{get, web, HttpResponse, Responder};
use sentry::{Breadcrumb, Hub, SentryFutureExt};
use serde::Deserialize;
use tera::Tera;

use crate::{
    kratos_client::{KratosClient, RegistrationBrowser, RegistrationFlowRequest},
    renderer::Renderer,
    Error, Flow,
};

#[derive(Deserialize, Debug)]
pub struct RegisterQuery {
    flow: Option<String>,
}

#[tracing::instrument]
#[get("/registration")]
pub async fn route(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<RegisterQuery>,
) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, kratos, req, query).bind_hub(hub).await
}

#[tracing::instrument]
pub async fn handler(
    renderer: web::Data<Renderer>,
    kratos: web::Data<KratosClient>,
    req: actix_web::HttpRequest,
    query: web::Query<RegisterQuery>,
) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => {
            tracing::info!("redirecting to login flow");
            Ok(kratos.redirect(RegistrationBrowser))
        }
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => {
                    tracing::error!("No CSRF token found!");
                    return Ok(kratos.redirect(RegistrationBrowser));
                }
            };
            tracing::info!("getting flow");
            let res = kratos
                .new_request(RegistrationFlowRequest(flow_id.to_string()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            Ok(renderer
                .render("register.html")
                .var("flow", &res.body)
                .ok()
                .finish()?)
        }
    }
}
