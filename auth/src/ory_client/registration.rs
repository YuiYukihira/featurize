use reqwest::{Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};

use super::{GenericError, Kratos, KratosRedirectType, OryRequestType, UiContainer, Yes};

#[derive(Debug, Deserialize, Serialize)]
pub struct RegistrationFlow {
    active: Option<String>,
    expires_at: String,
    id: String,
    issued_at: String,
    organization_id: Option<String>,
    request_url: String,
    return_to: Option<String>,
    state: String,
    transient_payload: Option<serde_json::Value>,
    ui: UiContainer,
}

#[derive(Debug)]
pub struct RegistrationFlowRequest(pub String);

impl OryRequestType for RegistrationFlowRequest {
    const PATH: &'static str = "self-service/registration/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Result<RegistrationFlow, GenericError<serde_json::Value>>;
    type NeedsCookie = Yes;
    type Service = Kratos;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }

    fn construct_response(
        status_code: reqwest::StatusCode,
        body: serde_json::Value,
    ) -> Result<Self::ResponseType, crate::Error> {
        match status_code {
            StatusCode::GONE => Ok(Err(serde_json::from_value(body)?)),
            _ => Ok(Ok(serde_json::from_value(body)?)),
        }
    }
}

#[derive(Debug)]
pub struct RegistrationBrowser;

impl KratosRedirectType for RegistrationBrowser {
    const PATH: &'static str = "self-service/registration/browser";
}
