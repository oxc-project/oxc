// <https://oxc-project.github.io/docs/guide/usage/linter-cli.html>
pub fn generate_cli() {
    use bpaf::Parser;
    use oxc_cli::lint_options;
    let markdown = lint_options().to_options().render_markdown("oxlint");
    println!("{markdown}");
}

// <https://oxc-project.github.io/docs/guide/usage/linter-rules.html>
pub fn generate_rules() {
    use oxc_linter::Linter;
    let mut v = vec![];
    Linter::print_rules(&mut v);
    println!("{}", String::from_utf8(v).unwrap());
}

pub fn generate_json_schema() {
    use schemars::schema_for;

    use oxc_linter::ESLintConfig;
    let schema = schema_for!(ESLintConfig);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
