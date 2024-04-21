use actix_web::{get, post, web, HttpResponse};
use rand::rngs::StdRng;
use sentry::{Hub, SentryFutureExt};
use serde::Deserialize;

use crate::{
    csrf::{Csrf, CsrfService, CsrfToken, HasCsrfToken},
    ory_client::{
        AcceptOAuth2ConsentRequest, GetOAuth2ConsentRequest, OAuth2ConsentRequest, OryClient,
        RejectOAuth2ConsentRequest, Session,
    },
    renderer::Renderer,
    Error,
};

#[derive(Debug, Deserialize)]
pub struct GetConsentQuery {
    consent_challenge: Option<String>,
}

#[tracing::instrument(skip(csrf_service))]
#[get("/consent")]
pub async fn get_route(
    renderer: web::Data<Renderer>,
    ory: web::Data<OryClient>,
    req: actix_web::HttpRequest,
    query: web::Query<GetConsentQuery>,
    csrf_service: web::Data<CsrfService<StdRng>>,
    user_session: Session,
) -> Result<HttpResponse, Error> {
    get_handler(renderer, ory, req, query, csrf_service, user_session)
        .bind_hub(Hub::current())
        .await
}

#[tracing::instrument(skip(csrf_service))]
pub async fn get_handler(
    renderer: web::Data<Renderer>,
    ory: web::Data<OryClient>,
    req: actix_web::HttpRequest,
    query: web::Query<GetConsentQuery>,
    csrf_service: web::Data<CsrfService<StdRng>>,
    user_session: Session,
) -> Result<HttpResponse, Error> {
    let challenge = match &query.consent_challenge {
        None => {
            tracing::warn!("No consent challenge, going home!");
            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", "/"))
                .finish());
        }
        Some(challenge) => challenge,
    };

    let oauth2_request = ory
        .new_request(GetOAuth2ConsentRequest(challenge.to_owned()))
        .send()
        .await?;

    tracing::info!("Got consent requst");

    if oauth2_request.body.skip.unwrap_or_default()
        || oauth2_request
            .body
            .client
            .as_ref()
            .and_then(|c| c.skip_consent)
            .unwrap_or_default()
    {
        tracing::info!("Can skip consent, approving");

        let grant_scopes = oauth2_request
            .body
            .requested_scope
            .clone()
            .unwrap_or_default();
        let grant_access_token_audience = oauth2_request
            .body
            .requested_access_token_audience
            .clone()
            .unwrap_or_default();

        let session = create_oauth2_consent_request_session(
            &ory,
            &grant_scopes,
            oauth2_request.body,
            &user_session,
        )?;

        // We should skip the flow and auto approve.
        let res = ory
            .new_request(
                AcceptOAuth2ConsentRequest::new(challenge.to_owned())
                    .grant_scope(grant_scopes)
                    .grant_access_token_audience(grant_access_token_audience)
                    .session(session)
                    .build(),
            )
            .send()
            .await?;

        // Then we redirect to the redirect_to url
        return Ok(HttpResponse::SeeOther()
            .append_header(("Location", res.body.redirect_to))
            .finish());
    }

    let mut orig_res = HttpResponse::Ok();
    let (csrf_token, res) =
        csrf_service.add_token(&user_session.id, orig_res.content_type("text/html"))?;

    let html = renderer
        .render("consent.html")
        .var("req", &oauth2_request.body)
        .var("anticsrf_token", &csrf_token)
        .finish()?;

    Ok(res.body(html))
}

#[tracing::instrument]
fn create_oauth2_consent_request_session(
    ory: &OryClient,
    grant_scopes: &[String],
    consent_request: OAuth2ConsentRequest,
    session: &Session,
) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
    let mut id_token = serde_json::Map::new();
    let access_token = serde_json::Map::new();

    if consent_request.subject.is_some()
        && !grant_scopes.is_empty()
        && grant_scopes.contains(&"email".to_owned())
    {
        id_token.insert(
            "email".to_owned(),
            session
                .identity
                .as_ref()
                .expect("Logged in without an identity!")
                .traits
                .get("email")
                .expect("no email!")
                .clone(),
        );
    }

    let mut r = serde_json::Map::new();

    r.insert(
        "access_token".to_owned(),
        serde_json::Value::Object(access_token),
    );
    r.insert("id_token".to_owned(), serde_json::Value::Object(id_token));

    Ok(r)
}

#[derive(Debug, Deserialize)]
pub struct ConsentSubmittedBody {
    csrf_token: CsrfToken,
    consent: bool,
}

impl HasCsrfToken for ConsentSubmittedBody {
    fn get_csrf_token(&self) -> &crate::csrf::CsrfToken {
        &self.csrf_token
    }
}

#[derive(Debug, Deserialize)]
pub struct ConsentSubmittedQuery {
    consent_challenge: String,
}

#[tracing::instrument]
#[post("/consent")]
async fn post_route(
    ory: web::Data<OryClient>,
    renderer: web::Data<Renderer>,
    req: actix_web::HttpRequest,
    query: web::Query<ConsentSubmittedQuery>,
    form: Csrf<web::Form<ConsentSubmittedBody>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    post_handler(ory, renderer, req, query, form, session)
        .bind_hub(Hub::current())
        .await
}

#[tracing::instrument]
async fn post_handler(
    ory: web::Data<OryClient>,
    renderer: web::Data<Renderer>,
    req: actix_web::HttpRequest,
    query: web::Query<ConsentSubmittedQuery>,
    form: Csrf<web::Form<ConsentSubmittedBody>>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let challenge = &query.consent_challenge;
    let oauth2_request = ory
        .new_request(GetOAuth2ConsentRequest(challenge.to_owned()))
        .send()
        .await?;

    if form.consent {
        let grant_scopes = oauth2_request
            .body
            .requested_scope
            .clone()
            .unwrap_or_default();
        let grant_access_token_audience = oauth2_request
            .body
            .requested_access_token_audience
            .clone()
            .unwrap_or_default();
        let oauth_session = create_oauth2_consent_request_session(
            &ory,
            &grant_scopes,
            oauth2_request.body,
            &session,
        )?;

        let res = ory
            .new_request(
                AcceptOAuth2ConsentRequest::new(query.consent_challenge.to_owned())
                    .grant_scope(grant_scopes)
                    .grant_access_token_audience(grant_access_token_audience)
                    .session(oauth_session)
                    .build(),
            )
            .send()
            .await?;

        return Ok(HttpResponse::SeeOther()
            .append_header(("Location", res.body.redirect_to))
            .finish());
    }

    let res = ory
        .new_request(RejectOAuth2ConsentRequest::new(query.consent_challenge.to_owned()).build())
        .send()
        .await?;

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", res.body.redirect_to))
        .finish())
}
