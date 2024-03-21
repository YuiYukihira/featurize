use std::env;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use actix_web::{get, web, App, HttpResponse, HttpResponseBuilder, HttpServer, Responder};
use tera::Tera;

#[get("/")]
async fn index(tera: web::Data<Tera>) -> impl Responder {
    let context = tera::Context::new();
    let html = tera.render("index.html", &context).unwrap();
    HttpResponse::Ok().body(html)
}

#[derive(Deserialize)]
pub struct LoginQuery {
    flow: Option<String>,
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
    nodes: Vec<FlowUiNode>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationFlowUiMessage {
    id: Option<usize>,
    text: String,
    r#type: String,
}


#[derive(Deserialize)]
pub struct RegisterQuery {
    flow: Option<String>
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    flow: Option<String>
}

#[get("/registration")]
async fn register(tera: web::Data<Tera>, req: actix_web::HttpRequest, query: web::Query<RegisterQuery>) -> impl Responder {
    match &query.flow {
        None => {
            HttpResponse::SeeOther()
                .append_header(("Location", "http://localhost:4433/self-service/registration/browser"))
                .finish()
        },
        Some(flow_id) => {
            let cookie = req.headers().get("Cookie").unwrap().to_str().unwrap();

            let client = reqwest::Client::new();
            let flow: Flow = client
                .get("http://localhost:4433/self-service/registration/flows")
                .query(&[("id", flow_id)])
                .header("Cookie", cookie)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let mut context = tera::Context::new();
            context.insert("flow", &flow);

            let html = tera.render("register.html", &context).unwrap();
            HttpResponse::Ok().body(html)
        }
    }
}

#[get("/verification")]
async fn verify(tera: web::Data<Tera>, req: actix_web::HttpRequest, query: web::Query<VerifyQuery>) -> impl Responder {
    match &query.flow {
        None => {
            HttpResponse::SeeOther()
                .append_header(("Location", "/login"))
                .finish()
        },
        Some(flow_id) => {
            let cookie = req.headers().get("Cookie").unwrap().to_str().unwrap();

            let client = reqwest::Client::new();
            let flow: VerificationFlow = client
                .get("http://localhost:4433/self-service/verification/flows")
                .query(&[("id", flow_id)])
                .header("Cookie", cookie)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let mut context = tera::Context::new();
            context.insert("flow", dbg!(&flow));

            let html = tera.render("verification.html", &context).unwrap();
            HttpResponse::Ok().body(html)
        }
    }
}

#[get("/login")]
async fn login(tera: web::Data<Tera>, req: actix_web::HttpRequest, query: web::Query<LoginQuery>) -> impl Responder {
    match &query.flow {
        None => {
            HttpResponse::SeeOther()
                .append_header(("Location", "http://localhost:4433/self-service/login/browser"))
                .finish()
        },
        Some(flow_id) => {
            let cookie = req.headers().get("Cookie").unwrap();

            let client = reqwest::Client::new();
            let flow: Flow = client
                .get("http://localhost:4433/self-service/login/flows")
                .query(&[("id", flow_id)])
                .header("Cookie", cookie.to_str().unwrap())
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let mut context = tera::Context::new();
            context.insert("flow", dbg!(&flow));

            let html = tera.render("login.html", &context).unwrap();
            HttpResponse::Ok().body(html)
        }
    }
}

#[derive(Deserialize)]
pub struct ErrorQuery {
    id: String,
}

#[derive(Deserialize)]
pub struct AuthError {
    error: ErrorMessage
}

#[derive(Deserialize)]
pub struct ErrorMessage {
    code: u16,
    message: String,
    reason: String,
}

#[get("/error")]
async fn error_page(tera: web::Data<Tera>, query: web::Query<ErrorQuery>) -> impl Responder {
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:4433/self-service/errors")
        .query(&[("id", &query.id)])
        .send()
        .await
        .unwrap();
    let error = res.json::<AuthError>()
        .await
        .unwrap()
        .error;

    let mut context = tera::Context::new();
    context.insert("msg", &error.message);
    context.insert("reason", &error.reason);

    let html = tera.render("error.html", &context).unwrap();
    HttpResponseBuilder::new(error.code.try_into().unwrap()).body(html)
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
                let tera = Tera::new("templates/**/*.html").unwrap();

                tera
            }))
            .service(index)
            .service(login)
            .service(register)
            .service(verify)
            .service(error_page)
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
