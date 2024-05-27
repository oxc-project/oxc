use oxc_linter::table::RuleTable;

// `cargo run -p website linter-rules > /path/to/oxc/oxc-project.github.io/src/docs/guide/usage/linter/generated-rules.md`
// <https://oxc-project.github.io/docs/guide/usage/linter/rules.html>
pub fn print_rules() {
    let table = RuleTable::new();

    let total = table.total;
    let turned_on_by_default_count = table.turned_on_by_default_count;

    let body = table
        .sections
        .into_iter()
        .map(|section| section.render_markdown_table())
        .collect::<Vec<_>>()
        .join("\n");

    println!("
# Rules

The progress of all rule implementations is tracked [here](https://github.com/oxc-project/oxc/issues/481).

- Total number of rules: {total}
- Rules turned on by default: {turned_on_by_default_count}

<!-- textlint-disable terminology -->

{body}

<!-- textlint-enable -->

");
}
