use oxc_linter::RULES;
use std::collections::HashSet;
use ureq::Response;

mod eslint;
mod typescript;

pub fn run(plugin_name: &str) -> Result<String, String> {
    let (rules_to_be_implemented, rules_implemented) = match plugin_name {
        "eslint" => (eslint::find_to_be_implemented_rules()?, list_implemented_rules("eslint")),
        "typescript" => {
            (typescript::find_to_be_implemented_rules()?, list_implemented_rules("typescript"))
        }
        _ => return Err(format!("😢 Unknown plugin name: {plugin_name}")),
    };

    let list = render_markdown_todo_list(&rules_to_be_implemented, &rules_implemented);
    let list = render_markdown_comment(plugin_name, &list);
    Ok(list)
}

fn fetch_plugin_rules_js_string(url: &str) -> Result<String, String> {
    let body = oxc_tasks_common::agent().get(url).call().map(Response::into_string);

    match body {
        Ok(Ok(body)) => Ok(body),
        Ok(Err(err)) => Err(err.to_string()),
        Err(err) => Err(err.to_string()),
    }
}

fn list_implemented_rules(plugin_name: &str) -> Vec<String> {
    RULES
        .iter()
        .filter(|rule| rule.plugin_name() == plugin_name)
        .map(|rule| rule.name().to_string())
        .collect()
}

fn render_markdown_todo_list(theirs: &[String], ours: &[String]) -> String {
    let mut ours = ours.iter().collect::<HashSet<_>>();

    let mut list = vec![];
    for rule in theirs {
        let mark = if ours.remove(rule) { "x" } else { " " };
        list.push(format!("- [{mark}] {rule}"));
    }

    for rule in &ours {
        eprintln!("⚠️ Rule: {rule} is implemented but not in their lists.");
    }

    list.join("\n")
}

fn render_markdown_comment(plugin_name: &str, list: &str) -> String {
    format!(
        r"
> [!WARNING]
> This comment is maintained by CI. Do not edit this comment directly.
> To update comment template, see https://github.com/oxc-project/oxc/tree/main/tasks/lint_rules

## Getting started

```sh
just new-{plugin_name}-rule <RULE_NAME>
```

Then register the rule in `crates/oxc_linter/src/rules.rs` and also `declare_all_lint_rules` at the bottom.

## Tasks
{list}
"
    )
}
