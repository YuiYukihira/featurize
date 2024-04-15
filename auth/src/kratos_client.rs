// Featurize, the FOSS feature flagging
// Copyright (C) 2024  Lucy Ekaterina
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use std::fmt::Debug;

use actix_web::HttpResponse;
use reqwest::{Method, RequestBuilder, StatusCode};
use sentry::{Breadcrumb, Hub};
use serde::{Deserialize, Serialize};

use crate::{error::AuthError, verification::VerificationFlow, Error, Flow};

#[derive(Deserialize)]
pub struct LogoutUrlResponse {
    pub logout_url: String,
}

#[derive(Deserialize, Serialize)]
pub struct GenericError<T> {
    pub code: Option<u64>,
    pub debug: Option<String>,
    pub details: Option<T>,
    pub id: Option<String>,
    pub message: String,
    pub reason: Option<String>,
    pub request: Option<String>,
    pub status: Option<String>,
}

pub trait NeedsCookieType {}
#[derive(Debug)]
pub struct Yes;
impl NeedsCookieType for Yes {}
#[derive(Debug)]
pub struct No;
impl NeedsCookieType for No {}

pub trait KratosRequestType {
    const PATH: &'static str;
    const METHOD: Method;
    type ResponseType: for<'de> Deserialize<'de>;
    type NeedsCookie: NeedsCookieType + Debug;

    fn get_url(&self) -> String {
        Self::PATH.to_string()
    }

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req
    }

    fn construct_response(
        _status_code: StatusCode,
        body: serde_json::Value,
    ) -> Result<Self::ResponseType, Error> {
        Ok(serde_json::from_value(body)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginFlowError {
    pub redirect_to: String,
    pub return_to: String,
}

#[derive(Debug)]
pub struct WhoAmIRequest;
#[derive(Debug)]
pub struct LogoutBrowserRequest;
#[derive(Debug)]
pub struct LoginFlowRequest(pub String);
#[derive(Debug)]
pub struct RegistrationFlowRequest(pub String);
#[derive(Debug)]
pub struct VerificationFlowRequest(pub String);
#[derive(Debug)]
pub struct ErrorsRequest(pub String);
#[derive(Debug)]
pub struct RecoveryFlowRequest(pub String);

impl KratosRequestType for WhoAmIRequest {
    const PATH: &'static str = "sessions/whoami";
    const METHOD: Method = Method::GET;
    type ResponseType = serde_json::Value;
    type NeedsCookie = Yes;
}

impl KratosRequestType for LogoutBrowserRequest {
    const PATH: &'static str = "self-service/logout/browser";
    const METHOD: Method = Method::GET;
    type ResponseType = LogoutUrlResponse;
    type NeedsCookie = Yes;
}

impl KratosRequestType for LoginFlowRequest {
    const PATH: &'static str = "self-service/login/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Result<Flow, GenericError<LoginFlowError>>;
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

impl KratosRequestType for RegistrationFlowRequest {
    const PATH: &'static str = "self-service/registration/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Flow;
    type NeedsCookie = Yes;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }
}

impl KratosRequestType for VerificationFlowRequest {
    const PATH: &'static str = "self-service/verification/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = VerificationFlow;
    type NeedsCookie = Yes;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }
}

impl KratosRequestType for ErrorsRequest {
    const PATH: &'static str = "self-service/errors";
    const METHOD: Method = Method::GET;
    type ResponseType = AuthError;
    type NeedsCookie = No;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }
}

impl KratosRequestType for RecoveryFlowRequest {
    const PATH: &'static str = "self-service/recovery/flows";
    const METHOD: Method = Method::GET;
    type ResponseType = Flow;
    type NeedsCookie = Yes;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("id", &self.0)])
    }
}

pub trait KratosRedirectType: Debug {
    const PATH: &'static str;

    fn get_url(&self) -> String {
        Self::PATH.to_string()
    }
}

#[derive(Debug)]
pub struct LoginBrowser;
#[derive(Debug)]
pub struct RegistrationBrowser;
#[derive(Debug)]
pub struct RecoveryBrowser;

