use std::fs;

use oxc_linter::Oxlintrc;
use oxlint::lsp::options::LintOptions;
use schemars::schema_for;
use website_common::{Renderer, generate_schema_json};

pub fn write_schema_json(path: &str) {
    let json = generate_schema_json::<Oxlintrc>();
    fs::write(path, format!("{json}\n")).expect("failed to write schema JSON");
}

#[test]
#[expect(clippy::disallowed_methods)]
fn test_schema_json() {
    use project_root::get_project_root;
    use std::fs;

    let path = get_project_root().unwrap().join("npm/oxlint/configuration_schema.json");
    let json = generate_schema_json::<Oxlintrc>();
    let existing_json = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        existing_json.trim().replace("\r\n", "\n"),
        json.trim().replace("\r\n", "\n"),
        "The generated schema JSON does not match the existing one. Run `just linter-schema-json` to update it.",
    );
}

#[test]
fn test_schema_markdown() {
    let snapshot = generate_schema_markdown();
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot);
    });
}

#[test]
fn test_schema_markdown_lsp() {
    let snapshot = generate_schema_markdown_lsp();
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot);
    });
}

#[expect(clippy::print_stdout)]
pub fn print_schema_markdown() {
    println!("{}", generate_schema_markdown());
}

#[expect(clippy::print_stdout)]
pub fn print_schema_markdown_lsp() {
    println!("{}", generate_schema_markdown_lsp());
}

fn generate_schema_markdown() -> String {
    let root_schema = schema_for!(Oxlintrc);
    let mut renderer = Renderer::new(root_schema);
    // rules.* and overrides[n].rules should be hidden from the documentation,
    // or else every rules will be listed in the documentation, which is not ideal.
    renderer.with_property_filters(vec!["rules", "overrides[n].rules"]);
    renderer.render()
}

fn generate_schema_markdown_lsp() -> String {
    let root_schema = schema_for!(LintOptions);
    let mut renderer = Renderer::new(root_schema);
    renderer.with_title(false);
    renderer.render()
}
