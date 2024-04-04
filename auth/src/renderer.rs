use serde::ser::Serialize;
use tera::{Context, Tera};


#[derive(Debug)]
pub struct Renderer {
    tera: Tera,
    sentry_dsn: String,
}

impl Renderer {
    pub fn new(tera: Tera, sentry_dsn: String) -> Self {
        Self {
            tera,
            sentry_dsn
        }
    }

    pub fn render<S: AsRef<str>>(&self, template_name: S) -> RenderBuilder<'_, S> {
        RenderBuilder::new(self, template_name)
    }
}

pub struct RenderBuilder<'a, S> {
    renderer: &'a Renderer,
    context: Context,
    template: S,
}

impl<'a, S: AsRef<str>> RenderBuilder<'a, S> {
    fn new(renderer: &'a Renderer, template_name: S) -> Self {
        let mut context = Context::new();
        context.insert("sentry_dsn", &renderer.sentry_dsn);
        Self {
            renderer,
            context,
            template: template_name
        }
    }

    pub fn var<K, V>(mut self, name: K, val: &V) -> Self
        where
        K: Into<String>,
    V: Serialize + ?Sized
        {
        self.context.insert(name, val);
        self
    }

    pub fn finish(self) -> tera::Result<String> {
        self.renderer.tera.render(self.template.as_ref(), &self.context)
    }
}
