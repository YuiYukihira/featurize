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
    ory_client::{ErrorsRequest, OryClient},
    renderer::Renderer,
    Error, StatusCodeConverter,
};

#[derive(Deserialize, Debug)]
pub struct ErrorQuery {
    id: String,
}

#[derive(Deserialize)]
pub struct AuthError {
    error: ErrorMessage,
}

#[derive(Deserialize)]
pub struct ErrorMessage {
    message: String,
    reason: String,
}

#[tracing::instrument]
#[get("/error")]
pub async fn route(
    renderer: web::Data<Renderer>,
    kratos: web::Data<OryClient>,
    query: web::Query<ErrorQuery>,
) -> Result<HttpResponse, Error> {
    handler(renderer, kratos, query)
        .bind_hub(Hub::current())
        .await
}

#[tracing::instrument]
pub async fn handler(
    renderer: web::Data<Renderer>,
    auth_config: web::Data<OryClient>,
    query: web::Query<ErrorQuery>,
) -> Result<HttpResponse, Error> {
    let error = auth_config
        .new_request(ErrorsRequest(query.id.clone()))
        .send()
        .await?;

    Ok(renderer
        .render("error.html")
        .var("msg", &error.body.error.message)
        .var("reason", &error.body.error.reason)
        .status(StatusCodeConverter(error.status_code).into())
        .finish()?)
}
