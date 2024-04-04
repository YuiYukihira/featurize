use std::env;

use serde::{Deserialize, Serialize};
use actix_web::{body::BoxBody, http::StatusCode, web, App, HttpResponse, HttpServer};
use tera::Tera;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{kratos_client::KratosClient, renderer::Renderer};


mod index;
mod registration;
mod login;
mod verification;
mod error;
mod renderer;
mod kratos_client;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error deserializing data")]
    DeserializationError(#[from] serde_json::Error),
    #[error("error rendering the template")]
    RenderingError(#[from] tera::Error),
    #[error("An error fetching data has occured")]
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
    ui: FlowUi
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowUi {
    nodes: Vec<FlowUiNode>,
    action: String,
    method: String,
    #[serde(default)]
    messages: Vec<FlowUiMessage>
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
    label: Option<FlowUiNodeAttributesLabel>
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let environment = env::var("ENV").unwrap_or("Dev".to_string());


    let _guard = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        send_default_pii: environment == "Dev",
        environment: Some(environment.into()),
        traces_sample_rate: 1.0,
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
        let auth_domain = env::var("KRATOS_DOMAIN").unwrap();
        let templates_dir = env::var("TEMPLATES_DIR").unwrap_or("templates".to_string());
        let public_dir = env::var("PUBLIC_DIR").unwrap_or("public".to_string());
        let sentry_dsn = env::var("SENTRY_DSN").unwrap();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(sentry_actix::Sentry::new())
            .app_data(web::Data::new(Renderer::new(
                Tera::new(&format!("{}/**/*.html", templates_dir)).unwrap(),
                sentry_dsn.clone()
            )))
            .app_data(web::Data::new(KratosClient::new(auth_domain, reqwest::Client::new())))
            .service(index::route)
            .service(registration::route)
            .service(login::route)
            .service(verification::route)
            .service(error::route)
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
