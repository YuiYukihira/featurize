use actix_web::{get, web, HttpResponseBuilder, Responder};
use serde::Deserialize;
use tera::Tera;



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
pub async fn route(tera: web::Data<Tera>, query: web::Query<ErrorQuery>) -> impl Responder {
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