impl KratosRedirectType for LoginBrowser {
    const PATH: &'static str = "self-service/login/browser";
}

impl KratosRedirectType for RegistrationBrowser {
    const PATH: &'static str = "self-service/registration/browser";
}

impl KratosRedirectType for RecoveryBrowser {
    const PATH: &'static str = "self-service/recovery/browser";
}

#[derive(Debug)]
pub struct KratosClient {
    domain: String,
    client: reqwest::Client,
}

impl KratosClient {
    pub fn new(domain: String, client: reqwest::Client) -> Self {
        Self { domain, client }
    }
    fn get_url<R: KratosRequestType>(&self, req: &R) -> String {
        format!("{}/{}", self.domain, req.get_url())
    }

    pub fn redirect<R: KratosRedirectType>(&self, typ: R) -> HttpResponse {
        HttpResponse::SeeOther()
            .append_header(("Location", format!("{}/{}", self.domain, typ.get_url())))
            .finish()
    }

    pub fn new_request<R: KratosRequestType>(&self, request: R) -> KratosRequest<R, NoCookie> {
        let req = request.build_req(self.client.request(R::METHOD, self.get_url(&request)));
        KratosRequest {
            client: self,
            request_type: request,
            _state: NoCookie,
            req,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoCookie;
#[derive(Debug, Clone, Copy)]
pub struct WithCookie;

#[derive(Debug)]
pub struct KratosRequest<'c, R, S> {
    client: &'c KratosClient,
    request_type: R,
    _state: S,
    req: reqwest::RequestBuilder,
}

#[derive(Debug)]
pub struct KratosResponse<R: KratosRequestType> {
    pub body: R::ResponseType,
    pub status_code: reqwest::StatusCode,
}

impl<'c, R: KratosRequestType<NeedsCookie = Yes>> KratosRequest<'c, R, NoCookie> {
    pub fn cookie(self, cookie: &'c [u8]) -> KratosRequest<R, WithCookie> {
        KratosRequest {
            client: self.client,
            request_type: self.request_type,
            _state: WithCookie,
            req: self.req.header("Cookie", cookie),
        }
    }
}

impl<'c, R: KratosRequestType<NeedsCookie = Yes> + Debug> KratosRequest<'c, R, WithCookie> {
    #[tracing::instrument]
    pub async fn send(self) -> Result<KratosResponse<R>, Error> {
        let url = self.client.get_url(&self.request_type);
        let method = R::METHOD;
        let res = self.req.send().await?;
        let status = res.status();
        let res_body: serde_json::Value = res.json().await?;
        Hub::current().add_breadcrumb(Breadcrumb {
            ty: "http".into(),
            data: {
                let mut map = sentry::protocol::Map::new();
                map.insert("url".into(), url.into());
                map.insert("method".into(), method.as_str().into());
                map.insert("status_code".into(), status.as_u16().into());
                map.insert("body".into(), res_body.clone());
                map
            },
            ..Default::default()
        });
        let body = R::construct_response(status, res_body)?;
        Ok(KratosResponse {
            body,
            status_code: status,
        })
    }
}

impl<'c, R: KratosRequestType<NeedsCookie = No> + Debug> KratosRequest<'c, R, NoCookie> {
    #[tracing::instrument]
    pub async fn send(self) -> Result<KratosResponse<R>, Error> {
        let url = self.client.get_url(&self.request_type);
        let method = R::METHOD;
        let res = self.req.send().await?;
        let status = res.status();
        let res_body: serde_json::Value = res.json().await?;
        Hub::current().add_breadcrumb(Breadcrumb {
            ty: "http".into(),
            data: {
                let mut map = sentry::protocol::Map::new();
                map.insert("url".into(), url.into());
                map.insert("method".into(), method.as_str().into());
                map.insert("status_code".into(), status.as_u16().into());
                map.insert("body".into(), res_body.clone());
                map
            },
            ..Default::default()
        });
        let body = R::construct_response(status, res_body)?;
        Ok(KratosResponse {
            body,
            status_code: status,
        })
    }
}
