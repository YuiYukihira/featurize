use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};

use super::{GenericError, KratosRedirectType, KratosRequestType, UiContainer, Yes};

#[derive(Debug, Deserialize, Serialize)]
pub struct RecoveryFlow {
    active: Option<String>,
    expires_with: String,
    id: String,
    issued_at: String,
    request_url: String,
    return_to: Option<String>,
    state: String,
    transient_payload: Option<serde_json::Value>,
    ui: UiContainer,
}

#[derive(Debug)]
pub struct RecoveryFlowRequest(pub String);

impl KratosRequestType for RecoveryFlowRequest {
    const PATH: &'static str = "self-service/recovery/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Result<RecoveryFlow, GenericError<serde_json::Value>>;
    type NeedsCookie = Yes;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }
}

#[derive(Debug)]
pub struct RecoveryBrowser;

impl KratosRedirectType for RecoveryBrowser {
    const PATH: &'static str = "self-service/recovery/browser";
}
