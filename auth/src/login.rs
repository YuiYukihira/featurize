use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use tera::Tera;

use crate::{redirect, AuthConfig, Flow};

#[derive(Deserialize)]
struct LoginQuery {
    flow: Option<String>
}


#[get("/login")]
pub async fn route(tera: web::Data<Tera>, auth_config: web::Data<AuthConfig>, req: actix_web::HttpRequest, query: web::Query<LoginQuery>) -> impl Responder {
    match &query.flow {
        None => redirect(&auth_config.get_url("self-service/login/browser")),
        Some(flow_id) => {
            let client = reqwest::Client::new();

            let mut flow_req = client
                .get(auth_config.get_url("self-service/login/flows"))
                .query(&[("id", flow_id)]);

            if let Some(cookie) = req.headers().get("Cookie") {
                flow_req = flow_req.header("Cookie", cookie.to_str().unwrap());
            }

            let flow: Flow = flow_req
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let mut context = tera::Context::new();
            context.insert("flow", &flow);

            let html = tera.render("login.html", &context).unwrap();
            HttpResponse::Ok().body(html)
        }
    }
}
