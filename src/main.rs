use std::{env, future::IntoFuture, sync::Arc};

use actix_web::{http::StatusCode, web, App, HttpServer};
use tera::Tera;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{ory_client::OryClient, renderer::Renderer};

mod index;
mod ory_client;
mod renderer;

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
    #[error("Ory client missing")]
    NoOryClient,
    #[error("No session available")]
    NoSession,
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
            Error::NoOryClient => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NoSession => StatusCode::UNAUTHORIZED,
        }
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let environment = env::var("ENV").unwrap_or("Dev".to_string());

    let _guard = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        send_default_pii: environment == "Dev",
        traces_sample_rate: if environment == "Dev" { 1.0 } else { 0.1 },
        environment: Some(environment.into()),
        #[cfg(debug_assertions)]
        before_send: Some(Arc::new(Some)),
        #[cfg(debug_assertions)]
        before_breadcrumb: Some(Arc::new(Some)),
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
        let hydra_domain = env::var("HYDRA_DOMAIN").unwrap();
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
                hydra_domain,
                reqwest::Client::new(),
            )))
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
