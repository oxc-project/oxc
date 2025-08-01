use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_index::Idx;
use oxc_parser::Parser;
use oxc_semantic::{ScopeId, Semantic, SemanticBuilder};
use oxc_span::SourceType;

mod conformance;
use conformance::SemanticConformance;

/// A test case representing an input source file.
struct TestContext<'a> {
    pub path: &'a Path,
    pub semantic: Semantic<'a>,
}

fn get_scope_snapshot(semantic: &Semantic, scopes: impl Iterator<Item = ScopeId>) -> String {
    let scope_tree = semantic.scoping();
    let mut result = String::default();

    result.push('[');
    scopes.enumerate().for_each(|(index, scope_id)| {
        if index != 0 {
            result.push(',');
        }
        let flags = scope_tree.scope_flags(scope_id);
        result.push('{');
        let child_ids = semantic
            .scoping()
            .scope_descendants_from_root()
            .filter(|id| {
                scope_tree.scope_parent_id(*id).is_some_and(|parent_id| parent_id == scope_id)
            })
            .collect::<Vec<_>>();
        result.push_str("\"children\":");
        result.push_str(&get_scope_snapshot(semantic, child_ids.iter().copied()));
        result.push(',');
        result.push_str(format!("\"flags\": \"{flags:?}\",").as_str());
        result.push_str(format!("\"id\": {},", scope_id.index()).as_str());
        result.push_str(
            format!(
                "\"node\": {:?},",
                semantic.nodes().kind(scope_tree.get_node_id(scope_id)).debug_name()
            )
            .as_str(),
        );
        result.push_str("\"symbols\": ");
        let mut bindings = scope_tree.get_bindings(scope_id).iter().collect::<Vec<_>>();
        bindings.sort_unstable_by_key(|&(_, symbol_id)| symbol_id);
        result.push('[');
        bindings.iter().enumerate().for_each(|(index, &(name, &symbol_id))| {
            if index != 0 {
                result.push(',');
            }
            result.push('{');
            result.push_str(
                format!("\"flags\": \"{:?}\",", semantic.scoping().symbol_flags(symbol_id))
                    .as_str(),
            );
            result.push_str(format!("\"id\": {},", symbol_id.index()).as_str());
            result.push_str(format!("\"name\": {name:?},").as_str());
            result.push_str(
                format!(
                    "\"node\": {:?},",
                    semantic
                        .nodes()
                        .kind(semantic.scoping().symbol_declaration(symbol_id))
                        .debug_name()
                )
                .as_str(),
            );
            {
                result.push_str("\"references\": ");
                result.push('[');
                semantic
                    .scoping()
                    .get_resolved_reference_ids(symbol_id)
                    .iter()
                    .enumerate()
                    .for_each(|(index, &reference_id)| {
                        if index != 0 {
                            result.push(',');
                        }
                        let reference = &semantic.scoping().get_reference(reference_id);
                        result.push('{');
                        result
                            .push_str(format!("\"flags\": \"{:?}\",", reference.flags()).as_str());
                        result.push_str(format!("\"id\": {},", reference_id.index()).as_str());
                        result.push_str(
                            format!("\"name\": {:?},", semantic.reference_name(reference)).as_str(),
                        );
                        result.push_str(
                            format!("\"node_id\": {}", reference.node_id().index()).as_str(),
                        );
                        result.push('}');
                    });
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

fn run_scope_snapshot_test(ctx: &TestContext<'_>) -> String {
    let scopes = vec![ctx.semantic.scoping().root_scope_id()].into_iter();
    // this is a JSON object
    let scopes = get_scope_snapshot(&ctx.semantic, scopes);

    // pretty-print the results
    let value: serde_json::Value = serde_json::from_str(scopes.as_str()).unwrap();
    serde_json::to_string_pretty(&value).unwrap()
}

fn analyze(
    path: &Path,
    source_text: &str,
    conformance_suite: &SemanticConformance,
) -> (String, String) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let semantic =
        SemanticBuilder::new().with_check_syntax_error(true).build(&ret.program).semantic;
    let ctx = TestContext { path, semantic };
    let scope_snapshot = run_scope_snapshot_test(&ctx);
    let conformance_snapshot = conformance_suite.run_on_source(&ctx);
    (scope_snapshot, conformance_snapshot)
}

/// # Panics
/// cargo test --package oxc_semantic --test main
#[test]
fn main() {
    insta::glob!("fixtures/**/*.{js,jsx,ts,tsx}", |path| {
        let source_text = fs::read_to_string(path).unwrap();
        let conformance_suite = conformance::conformance_suite();
        let (scope_snapshot, conformance_snapshot) =
            analyze(path, &source_text, &conformance_suite);

        let name = path.file_stem().unwrap().to_str().unwrap();

        insta::with_settings!({ snapshot_path => path.parent().unwrap(), prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(name, scope_snapshot);
        });

        if !conformance_snapshot.is_empty() {
            insta::with_settings!({ snapshot_path => path.parent().unwrap(), prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
                insta::assert_snapshot!(format!("{name}.fail"), conformance_snapshot);
            });
        }
    });
}
