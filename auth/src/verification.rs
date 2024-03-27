
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{redirect, FlowUiNode};


#[derive(Deserialize)]
pub struct VerifyQuery {
    flow: Option<String>
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

#[get("/verification")]
pub async fn route(tera: web::Data<Tera>, req: actix_web::HttpRequest, query: web::Query<VerifyQuery>) -> impl Responder {
    match &query.flow {
        None => redirect("/login"),
        Some(flow_id) => {
            let cookies = req.headers().get("Cookie").unwrap().to_str().unwrap();

            let client = reqwest::Client::new();
            let flow: VerificationFlow = client
                .get("http://localhost:4433/self-service/verfication/flows")
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

            let html = tera.render("verification.html", &context).unwrap();
            HttpResponse::Ok().body(html)
        }
    }
}
