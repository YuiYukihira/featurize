use std::env;

use serde::{Deserialize, Serialize};
use actix_web::{body::BoxBody, dev::AppConfig, web, App, HttpResponse, HttpServer};
use tera::Tera;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


mod index;
mod registration;
mod login;
mod verification;
mod error;

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

#[derive(Debug)]
pub struct AuthConfig {
    domain: String,
}

impl AuthConfig {
    pub fn get_url(&self, p: &str) -> String {
        format!("{}/{}", self.domain, p)
    }
}

pub fn redirect(s: &str) -> HttpResponse<BoxBody> {
    HttpResponse::SeeOther()
        .append_header(("Location", s))
        .finish()
}


#[actix_web::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _guard = sentry::init(sentry::ClientOptions {
        traces_sample_rate: 1.0,
        ..Default::default()
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

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
        println!("TEMPLATES_DIR: {}", templates_dir);
        println!("PUBLIC_DIR: {}", public_dir);
        println!("KRATOS_DOMAIN: {}", auth_domain);

        App::new()
            .app_data(web::Data::new(Tera::new(&format!("{}/**/*.html", templates_dir)).unwrap()))
            .app_data(web::Data::new(AuthConfig { domain: auth_domain }))
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
