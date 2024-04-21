use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};

use super::{GenericError, Kratos, OryRequestType, UiContainer, Yes};

#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationFlow {
    id: String,
    request_url: String,
    return_to: Option<String>,
    state: Option<String>,
    r#type: String,
    ui: UiContainer,
}

#[derive(Debug)]
pub struct VerificationFlowRequest(pub String);

impl OryRequestType for VerificationFlowRequest {
    const PATH: &'static str = "self-service/verification/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Result<VerificationFlow, GenericError<serde_json::Value>>;
    type NeedsCookie = Yes;
    type Service = Kratos;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }
}
