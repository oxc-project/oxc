use bpaf::Parser;
use oxc_cli::lint_options;

// <https://oxc-project.github.io/docs/guide/usage/linter/cli.html>
pub fn generate_cli() {
    let markdown = lint_options().to_options().render_markdown("oxlint");
    // Remove the extra header
    let markdown = markdown.trim_start_matches("# oxlint\n");

    // Hack usage line
    let markdown = markdown.replacen("**Usage**:", "## Usage\n", 1);

    let markdown = markdown
        .split('\n')
        .flat_map(|line| {
            // Hack the bug on the line containing `###`
            if line.contains("###") {
                line.split("###").map(str::trim).chain(["\n"]).collect::<Vec<_>>()
            } else {
                vec![line]
            }
        })
        .map(|line| {
            // Make `** title **` to `## title`
            if let Some(line) = line.strip_prefix("**") {
                if let Some(line) = line.strip_suffix("**") {
                    return format!("## {line}");
                }
            }

            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    println!(
        "
<!-- textlint-disable -->

{markdown}

<!-- textlint-enable -->
"
    );
}
