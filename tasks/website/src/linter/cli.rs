use oxlint::cli::lint_command;

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
pub fn print_cli() {
    println!("{}", generate_cli());
}

fn generate_cli() -> String {
    let markdown = lint_command().render_markdown("oxlint");
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

    markdown
}
