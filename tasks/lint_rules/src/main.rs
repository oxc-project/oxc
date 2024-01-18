use oxc_allocator::Allocator;
use oxc_ast::{ast::ObjectPropertyKind, syntax_directed_operations::PropName, AstKind};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::{fs::read_dir, path::Path};
use ureq::Response;

// TODOs: Fix up to support other rules

const ORIGINAL: &str =
    "https://raw.githubusercontent.com/eslint/eslint/main/packages/js/src/configs/eslint-all.js";
const OURS_DIR: &str = "crates/oxc_linter/src/rules/eslint";

fn main() {
    let run = |_plugin: &str| -> Result<(), String> {
        let js_string = fetch_plugin_rules_js_string(ORIGINAL)?;

        let rules_to_be_implemented = find_to_be_implemented_rules(&js_string)?;
        let rules_implemented = list_implemented_rules(Path::new(OURS_DIR))?;

        print_markdown_todo_list(&rules_to_be_implemented, &rules_implemented);

        Ok(())
    };

    let plugin = "eslint";

    run(plugin).unwrap_or_else(|err| {
        println!("Failed to run: {plugin}");
        println!("{err}");
    });
}

// Plugin specific
fn find_to_be_implemented_rules(source_text: &str) -> Result<Vec<String>, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    let program = allocator.alloc(ret.program);
    let semantic_ret = SemanticBuilder::new(source_text, source_type).build(program);

    // This code assumes that the `rules` property appears only once
    let mut rules = vec![];
    let mut is_rules = false;
    for node in semantic_ret.semantic.nodes().iter() {
        if let AstKind::ObjectProperty(prop) = node.kind() {
            if let Some((name, _)) = prop.key.prop_name() {
                if name == "rules" {
                    is_rules = true;
                    continue;
                }
            }
        }

        if is_rules {
            if let AstKind::ObjectExpression(obj) = node.kind() {
                for prop in &obj.properties {
                    if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                        if let Some((name, _)) = prop.key.prop_name() {
                            rules.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    if rules.is_empty() {
        return Err("No rules are found".to_string());
    }

    Ok(rules)
}

// Utils

fn fetch_plugin_rules_js_string(url: &str) -> Result<String, String> {
    let body = oxc_tasks_common::agent().get(url).call().map(Response::into_string);

    match body {
        Ok(Ok(body)) => Ok(body),
        Err(err) => Err(err.to_string()),
        Ok(Err(err)) => Err(err.to_string()),
    }
}

fn list_implemented_rules(path: &Path) -> Result<Vec<String>, String> {
    let entries = match read_dir(path) {
        Ok(entries) => entries,
        Err(err) => return Err(err.to_string()),
    };

    let mut rules = vec![];
    for entry in entries.flatten() {
        let os_str = entry.file_name();
        let name = os_str.to_string_lossy().trim_end_matches(".rs").replace('_', "-");
        rules.push(name);
    }

    Ok(rules)
}

fn print_markdown_todo_list(theirs: &[String], ours: &[String]) {
    for rule in theirs {
        let mark = if ours.contains(rule) { "x" } else { " " };
        println!("- [{mark}] {rule}");
    }
}
