use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use tera::Tera;

use crate::{redirect, Flow};

#[derive(Deserialize)]
struct LoginQuery {
    flow: Option<String>
}


#[get("/login")]
pub async fn route(tera: web::Data<Tera>, req: actix_web::HttpRequest, query: web::Query<LoginQuery>) -> impl Responder {
    match &query.flow {
        None => redirect("http://localhost:4433/self-service/login/browser"),
        Some(flow_id) => {
            let cookies = req.headers().get("Cookie").unwrap().to_str().unwrap();

            let client = reqwest::Client::new();
            let flow: Flow = client
                .get("http://localhost:4433/self-service/login/flows")
                .query(&[("id", flow_id)])
                .header("Cookie", cookies)
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
