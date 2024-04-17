use reqwest::{Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};

use crate::Error;

use super::{GenericError, KratosRedirectType, KratosRequestType, UiContainer, Yes};

#[derive(Debug)]
pub struct LoginFlowRequest(pub String);

#[derive(Debug, Deserialize)]
pub struct LoginFlowError {
    pub redirect_to: String,
    pub return_to: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginFlow {
    active: Option<String>,
    created_at: Option<String>,
    expires_at: String,
    id: String,
    issued_at: String,
    organization_id: Option<String>,
    refresh: Option<bool>,
    request_url: String,
    requested_aal: Option<String>,
    return_to: Option<String>,
    state: String,
    transient_payload: Option<serde_json::Value>,
    ui: UiContainer,
    updated_at: String,
}

impl KratosRequestType for LoginFlowRequest {
    const PATH: &'static str = "self-service/login/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Result<LoginFlow, GenericError<LoginFlowError>>;
    type NeedsCookie = Yes;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }

    fn construct_response(
        status_code: StatusCode,
        body: serde_json::Value,
    ) -> Result<Self::ResponseType, Error> {
        match status_code {
            StatusCode::GONE => Ok(Err(serde_json::from_value(body)?)),
            _ => Ok(Ok(serde_json::from_value(body)?)),
        }
    }
}

#[derive(Debug)]
pub struct LoginBrowser(pub Option<String>);

impl KratosRedirectType for LoginBrowser {
    const PATH: &'static str = "self-service/login/browser";

    fn get_url(&self) -> String {
        match &self.0 {
            Some(login_challenge) => format!("{}?login_challenge={}", Self::PATH, login_challenge),
            None => Self::PATH.to_string(),
        }
    }
}
