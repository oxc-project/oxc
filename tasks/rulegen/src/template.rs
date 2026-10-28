use std::{
    fs::File,
    io::{Error, Write},
    path::Path,
    process::{Child, Command},
};

use handlebars::Handlebars;

use crate::{Context, RuleKind, get_rule_path};

const RULE_TEMPLATE: &str = include_str!("../template.txt");

pub struct Template<'a> {
    context: &'a Context,
    registry: Handlebars<'a>,
}

impl<'a> Template<'a> {
    pub fn with_context(context: &'a Context) -> Self {
        let mut registry = handlebars::Handlebars::new();
        registry.register_escape_fn(handlebars::no_escape);
        Self { context, registry }
    }

    pub fn render(&self, rule_kind: RuleKind) -> Result<(), Error> {
        let rendered = self
            .registry
            .render_template(RULE_TEMPLATE, &handlebars::to_json(self.context))
            .unwrap();

        let path = get_rule_path(rule_kind);

        std::fs::create_dir_all(&path)?;
        let out_path = path.join(format!("{}.rs", self.context.snake_rule_name));

        File::create(out_path.clone())?.write_all(rendered.as_bytes())?;
        println!("Saved file to {}", out_path.display());

        let res =
            format_rule_output(&out_path).map(|mut child| child.wait().expect("failed to format"));

        match res {
            Ok(exit_status) if exit_status.success() => println!("Formatted rule file"),
            Ok(exit_status) => println!("Failed to format rule file: exited with {exit_status}"),
            Err(e) => println!("Failed to format rule file: {e}"),
        }

        Ok(())
    }
}

fn format_rule_output(path: &Path) -> Result<Child, Error> {
    Command::new("cargo").arg("fmt").arg("--").arg(path).spawn()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, RuleKind};
    use handlebars::Handlebars;

    #[test]
    fn template_render_snapshot() {
        // Construct a representative Context
        let ctx = Context::new(
            RuleKind::ESLint,
            "my-rule",
            // simple pass and fail cases
            "(\"a\")".to_string(),
            "(\"b\")".to_string(),
        )
        .with_language("ts")
        .with_filename(true)
        .with_fix_cases("(\"fixed\")".to_string())
        .with_rule_config(
            r#"#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct ConfigObject {
    pub foo: String,
    pub bar: Option<i32>,
}"#
            .to_string(),
            "(ConfigObject)".to_string(),
            false,
            false,
        );

        let mut registry = Handlebars::new();
        registry.register_escape_fn(handlebars::no_escape);
        let rendered = registry
            .render_template(RULE_TEMPLATE, &handlebars::to_json(&ctx))
            .expect("Failed to render template");

        insta::assert_snapshot!("rulegen_template_render", rendered);
    }
}
