use oxc_linter::Oxlintrc;
use schemars::schema_for;
use website_common::Renderer;

#[expect(clippy::print_stdout)]
pub fn print_schema_json() {
    println!("{}", Oxlintrc::generate_schema_json());
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
    let root_schema = schema_for!(Oxlintrc);
    Renderer::new(root_schema).render()
}
