use actix_web::{http::StatusCode, HttpResponse};
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
use serde::ser::Serialize;
use tera::{Context, Tera};

#[derive(Debug)]
pub struct Renderer {
    tera: Tera,
    sentry_dsn: String,
}

impl Renderer {
    pub fn new(tera: Tera, sentry_dsn: String) -> Self {
        Self { tera, sentry_dsn }
    }

    pub fn render<S: AsRef<str>>(&self, template_name: S) -> RenderBuilder<'_, S, NoStatusCode> {
        RenderBuilder::new(self, template_name)
    }
}

pub struct RenderBuilder<'a, S, T> {
    renderer: &'a Renderer,
    context: Context,
    template: S,
    state: T
}

pub struct NoStatusCode;
pub struct WithStatusCode(StatusCode);

impl<'a, S: AsRef<str>> RenderBuilder<'a, S, NoStatusCode> {
    fn new(renderer: &'a Renderer, template_name: S) -> Self {
        let mut context = Context::new();
        context.insert("sentry_dsn", &renderer.sentry_dsn);
        Self {
            renderer,
            context,
            template: template_name,
            state: NoStatusCode
        }
    }

    pub fn var<K, V>(mut self, name: K, val: &V) -> Self
    where
        K: Into<String>,
        V: Serialize + ?Sized,
    {
        self.context.insert(name, val);
        self
    }

    pub fn status(self, status: StatusCode) -> RenderBuilder<'a, S, WithStatusCode> {
        RenderBuilder {
            state: WithStatusCode(status),
            renderer: self.renderer,
            context: self.context,
            template: self.template

        }
    }

    pub fn ok(self) -> RenderBuilder<'a, S, WithStatusCode> {
        self.status(StatusCode::OK)
    }

    pub fn finish(self) -> tera::Result<String> {
        self.renderer
            .tera
            .render(self.template.as_ref(), &self.context)
    }
}

impl<'a, S: AsRef<str>> RenderBuilder<'a, S, WithStatusCode> {
    pub fn finish(self) -> tera::Result<HttpResponse> {
        self.renderer
            .tera
            .render(self.template.as_ref(), &self.context)
            .map(|html| HttpResponse::build(self.state.0)
            .content_type("text/html")
            .body(html))
    }
}
