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
use std::{env, sync::Arc};

use actix_web::{http::StatusCode, web, App, HttpServer};
use serde::{Deserialize, Serialize};
use tera::Tera;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{ory_client::OryClient, renderer::Renderer};

mod error;
mod index;
mod login;
mod ory_client;
mod recovery;
mod registration;
mod renderer;
mod settings;
mod verification;

pub struct StatusCodeConverter(reqwest::StatusCode);

impl From<StatusCodeConverter> for actix_web::http::StatusCode {
    fn from(value: StatusCodeConverter) -> Self {
        Self::from_u16(value.0.as_u16()).unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error deserializing data: {0}")]
    DeserializationError(#[from] serde_json::Error),
    #[error("error rendering the template: {0}")]
    RenderingError(#[from] tera::Error),
    #[error("An error fetching data has occured: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Could not read cookie header")]
    CookieToString(#[from] actix_web::http::header::ToStrError),
    #[error("An unknown error has occured")]
    Unknown,
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::DeserializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::RenderingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CookieToString(_) => StatusCode::BAD_REQUEST,
            Error::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Flow {
    id: String,
    ui: FlowUi,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUi {
    nodes: Vec<FlowUiNode>,
    action: String,
    method: String,
    #[serde(default)]
    messages: Vec<FlowUiMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUiMessage {
    id: Option<usize>,
    text: String,
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUiNode {
    attributes: FlowUiNodeAttributes,
    group: String,
    r#type: String,
    messages: Vec<FlowUiNodeMessages>,
    meta: FlowUiNodeMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUiNodeMeta {
    label: Option<FlowUiNodeAttributesLabel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUiNodeMessages {
    id: Option<usize>,
    text: String,
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUiNodeAttributesLabel {
    id: usize,
    text: String,
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUiNodeAttributes {
    name: Option<String>,
    value: Option<String>,
    autocomplete: Option<String>,
    disabled: Option<bool>,
    node_type: String,
    onclick: Option<String>,
    pattern: Option<String>,
    required: Option<bool>,
    r#type: Option<String>,
    label: Option<FlowUiNodeAttributesLabel>,
    title: Option<FlowUiNodeAttributesLabel>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let environment = env::var("ENV").unwrap_or("Dev".to_string());

    let _guard = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        send_default_pii: environment == "Dev",
        environment: Some(environment.into()),
        traces_sample_rate: 1.0,
        #[cfg(debug_assertions)]
        before_send: Some(Arc::new(|e| Some(e))),
        #[cfg(debug_assertions)]
        before_breadcrumb: Some(Arc::new(|e| Some(e))),
        ..Default::default()
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    actix_web::rt::System::new().block_on(run_server())?;

    Ok(())
}

async fn run_server() -> color_eyre::Result<()> {
    let port = {
        let p = env::var("PORT");

        match p {
            Ok(p) => p.parse(),
            Err(_) => Ok(80),
        }
    }?;

    println!("Starting on: 0.0.0.0:{}", port);
    HttpServer::new(|| {
        let kratos_domain = env::var("KRATOS_DOMAIN").unwrap();
        let templates_dir = env::var("TEMPLATES_DIR").unwrap_or("templates".to_string());
        let public_dir = env::var("PUBLIC_DIR").unwrap_or("public".to_string());
        let sentry_dsn = env::var("SENTRY_DSN").unwrap();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(sentry_actix::Sentry::new())
            .app_data(web::Data::new(Renderer::new(
                Tera::new(&format!("{}/**/*.html", templates_dir)).unwrap(),
                sentry_dsn.clone(),
            )))
            .app_data(web::Data::new(OryClient::new(
                kratos_domain,
                reqwest::Client::new(),
            )))
            .service(index::route)
            .service(registration::route)
            .service(login::route)
            .service(verification::route)
            .service(error::route)
            .service(recovery::route)
            .service(settings::route)
            .service(
                actix_files::Files::new("/public", public_dir)
                    .show_files_listing()
                    .use_last_modified(true),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
