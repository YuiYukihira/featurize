use std::env;

use serde::{Deserialize, Serialize};
use actix_web::{body::BoxBody, web, App, HttpResponse, HttpServer};
use tera::Tera;


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

pub fn redirect(s: &str) -> HttpResponse<BoxBody> {
    HttpResponse::SeeOther()
        .append_header(("Location", s))
        .finish()
}


#[actix_web::main]
async fn main() -> color_eyre::Result<()> {
    let port = {
        let p = env::var("PORT");

        match p {
            Ok(p) => p.parse(),
            Err(_) => Ok(8080),
        }
    }?;

    println!("Starting on: 0.0.0.0:{}", port);
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new({
                

                Tera::new("templates/**/*.html").unwrap()
            }))
            .service(index::route)
            .service(registration::route)
            .service(login::route)
            .service(verification::route)
            .service(error::route)
            .service(
                actix_files::Files::new("/public", "public")
                    .show_files_listing()
                    .use_last_modified(true),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
