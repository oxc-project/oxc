use std::{
    fs::File,
    io::{Error, Write},
    path::{Path, PathBuf},
    process::{Child, Command},
};

use handlebars::Handlebars;

use crate::Context;

const RULE_TEMPLATE: &str = include_str!("../template.txt");

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
            .render_template(RULE_TEMPLATE, &handlebars::to_json(self.context))
            .unwrap();

        let out_path =
            Path::new("crates/oxc_linter/src/rules").join(format!("{}.rs", self.context.rule_name));

        File::create(out_path.clone())?.write_all(rendered.as_bytes())?;
        format_rule_output(out_path)?;

        Ok(())
    }
}

fn format_rule_output(path: PathBuf) -> Result<Child, Error> {
    Command::new("cargo").arg("fmt").arg("--").arg(path).spawn()
}
