use oxfmt::oxfmtrc::Oxfmtrc;
use schemars::schema_for;
use website_common::{Renderer, generate_schema_json};

#[expect(clippy::print_stdout)]
pub fn print_schema_json() {
    println!("{}", generate_schema_json::<Oxfmtrc>());
}

#[test]
fn test_schema_json() {
    use project_root::get_project_root;
    use std::fs;

    let path = get_project_root().unwrap().join("npm/oxfmt/configuration_schema.json");
    let json = generate_schema_json::<Oxfmtrc>();
    let existing_json = fs::read_to_string(&path).unwrap_or_default();
    if existing_json.trim() != json.trim() {
        fs::write(&path, &json).unwrap();
    }
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(json);
    });
}

#[test]
fn test_schema_markdown() {
    let snapshot = generate_schema_markdown();
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot);
    });
}

#[expect(clippy::print_stdout)]
pub fn print_schema_markdown() {
    println!("{}", generate_schema_markdown());
}

fn generate_schema_markdown() -> String {
    let root_schema = schema_for!(Oxfmtrc);
    Renderer::new(root_schema).render()
}
