#![cfg(all(feature = "cfg", target_pointer_width = "64"))]

use std::fs;

use oxc_ast::AstKind;
use oxc_cfg::{EdgeType, InstructionKind};
use oxc_span::SourceType;

use crate::util::SemanticTester;

#[test]
fn test_cfg_files() {
    insta::glob!("cfg_fixtures/*.js", |path| {
        let code = fs::read_to_string(path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let output =
            SemanticTester::new(&code, SourceType::from_path(path).unwrap()).with_cfg(true);
        let snapshot = format!("{}\n\n{}", output.basic_blocks_printed(), output.cfg_dot_diagram());
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "" }, {
            insta::assert_snapshot!(name, snapshot);
        });
    });
}

#[test]
fn for_iteration_targets_run_on_success() {
    for operator in ["in", "of"] {
        for target in [
            "getTarget().x",
            "getTarget()[getKey()]",
            "[getTarget().x = getDefault()]",
            "const { [getKey()]: value = getDefault() }",
        ] {
            for next in ["", "continue;", "break;"] {
                let code =
                    format!("for ({target} {operator} getValues()) {{ body(); {next} }} after();");
                let tester = SemanticTester::new(&code, SourceType::mjs()).with_cfg(true);
                let semantic = tester.build();
                let cfg = semantic.cfg().unwrap();
                let nodes = semantic.nodes();
                let call_block = |name: &str| {
                    nodes
                        .iter()
                        .find_map(|node| {
                            let AstKind::CallExpression(call) = node.kind() else { return None };
                            (call.callee.get_identifier_reference()?.name == name)
                                .then(|| nodes.cfg_id(node.id()))
                        })
                        .unwrap()
                };
                let body = call_block("body");
                let values = call_block("getValues");
                let after = call_block("after");
                let iteration = cfg
                    .graph
                    .node_indices()
                    .find(|&id| {
                        cfg.basic_block(id).instructions.iter().any(|instruction| {
                            matches!(instruction.kind, InstructionKind::Iteration(_))
                        })
                    })
                    .unwrap();
                let has_edge = |from, to, predicate: fn(&EdgeType) -> bool| {
                    cfg.graph.edges_connecting(from, to).any(|edge| predicate(edge.weight()))
                };
                assert!(has_edge(values, iteration, |edge| matches!(edge, EdgeType::Normal)));
                assert!(has_edge(iteration, body, |edge| matches!(edge, EdgeType::Jump)));
                assert!(has_edge(iteration, after, |edge| matches!(edge, EdgeType::Normal)));
                for name in ["getTarget", "getKey", "getDefault"] {
                    if target.contains(name) {
                        assert_eq!(call_block(name), body, "{code}: {name}");
                    }
                }
                match next {
                    "continue;" => {
                        assert!(has_edge(body, iteration, |edge| matches!(edge, EdgeType::Jump)))
                    }
                    "break;" => {
                        assert!(has_edge(body, after, |edge| matches!(edge, EdgeType::Jump)))
                    }
                    _ => assert!(has_edge(body, iteration, |edge| matches!(
                        edge,
                        EdgeType::Backedge
                    ))),
                }
            }
        }
    }
}

#[test]
fn for_in_initializer_runs_before_collection() {
    let tester = SemanticTester::new(
        "for (var key = initialize() in getValues()) { body(); }",
        SourceType::cjs(),
    )
    .with_cfg(true);
    let semantic = tester.build();
    let nodes = semantic.nodes();
    let calls: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.kind(), AstKind::CallExpression(_)))
        .map(|node| nodes.cfg_id(node.id()))
        .collect();
    let [initializer, collection, body] = calls.as_slice() else { panic!("expected three calls") };
    let cfg = semantic.cfg().unwrap();
    assert!(
        cfg.graph
            .edges_connecting(*initializer, *collection)
            .any(|edge| matches!(edge.weight(), EdgeType::Normal))
    );
    assert_ne!(initializer, body);
    assert_ne!(collection, body);
}
