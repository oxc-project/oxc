use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_index::Idx;
use oxc_parser::Parser;
use oxc_semantic::{ScopeId, Semantic, SemanticBuilder};
use oxc_span::SourceType;

fn get_scope_snapshot(semantic: &Semantic, scopes: impl Iterator<Item = ScopeId>) -> String {
    let mut result = String::default();

    result.push('[');
    scopes.enumerate().for_each(|(index, scope_id)| {
        if index != 0 {
            result.push(',');
        }
        let flag = semantic.scopes().get_flags(scope_id);
        result.push('{');
        if let Some(child_ids) = semantic.scopes().get_child_ids(scope_id) {
            result.push_str("\"children\":");
            result.push_str(&get_scope_snapshot(semantic, child_ids.iter().copied()));
            result.push(',');
        }
        result.push_str(format!("\"flag\": \"{flag:?}\",").as_str());
        result.push_str(format!("\"id\": {},", scope_id.index()).as_str());
        result.push_str("\"symbols\": ");
        let bindings = semantic.scopes().get_bindings(scope_id);
        result.push('[');
        bindings.iter().enumerate().for_each(|(index, (name, symbol_id))| {
            if index != 0 {
                result.push(',');
            }
            result.push('{');
            result.push_str(
                format!("\"flag\": \"{:?}\",", semantic.symbols().get_flag(*symbol_id)).as_str(),
            );
            result.push_str(format!("\"id\": {},", symbol_id.index()).as_str());
            result.push_str(format!("\"name\": {name:?},").as_str());
            {
                result.push_str("\"references\": ");
                result.push('[');
                semantic.symbol_references(*symbol_id).enumerate().for_each(
                    |(index, reference)| {
                        if index != 0 {
                            result.push(',');
                        }
                        result.push('{');
                        result.push_str(format!("\"flag\": \"{:?}\",", reference.flag()).as_str());
                        result.push_str(format!("\"name\": {:?}", reference.name()).as_str());
                        result.push('}');
                    },
                );
                result.push(']');
            }
            result.push('}');
        });
        result.push(']');
        result.push('}');
    });
    result.push(']');
    result
}

fn analyze(path: &Path, source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();

    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let semantic = SemanticBuilder::new(source_text, source_type).build(&ret.program).semantic;

    let scopes = get_scope_snapshot(&semantic, vec![semantic.scopes().root_scope_id()].into_iter());
    let value: serde_json::Value = serde_json::from_str(scopes.as_str()).unwrap();
    serde_json::to_string_pretty(&value).unwrap()
}

/// # Panics
#[test]
pub fn main() {
    insta::glob!("fixtures/**/*.{ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();
        let snapshot = analyze(path, &source_text);
        let name = path.file_stem().unwrap().to_str().unwrap();
        insta::with_settings!({ snapshot_path => path.parent().unwrap(), prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, snapshot);
        });
    });
}
