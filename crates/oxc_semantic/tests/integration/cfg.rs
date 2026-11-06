#![cfg(all(feature = "cfg", target_pointer_width = "64"))]

use std::fs;

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
fn test_short_circuit_condition_paths() {
    use oxc_ast::AstKind;
    use oxc_cfg::{
        EdgeType, EvalConstConditionResult,
        graph::{
            algo::has_path_connecting,
            visit::{EdgeFiltered, EdgeRef},
        },
    };

    for (source, can_skip_rhs_on_truthy, can_skip_rhs_on_falsy) in [
        ("if (left() && rhs()) truthy(); else falsy();", false, true),
        ("if (left() || rhs()) truthy(); else falsy();", true, false),
        ("while (left() && rhs()) truthy(); falsy();", false, true),
        ("while (left() || rhs()) truthy(); falsy();", true, false),
        ("for (; left() && rhs();) truthy(); falsy();", false, true),
        ("for (; left() || rhs();) truthy(); falsy();", true, false),
        ("left() && rhs() ? truthy() : falsy();", false, true),
        ("left() || rhs() ? truthy() : falsy();", true, false),
        ("do truthy(); while (left() && rhs()); falsy();", false, true),
        ("do truthy(); while (left() || rhs()); falsy();", true, false),
        ("if ((left() && middle()) && rhs()) truthy(); else falsy();", false, true),
        ("if (left() || (middle() || rhs())) truthy(); else falsy();", true, false),
        ("if ((left() || middle()) && rhs()) truthy(); else falsy();", false, true),
        ("if ((left() && middle()) || rhs()) truthy(); else falsy();", true, false),
        ("if (left() ?? rhs()) truthy(); else falsy();", true, true),
        ("const value = left() && rhs(); truthy(value); falsy(value);", true, true),
    ] {
        let tester = SemanticTester::js(source).with_cfg(true);
        let semantic = tester.build();
        let cfg = semantic.cfg().unwrap();
        for node in cfg.graph().node_indices() {
            cfg.is_infinite_loop_start(node, |_| EvalConstConditionResult::Fail);
        }
        let block = |name| {
            semantic
                .nodes()
                .iter()
                .find_map(|node| match node.kind() {
                    AstKind::IdentifierReference(ident) if ident.name == name => {
                        Some(semantic.nodes().cfg_id(node.id()))
                    }
                    _ => None,
                })
                .unwrap()
        };
        let rhs = block("rhs");
        let without_rhs = EdgeFiltered::from_fn(cfg.graph(), |edge| {
            edge.target() != rhs
                && matches!(edge.weight(), EdgeType::Normal | EdgeType::Jump | EdgeType::Backedge)
        });
        for (target, expected) in
            [("truthy", can_skip_rhs_on_truthy), ("falsy", can_skip_rhs_on_falsy)]
        {
            assert_eq!(
                has_path_connecting(&without_rhs, block("left"), block(target), None),
                expected,
                "{source}: {target} without RHS",
            );
            assert!(cfg.is_reachable(block("rhs"), block(target)), "{source}: {target} after RHS");
        }
    }
}

#[test]
fn test_condition_call_is_recorded_once() {
    use oxc_ast::AstKind;
    use oxc_cfg::InstructionKind;

    for source in [
        "if (condition(a && b)) body();",
        "while (condition(a && b)) body();",
        "for (; condition(a && b);) body();",
        "do body(); while (condition(a && b));",
        "condition(a && b) ? truthy() : falsy();",
        "if (left() && condition(a && b)) body();",
        "if (left() || condition(a ? b : c)) body();",
    ] {
        let tester = SemanticTester::js(source).with_cfg(true);
        let semantic = tester.build();
        let call = semantic
            .nodes()
            .iter()
            .find(|node| {
                matches!(node.kind(), AstKind::CallExpression(call)
                if call.callee.is_specific_id("condition"))
            })
            .unwrap();
        let count = semantic
            .cfg()
            .unwrap()
            .basic_blocks
            .iter()
            .flat_map(oxc_cfg::BasicBlock::instructions)
            .filter(|instruction| {
                instruction.kind == InstructionKind::Condition
                    && instruction.node_id == Some(call.id())
            })
            .count();
        assert_eq!(count, 1, "{source}");
    }
}

#[test]
fn test_loop_analysis_uses_complete_condition() {
    use oxc_ast::AstKind;
    use oxc_cfg::EvalConstConditionResult;

    for source in [
        "while (true && condition()) body();",
        "while (condition() || false) body();",
        "while (!(left() && right())) body();",
        "for (; true && condition();) body();",
        "do body(); while (condition() || false);",
    ] {
        let tester = SemanticTester::js(source).with_cfg(true);
        let semantic = tester.build();
        let cfg = semantic.cfg().unwrap();
        for node in cfg.graph().node_indices() {
            assert!(
                cfg.is_infinite_loop_start(node, |instruction| {
                    instruction.node_id.map_or(EvalConstConditionResult::NotFound, |id| {
                        match semantic.nodes().kind(id) {
                            AstKind::BooleanLiteral(lit) => {
                                EvalConstConditionResult::Eval(lit.value)
                            }
                            _ => EvalConstConditionResult::Fail,
                        }
                    })
                })
                .is_none(),
                "{source}",
            );
        }
    }
}

#[test]
fn test_condition_cfg_does_not_require_ast_nodes() {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;

    let source = "
        for (let i = 0; a() && (b() || c()); i++) {
            if (d() || e()) continue;
        }
        do f(); while (g() && h());
        a() && b() ? c() : d();
    ";
    let allocator = Allocator::default();
    let mut expected = None;
    for build_nodes in [true, false] {
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty());
        let built = SemanticBuilder::new_compiler()
            .with_cfg(true)
            .with_build_nodes(build_nodes)
            .build(allocator.alloc(parsed.program));
        assert!(built.diagnostics.is_empty());
        let cfg = format!("{:?}", built.semantic.cfg().unwrap());
        if let Some(expected) = expected.as_ref() {
            assert_eq!(&cfg, expected);
        } else {
            expected = Some(cfg);
        }
    }
}
