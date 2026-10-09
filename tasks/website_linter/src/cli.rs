use oxlint::cli::lint_command;
use website_common::generate_cli_docs;

// <https://oxc.rs/docs/guide/usage/linter/cli.html>
#[expect(clippy::print_stdout)]
pub fn print_cli() {
    println!("{}", generate_cli());
}

fn generate_cli() -> String {
    let markdown = lint_command().render_markdown("oxlint");
    generate_cli_docs(&markdown, "oxlint")
}
