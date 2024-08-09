use oxc_linter::table::RuleTable;

// `cargo run -p website linter-rules > /path/to/oxc/oxc-project.github.io/src/docs/guide/usage/linter/generated-rules.md`
// <https://oxc.rs/docs/guide/usage/linter/rules.html>
/// `docs_prefix` is a path prefix to the base URL all rule documentation pages
/// share in common.
pub fn render_rules_table(table: &RuleTable, docs_prefix: &str) -> String {
    let total = table.total;
    let turned_on_by_default_count = table.turned_on_by_default_count;

    let body = table
        .sections
        .iter()
        .map(|s| s.render_markdown_table(Some(docs_prefix)))
        .collect::<Vec<_>>()
        .join("\n");

    format!("
# Rules

The progress of all rule implementations is tracked [here](https://github.com/oxc-project/oxc/issues/481).

- Total number of rules: {total}
- Rules turned on by default: {turned_on_by_default_count}

<!-- textlint-disable terminology -->

{body}

<!-- textlint-enable -->

")
}
