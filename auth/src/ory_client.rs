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
use std::{fmt::Debug, pin::Pin, task::Poll};

use actix_web::{web::Data, FromRequest, HttpRequest, HttpResponse};
use futures::Future;
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use sentry::{Breadcrumb, Hub};
use serde::{Deserialize, Serialize};

use crate::{error::AuthError, Error};

mod common;
mod login;
mod recovery;
mod registration;
mod settings;
mod verification;

pub use common::*;
pub use login::*;
pub use recovery::*;
pub use registration::*;
pub use settings::*;
pub use verification::*;

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

pub trait OryServiceType {
    fn get_domain(client: &OryClient) -> &str;
}
#[derive(Debug)]
pub struct Kratos;
impl OryServiceType for Kratos {
    fn get_domain(client: &OryClient) -> &str {
        &client.kratos_domain
    }
}
#[derive(Debug)]
pub struct Hydra;
impl OryServiceType for Hydra {
    fn get_domain(client: &OryClient) -> &str {
        &client.hydra_domain
    }
}

pub trait OryRequestType {
    const PATH: &'static str;
    const METHOD: Method;
    type ResponseType: for<'de> Deserialize<'de>;
    type NeedsCookie: NeedsCookieType + Debug;
    type Service: OryServiceType + Debug;

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

#[derive(Debug)]
pub struct WhoAmIRequest;
#[derive(Debug)]
pub struct LogoutBrowserRequest;
#[derive(Debug)]
pub struct ErrorsRequest(pub String);

impl OryRequestType for WhoAmIRequest {
    const PATH: &'static str = "sessions/whoami";
    const METHOD: Method = Method::GET;
    type ResponseType = Session;
    type NeedsCookie = Yes;
    type Service = Kratos;
}

#[derive(Debug, Deserialize)]
pub struct Session {
    pub active: Option<bool>,
    pub authenticated_at: Option<String>,
    pub authenticator_assurance_level: Option<String>,
    pub expires_at: String,
    pub id: String,
    pub identity: Option<Identity>,
}

#[derive(Debug, Deserialize)]
pub struct Identity {
    pub id: String,
    pub traits: serde_json::Map<String, serde_json::Value>,
}

impl OryRequestType for LogoutBrowserRequest {
    const PATH: &'static str = "self-service/logout/browser";
    const METHOD: Method = Method::GET;
    type ResponseType = LogoutUrlResponse;
    type NeedsCookie = Yes;
    type Service = Kratos;
}

impl OryRequestType for ErrorsRequest {
    const PATH: &'static str = "self-service/errors";
    const METHOD: Method = Method::GET;
    type ResponseType = AuthError;
    type NeedsCookie = No;
    type Service = Kratos;

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
pub struct OryClient {
    kratos_domain: String,
    hydra_domain: String,
    client: Client,
}

impl OryClient {
    pub fn new(kratos_domain: String, hydra_domain: String, client: Client) -> Self {
        Self {
            kratos_domain,
            hydra_domain,
            client,
        }
    }

    fn get_url<R: OryRequestType>(&self, req: &R) -> String {
        format!("{}/{}", <R::Service>::get_domain(self), req.get_url())
    }

    pub fn redirect<R: KratosRedirectType>(&self, typ: R) -> HttpResponse {
        HttpResponse::SeeOther()
            .append_header((
                "Location",
                format!("{}/{}", self.kratos_domain, typ.get_url()),
            ))
            .finish()
    }

