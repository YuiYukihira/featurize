
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::{redirect, AuthConfig, FlowUiNode};


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
pub async fn route(tera: web::Data<Tera>, auth_config: web::Data<AuthConfig>, req: actix_web::HttpRequest, query: web::Query<VerifyQuery>) -> impl Responder {
    match &query.flow {
        None => redirect("/login"),
        Some(flow_id) => {
            let cookies = match req.headers().get("Cookie") {
                Some(cookies) => cookies.to_str().unwrap(),
                None => return redirect("/login")
            };

            let client = reqwest::Client::new();
            let flow: VerificationFlow = client
                .get(auth_config.get_url("self-service/verfication/flows"))
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
