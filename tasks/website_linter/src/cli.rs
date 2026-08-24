use oxlint::cli::LintCommand;
use usage_parser::docs::markdown::MarkdownRenderer;
use website_common::generate_cli_docs;

#[test]
fn test_cli() {
    let snapshot = generate_cli();
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot);
    });
}

#[test]
fn test_cli_terminal() {
    let snapshot = LintCommand::render_help(LintCommand::command(), true).unwrap();
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot);
    });
}

// <https://oxc.rs/docs/guide/usage/linter/cli.html>
#[expect(clippy::print_stdout)]
pub fn print_cli() {
    println!("{}", generate_cli());
}

fn generate_cli() -> String {
    let mut spec: usage_parser::Spec = LintCommand::to_kdl().parse().unwrap();
    spec.version = None;
    let markdown = MarkdownRenderer::new(spec).with_html_encode(false).render_spec().unwrap();
    generate_cli_docs(&markdown, "oxlint")
}
