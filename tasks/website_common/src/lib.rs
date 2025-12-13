mod schema_markdown;

pub use schema_markdown::{Renderer, Section};

/// Generate CLI documentation from bpaf-generated markdown.
///
/// Takes raw markdown from bpaf's `render_markdown()` and processes it into
/// website-ready format with proper frontmatter and section headers.
///
/// # Arguments
/// * `raw_markdown` - The markdown string from bpaf's render_markdown()
/// * `tool_name` - The name of the tool (e.g., "oxlint", "oxfmt") used to strip the header
/// * `gitignore_note_anchor` - Optional section header after which to insert the gitignore note
///
/// # Returns
/// Processed markdown ready for the website
#[expect(clippy::disallowed_methods)]
pub fn generate_cli_docs(
    raw_markdown: &str,
    tool_name: &str,
    gitignore_note_anchor: Option<&str>,
) -> String {
    // Remove the extra header
    let header = format!("# {tool_name}\n");
    let markdown = raw_markdown.trim_start_matches(header.as_str());

    // Add ---\nsearch: false\n---\n at the top to prevent Vitepress from indexing this file.
    let markdown = format!("---\nsearch: false\n---\n\n{markdown}");

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
            if let Some(line) = line.strip_prefix("**")
                && let Some(line) = line.strip_suffix("**")
            {
                format!("## {line}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Add note about .gitignore only being respected inside Git repositories
    if let Some(anchor) = gitignore_note_anchor {
        let search_pattern = format!("\n\n## {anchor}\n");
        let replacement = format!(
            "\n\n> [!NOTE]\n> `.gitignore` is only respected inside a Git repository.\n\n## {anchor}\n"
        );
        markdown.replace(&search_pattern, &replacement)
    } else {
        markdown
    }
}
