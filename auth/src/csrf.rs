use std::{cell::RefCell, future::Ready, ops::Deref, pin::Pin, task::Poll};

use actix_web::{
    cookie::Cookie,
    http::StatusCode,
    web::{Data, Form, Query},
    FromRequest, HttpRequest, HttpResponse, HttpResponseBuilder, ResponseError,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use futures::{Future, FutureExt};
use hmac::{Hmac, Mac};
use rand::{rngs::StdRng, CryptoRng, Fill, RngCore, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub trait CsrfTokenRng: CryptoRng {
    fn generate_token(&mut self) -> Result<String, rand::Error>;
}

impl<T: CryptoRng + RngCore> CsrfTokenRng for T {
    fn generate_token(&mut self) -> Result<String, rand::Error> {
        let mut buf = [0; 32];
        buf.try_fill(self)?;
        Ok(URL_SAFE_NO_PAD.encode(buf))
    }
}

pub struct CsrfService<T> {
    secret: Vec<u8>,
    domain: String,
    rng: RefCell<T>,
}

impl CsrfService<StdRng> {
    pub fn new(secret: Vec<u8>, domain: String) -> Self {
        Self {
            secret,
            domain,
            rng: RefCell::new(StdRng::from_entropy()),
        }
    }
}

impl<T: CsrfTokenRng> CsrfService<T> {
    #[tracing::instrument(skip(self))]
    fn generate_token(&self, session_id: &str) -> Result<CsrfToken, rand::Error> {
        let rand_val = self.rng.borrow_mut().generate_token()?;
        let message = format!("{}!{}", session_id, rand_val);
        let mut hasher = HmacSha256::new_from_slice(&self.secret).expect("Error initializing Hmac");
        hasher.update(message.as_bytes());
        let mac = hasher.finalize().into_bytes();
        let encoded_mac = URL_SAFE_NO_PAD.encode(mac);
        Ok(CsrfToken(format!("{}.{}", encoded_mac, message)))
    }

    #[tracing::instrument(skip(self, res))]
    pub fn add_token<'r>(
        &self,
        session_id: &str,
        res: &'r mut HttpResponseBuilder,
    ) -> Result<(CsrfToken, &'r mut HttpResponseBuilder), Error> {
        let token = self.generate_token(session_id)?;
        let cookie = Cookie::build("AntiCSRFToken", &token.0)
            .same_site(actix_web::cookie::SameSite::Strict)
            .secure(true)
            .http_only(true)
            .domain(&self.domain)
            .finish();
        let res = res.cookie(cookie);
        Ok((token, res))
    }

    #[tracing::instrument(skip(self))]
    fn verify_token(&self, token: &CsrfToken) -> Result<(), Error> {
        let mut split = token.0.split('.');

        let hmac = split.next().ok_or(Error::TokenInvalid)?;
        let message = split.next().ok_or(Error::TokenInvalid)?;

        let hmac_decoded = URL_SAFE_NO_PAD
            .decode(hmac)
            .map_err(|_| Error::TokenInvalid)?;

        let mut hasher = HmacSha256::new_from_slice(&self.secret).expect("Error initializing Hmac");
        hasher.update(message.as_bytes());
        hasher
            .verify_slice(&hmac_decoded)
            .map_err(|_| Error::CouldNotVerify(token.0.to_owned()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The CSRF Token is missing")]
    TokenMissing,
    #[error("The CSRF Token is invalid")]
    TokenInvalid,
    #[error("The CSRF Token '{0}' could not be verified")]
    CouldNotVerify(String),
    #[error("The CSRF Service could not be found")]
    ServiceMissing,
    #[error("There was an error producing the CSRF token: {0}")]
    CouldNotProduceToken(#[from] rand::Error),
    #[error("There was actix: {0}")]
    Actix(#[from] actix_web::Error),
    #[error("The sent tokens, '{0}' and '{1}' do not match")]
    TokensDoNotMatch(String, String),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::TokenMissing => StatusCode::BAD_REQUEST,
            Error::TokenInvalid => StatusCode::BAD_REQUEST,
            Error::CouldNotVerify(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Error::ServiceMissing => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CouldNotProduceToken(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Actix(e) => e.as_response_error().status_code(),
            Error::TokensDoNotMatch(_, _) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct CsrfToken(String);

#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Csrf<T>(T);

impl<T> AsRef<T> for Csrf<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Csrf<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait HasCsrfToken {
    fn get_csrf_token(&self) -> &CsrfToken;
}

impl<T: HasCsrfToken> HasCsrfToken for Form<T> {
    fn get_csrf_token(&self) -> &CsrfToken {
        self.0.get_csrf_token()
    }
}

impl<T: HasCsrfToken> HasCsrfToken for Query<T> {
    fn get_csrf_token(&self) -> &CsrfToken {
        self.0.get_csrf_token()
    }
}

impl<T: HasCsrfToken + FromRequest<Error = actix_web::Error>> FromRequest for Csrf<T> {
    type Error = Error;

    type Future = CsrfTokenFut<T>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        CsrfTokenFut {
            req: req.to_owned(),
            payload: payload.take(),
            fut: None,
        }
    }
}

pub struct CsrfTokenFut<T: FromRequest> {
    req: HttpRequest,
    payload: actix_web::dev::Payload,
    fut: Option<Pin<Box<T::Future>>>,
}

impl<T> Future for CsrfTokenFut<T>
where
    T: HasCsrfToken + FromRequest<Error = actix_web::Error>,
{
    type Output = Result<Csrf<T>, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.as_mut();

        let req_fut = this.fut.take();
        let mut p = this.payload.take();
        let mut req_fut = match req_fut {
            Some(f) => f,
            None => Box::pin(T::from_request(&this.req, &mut p)),
        };

        let csrf_bearer = match req_fut.as_mut().poll(cx) {
            std::task::Poll::Ready(v) => v?,
            std::task::Poll::Pending => {
                this.fut = Some(req_fut);
                this.payload = p;
                return std::task::Poll::Pending;
            }
        };

        let csrf_service = this
            .req
            .app_data::<Data<CsrfService<StdRng>>>()
            .ok_or(Error::ServiceMissing)?;

        let csrf_cookie = this
            .req
            .cookie("AntiCSRFToken")
            .ok_or(Error::TokenMissing)?;

        if csrf_cookie.value() != csrf_bearer.get_csrf_token().0 {
            return Poll::Ready(Err(Error::TokensDoNotMatch(
                csrf_cookie.value().to_owned(),
                csrf_bearer.get_csrf_token().0.to_owned(),
            )));
        }

        Poll::Ready(
            csrf_service
                .verify_token(csrf_bearer.get_csrf_token())
                .map(|_| Csrf(csrf_bearer)),
        )
    }
}
