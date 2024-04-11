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
use actix_web::{dev::Response, get, http::StatusCode, web, HttpResponse};
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

            match res.body {
                Ok(res) => {
                    Ok(renderer
                        .render("login.html")
                        .var("flow", &res)
                        .ok()
                        .finish()?)
                },
                Err(err) => {
                    tracing::info!("flow expired! redirecting");
                   match err.details {
                       Some(deets) => Ok(HttpResponse::SeeOther().append_header(("Location", deets.redirect_to)).finish()),
                       None => Ok(kratos.redirect(LoginBrowser))
                   } 
                }
            }
        }
    }
}
