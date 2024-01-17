use std::{fs::read_dir, path::Path};
use oxc_allocator::Allocator;
use oxc_ast::{ast::ObjectPropertyKind, syntax_directed_operations::PropName, AstKind};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use ureq::Response;

// TODOs:
// - Error handling, message
//   - JS parsing may fail?
// - Use Result properly
// - Support other rules
// - Better AST traversal... :(

const ORIGINAL: &str = "https://raw.githubusercontent.com/eslint/eslint/main/packages/js/src/configs/eslint-recommended.js";
const OURS_DIR: &str = "crates/oxc_linter/src/rules/eslint";

fn main() {
    let js_string = fetch_to_be_implemented_rules_js(ORIGINAL).unwrap();
    let rules_to_be_implemented = find_to_be_implemented_rules(&js_string);

    let rules_implemented = list_implemented_rules(Path::new(OURS_DIR));

    for rule in &rules_to_be_implemented {
        let mark = if rules_implemented.contains(rule) { "x" } else { " " };
        println!("- [{mark}] {rule}");
    }
}

fn fetch_to_be_implemented_rules_js(url: &str) -> Result<String, String> {
    let body = oxc_tasks_common::agent().get(url).call().map(Response::into_string);

    match body {
        Ok(Ok(body)) => Ok(body),
        Err(err) => {
            println!("Failed to fetch {ORIGINAL}");
            Err(err.to_string())
        }
        Ok(Err(err)) => {
            println!("Failed to fetch {ORIGINAL}");
            Err(err.to_string())
        }
    }
}

fn find_to_be_implemented_rules(source_text: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    let program = allocator.alloc(ret.program);
    let semantic_ret = SemanticBuilder::new(source_text, source_type).build(program);

    let mut rules = vec![];
    let mut is_rules = false;
    for node in semantic_ret.semantic.nodes().iter() {
        if let AstKind::IdentifierName(kind) = node.kind() {
            if kind.name == "rules" {
                is_rules = true;
                continue;
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

    rules
}

fn list_implemented_rules(path: &Path) -> Vec<String> {
    let Ok(entries) = read_dir(path) else {
        println!("Failed to read dir {path:?}");
        return vec![];
    };

    let mut rules = vec![];
    for entry in entries.flatten() {
        let os_str = entry.file_name();
        let name = os_str.to_string_lossy().trim_end_matches(".rs").replace('_', "-");
        rules.push(name);
    }

    rules
}

