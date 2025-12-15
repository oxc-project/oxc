use oxlint::cli::lint_command;
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
    let snapshot = oxlint::cli::lint_command().run_inner(&["--help"]).unwrap_err().unwrap_stdout();
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
    let markdown = lint_command().render_markdown("oxlint");
    generate_cli_docs(&markdown, "oxlint", Some("Handle Warnings"))
}
