use oxc_linter::table::RuleTable;

/// Renders the website's Rules page. Each [`category`] gets its own table with
/// links to documentation pages for each rule.
///
/// `docs_prefix` is a path prefix to the base URL all rule documentation pages
/// share in common.
///
/// [`category`]: oxc_linter::RuleCategory
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

**Legend for 'Fixable?' column:**
- 🛠️: an auto-fix is available for this rule
- 💡: a suggestion is available for this rule
- ⚠️🛠️: a dangerous auto-fix is available for this rule
- ⚠️💡: a dangerous suggestion is available for this rule
- 🚧: an auto-fix or suggestion is possible, but currently not implemented

{body}
")
}