    pub fn new_request<R: OryRequestType>(&self, request: R) -> OryRequest<R, NoCookie> {
        let req = request.build_req(self.client.request(R::METHOD, self.get_url(&request)));
        OryRequest {
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
pub struct OryRequest<'c, R, S> {
    client: &'c OryClient,
    request_type: R,
    _state: S,
    req: RequestBuilder,
}

#[derive(Debug)]
pub struct KratosResponse<R: OryRequestType> {
    pub body: R::ResponseType,
    pub status_code: reqwest::StatusCode,
}

impl<'c, R: OryRequestType<NeedsCookie = Yes>> OryRequest<'c, R, NoCookie> {
    pub fn cookie(self, cookie: &'c [u8]) -> OryRequest<R, WithCookie> {
        OryRequest {
            client: self.client,
            request_type: self.request_type,
            _state: WithCookie,
            req: self.req.header("Cookie", cookie),
        }
    }
}

impl<'c, R: OryRequestType<NeedsCookie = Yes> + Debug> OryRequest<'c, R, WithCookie> {
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

impl<'c, R: OryRequestType<NeedsCookie = No> + Debug> OryRequest<'c, R, NoCookie> {
    #[tracing::instrument]
    pub async fn send(self) -> Result<KratosResponse<R>, Error> {
        let url = self.client.get_url(&self.request_type);
        let method = R::METHOD;
        let res = self.req.send().await?;
        let status = res.status();
        let text = res.text().await?;
        Hub::current().add_breadcrumb(Breadcrumb {
            ty: "http".into(),
            data: {
                let mut map = sentry::protocol::Map::new();
                map.insert("url".into(), url.into());
                map.insert("method".into(), method.as_str().into());
                map.insert("status_code".into(), status.as_u16().into());
                map.insert("body".into(), serde_json::Value::String(text.clone()));
                map
            },
            ..Default::default()
        });
        let res_body: serde_json::Value = serde_json::from_str(&text)?;
        let body = R::construct_response(status, res_body)?;
        Ok(KratosResponse {
            body,
            status_code: status,
        })
    }
}

#[derive(Debug)]
pub struct GetOAuth2ConsentRequest(pub String);

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth2ConsentRequest {
    pub acr: Option<String>,
    pub amr: Option<Vec<String>>,
    pub challenge: String,
    pub client: Option<OAuth2Client>,
    pub context: Option<serde_json::Value>,
    pub login_challenge: Option<String>,
    pub session_id: Option<String>,
    pub oidc_context: Option<serde_json::Value>,
    pub request_url: Option<String>,
    pub requested_access_token_audience: Option<Vec<String>>,
    pub requested_scope: Option<Vec<String>>,
    pub skip: Option<bool>,
    pub subject: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth2Client {
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub policy_uri: Option<String>,
    pub skip_consent: Option<bool>,
    pub skip_logout_consent: Option<bool>,
}

impl OryRequestType for GetOAuth2ConsentRequest {
    const PATH: &'static str = "/oauth2/auth/requests/consent";

    const METHOD: Method = Method::GET;

    type ResponseType = OAuth2ConsentRequest;

    type NeedsCookie = No;

    type Service = Hydra;

    fn build_req(&self, req: RequestBuilder) -> RequestBuilder {
        req.query(&[("consent_challenge", &self.0)])
    }
}

impl FromRequest for Session {
    type Error = Error;

    type Future = SessionFut;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        SessionFut {
            req: req.to_owned(),
            fut: None,
        }
    }
}

pub struct SessionFut {
    req: HttpRequest,
    fut: Option<Pin<Box<dyn Future<Output = Result<KratosResponse<WhoAmIRequest>, Error>>>>>,
}

impl Future for SessionFut {
    type Output = Result<Session, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut fut = match self.fut.take() {
            Some(f) => f,
            None => {
                let ory = match self.req.app_data::<Data<OryClient>>() {
                    Some(o) => o.clone(),
                    None => return Poll::Ready(Err(Error::NoOryClient)),
                };
                let cookie = self
                    .req
                    .headers()
                    .get("Cookie")
                    .ok_or(Error::NoSession)?
                    .as_bytes()
                    .to_vec();
                let f = async move { ory.new_request(WhoAmIRequest).cookie(&cookie).send().await };
                Box::pin(f)
            }
        };

        match fut.as_mut().poll(cx) {
            Poll::Ready(r) => Poll::Ready(r.and_then(|s| {
                if s.status_code == StatusCode::OK {
                    Ok(s.body)
                } else {
                    Err(Error::NoSession)
                }
            })),
            Poll::Pending => {
                self.fut = Some(fut);
                Poll::Pending
            }
        }
    }
}
