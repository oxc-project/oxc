use oxfmt::cli::FormatCommand;
use usage_parser::docs::markdown::{MarkdownRenderer, MarkdownTheme};
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
    let snapshot = FormatCommand::render_help(FormatCommand::command(), true).unwrap();
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot);
    });
}

// <https://oxc.rs/docs/guide/usage/formatter/cli.html>
#[expect(clippy::print_stdout)]
pub fn print_cli() {
    println!("{}", generate_cli());
}

fn generate_cli() -> String {
    let spec: usage_parser::Spec = FormatCommand::to_kdl().parse().unwrap();
    let markdown = MarkdownRenderer::new(spec)
        .with_theme(MarkdownTheme::Compact)
        .with_html_encode(false)
        .render_spec()
        .unwrap();
    generate_cli_docs(&markdown, "oxfmt")
}
