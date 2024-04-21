use reqwest::{Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};

use super::{GenericError, Kratos, KratosRedirectType, OryRequestType, UiContainer, Yes};

#[derive(Debug, Deserialize, Serialize)]
pub struct SettingsFlow {
    active: Option<String>,
    expires_at: String,
    id: String,
    issued_at: String,
    request_url: String,
    return_to: Option<String>,
    state: String,
    r#type: String,
    ui: UiContainer,
}

#[derive(Debug)]
pub struct SettingsFlowRequest(pub String);

impl OryRequestType for SettingsFlowRequest {
    const PATH: &'static str = "self-service/settings/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Result<SettingsFlow, GenericError<serde_json::Value>>;
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
pub struct SettingsRedirect;

impl KratosRedirectType for SettingsRedirect {
    const PATH: &'static str = "self-service/settings/browser";
}
