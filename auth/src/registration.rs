use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use tera::Tera;

use crate::{redirect, AuthConfig, Flow};


#[derive(Deserialize, Debug)]
pub struct RegisterQuery {
    flow: Option<String>
}

#[tracing::instrument]
#[get("/registration")]
pub async fn route(tera: web::Data<Tera>, auth_config: web::Data<AuthConfig>, req: actix_web::HttpRequest, query: web::Query<RegisterQuery>) -> impl Responder {
    match &query.flow {
        None => redirect(&auth_config.get_url("self-service/registration/browser")),
        Some(flow_id) => {
            let client = reqwest::Client::new();

            let mut flow_req = client
                .get(auth_config.get_url("self-service/registration/flows"))
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

            let html = tera.render("register.html", &context).unwrap();
            HttpResponse::Ok().body(html)
        }
    }
}
