mod schema_json;
mod schema_markdown;

pub use schema_json::generate_schema_json;
pub use schema_markdown::{Renderer, Section};

/// Generate CLI documentation from usage-generated markdown.
///
/// Takes raw markdown from usage's `MarkdownRenderer` and processes it into
/// website-ready format with proper frontmatter and section headers.
///
/// # Arguments
/// * `raw_markdown` - The markdown string from usage's MarkdownRenderer
/// * `tool_name` - The name of the tool (e.g., "oxlint", "oxfmt") used to strip the header
///
/// # Returns
/// Processed markdown ready for the website
#[expect(clippy::disallowed_methods)]
pub fn generate_cli_docs(raw_markdown: &str, tool_name: &str) -> String {
    // Remove the extra header
    let header = format!("# `{tool_name}`\n");
    let markdown = raw_markdown.trim_start_matches(header.as_str());

    // Add ---\nsearch: false\n---\n at the top to prevent Vitepress from indexing this file.
    let markdown = format!("---\nsearch: false\n---\n\n{markdown}");

    markdown.replacen("- **Usage**: ", "## Usage\n\n", 1)
}
