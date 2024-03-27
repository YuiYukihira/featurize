use actix_web::{get, web, HttpResponse, Responder};
use reqwest::StatusCode;
use serde::Deserialize;
use tera::Tera;

#[derive(Deserialize)]
struct LogoutUrlResponse {
    logout_url: String,
}

#[get("/")]
pub async fn route(tera: web::Data<Tera>, req: actix_web::HttpRequest) -> impl Responder {
    let cookie = req
        .headers()
        .get("Cookie")
        .unwrap()
        .to_str()
        .unwrap();

    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:4433/session/whoami")
        .header("Cookie", cookie)
        .send()
        .await
        .unwrap();

    let mut context = tera::Context::new();
    let mut template_file = "index.html";

    if res.status() == StatusCode::OK {
        let logout_url: LogoutUrlResponse = client
            .get("http://localhost:4433/self-service/logout/browser")
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
