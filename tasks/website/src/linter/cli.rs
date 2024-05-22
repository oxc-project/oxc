use bpaf::Parser;
use oxc_cli::lint_options;

// <https://oxc-project.github.io/docs/guide/usage/linter/cli.html>
pub fn generate_cli() {
    let markdown = lint_options().to_options().render_markdown("oxlint");
    println!("{markdown}");
}
