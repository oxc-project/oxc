use oxc_allocator::Allocator;
use oxc_ast::{ast::ObjectPropertyKind, syntax_directed_operations::PropName, AstKind};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::collections::HashSet;

pub const ORIGINAL_JS_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/eslint/eslint/main/packages/js/src/configs/eslint-all.js";

const UNSUPPORTED_RULES: &[&str] = &["yoda"];

pub fn find_to_be_implemented_rules(source_text: &str) -> Result<Vec<String>, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    let program = allocator.alloc(ret.program);
    let semantic_ret = SemanticBuilder::new(source_text, source_type).build(program);

    let mut rules = vec![];
    let unsupported_rules = UNSUPPORTED_RULES.iter().collect::<HashSet<_>>();

    // This code assumes that the `rules` property appears only once
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
                            if unsupported_rules.contains(&name) {
                                continue;
                            }

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
