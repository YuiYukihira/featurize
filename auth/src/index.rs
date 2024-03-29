use actix_web::{get, web, HttpResponse, Responder};
use reqwest::StatusCode;
use serde::Deserialize;
use tera::Tera;

use crate::AuthConfig;

#[derive(Deserialize)]
struct LogoutUrlResponse {
    logout_url: String,
}

#[get("/")]
pub async fn route(tera: web::Data<Tera>, auth_config: web::Data<AuthConfig>, req: actix_web::HttpRequest) -> impl Responder {
    let cookie = match req
        .headers()
        .get("Cookie") {
            Some(cookie) => cookie.to_str().unwrap(),
            None => {

                let context = tera::Context::new();
                let template_file = "index.html";
                let html = tera.render(template_file, &context).unwrap();
                return HttpResponse::Ok().body(html);
            }
        };

    let client = reqwest::Client::new();
    let res = client
        .get(auth_config.get_url("session/whoami"))
        .header("Cookie", cookie)
        .send()
        .await
        .unwrap();

    let mut context = tera::Context::new();
    let mut template_file = "index.html";

    if res.status() == StatusCode::OK {
        let logout_url: LogoutUrlResponse = client
            .get(auth_config.get_url("self-service/logout/browser"))
            .header("Cookie", cookie)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        context.insert("logout_url", &logout_url.logout_url);
        template_file = "home.html";
    }

    let html = tera.render(template_file, &context).unwrap();
    HttpResponse::Ok().body(html)
}
