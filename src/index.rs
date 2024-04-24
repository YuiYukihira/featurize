use actix_web::{get, web, HttpRequest, HttpResponse};
use sentry::{Hub, SentryFutureExt};

use crate::{
    ory_client::{LogoutBrowserRequest, OryClient, Session, UserSession},
    renderer::{self, Renderer},
    Error,
};

#[tracing::instrument(skip(session))]
#[get("/")]
pub async fn route(
    renderer: web::Data<Renderer>,
    ory: web::Data<OryClient>,
    session: Result<UserSession, Error>,
) -> Result<HttpResponse, Error> {
    let hub = Hub::current();
    handler(renderer, ory, session).bind_hub(hub).await
}

#[tracing::instrument(skip(session))]
pub async fn handler(
    renderer: web::Data<Renderer>,
    ory: web::Data<OryClient>,
    session: Result<UserSession, Error>,
) -> Result<HttpResponse, Error> {
    let render = match session {
        Err(Error::NoSession) => Ok(renderer.render("index.html")),
        Err(e) => Err(e),
        Ok(session) => {
            let logout_url = ory
                .new_request(LogoutBrowserRequest)
                .cookie(&session.cookie)
                .send()
                .await?;
            Ok(renderer
                .render("home.html")
                .var("logout_url", &logout_url.body.logout_url))
        }
    }?;
    Ok(render.ok().finish()?)
}
