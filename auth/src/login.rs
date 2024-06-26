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
use serde::Deserialize;

use crate::{
    ory_client::{LoginBrowser, LoginFlowRequest, OryClient},
    renderer::Renderer,
    Error,
};

#[derive(Deserialize, Debug)]
pub struct LoginQuery {
    flow: Option<String>,
    login_challenge: Option<String>,
}

#[tracing::instrument]
#[get("/login")]
pub async fn route(
    renderer: web::Data<Renderer>,
    ory: web::Data<OryClient>,
    req: actix_web::HttpRequest,
    query: web::Query<LoginQuery>,
) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, ory, req, query).bind_hub(hub).await
}

#[tracing::instrument]
pub async fn handler(
    renderer: web::Data<Renderer>,
    ory: web::Data<OryClient>,
    req: actix_web::HttpRequest,
    query: web::Query<LoginQuery>,
) -> Result<HttpResponse, Error> {
    match &query.flow {
        None => {
            tracing::info!("redirecting to login flow");
            Ok(ory.redirect(LoginBrowser(query.login_challenge.clone())))
        }
        Some(flow_id) => {
            let cookie = match req.headers().get("Cookie") {
                Some(cookie) => cookie,
                None => {
                    tracing::error!("No CSRF token found!");
                    return Ok(ory.redirect(LoginBrowser(None)));
                }
            };
            tracing::info!("getting flow");
            let res = ory
                .new_request(LoginFlowRequest(flow_id.to_string()))
                .cookie(cookie.as_bytes())
                .send()
                .await?;

            match res.body {
                Ok(res) => Ok(renderer
                    .render("login.html")
                    .var("flow", &res)
                    .ok()
                    .finish()?),
                Err(err) => {
                    tracing::info!("flow expired! redirecting");
                    match err.details {
                        Some(deets) => Ok(HttpResponse::SeeOther()
                            .append_header(("Location", deets.redirect_to))
                            .finish()),
                        None => Ok(ory.redirect(LoginBrowser(None))),
                    }
                }
            }
        }
    }
}
