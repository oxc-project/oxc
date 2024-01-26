use std::{
    fs::File,
    io::{Error, Write},
    path::Path,
};

use handlebars::Handlebars;

use crate::Context;

const ENV_TEMPLATE: &str = include_str!("../template.hbs");

pub struct Template<'a> {
    context: &'a Context<'a>,
    registry: Handlebars<'a>,
}

impl<'a> Template<'a> {
    pub fn with_context(context: &'a Context) -> Self {
        let mut registry = handlebars::Handlebars::new();
        registry.register_escape_fn(handlebars::no_escape);
        Self { context, registry }
    }

    pub fn render(&self) -> Result<(), Error> {
        let rendered = self
            .registry
            .render_template(ENV_TEMPLATE, &handlebars::to_json(self.context))
            .unwrap();

        let out_path = Path::new("crates/oxc_linter/src/javascript_globals.rs");
        File::create(out_path)?.write_all(rendered.as_bytes())?;

        println!("Saved env file to {out_path:?}");

        Ok(())
    }
}
