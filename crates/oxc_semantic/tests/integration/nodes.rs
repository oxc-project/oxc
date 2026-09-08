use oxc_ast::AstKind;
use oxc_span::SourceType;

use crate::util::SemanticTester;

#[test]
fn for_loop_targets_precede_collection_in_node_order() {
    for operator in ["in", "of"] {
        for build_cfg in [false, cfg!(feature = "cfg")] {
            let code = format!("for (const [x = afterEach(f)] {operator} beforeAll(f)) {{}}");
            let tester = SemanticTester::new(&code, SourceType::mjs()).with_cfg(build_cfg);
            let semantic = tester.build();
            let calls: Vec<_> = semantic
                .nodes()
                .iter()
                .filter_map(|node| {
                    let AstKind::CallExpression(call) = node.kind() else { return None };
                    Some(call.callee.get_identifier_reference().unwrap().name.as_str())
                })
                .collect();
            assert_eq!(calls, ["afterEach", "beforeAll"], "{code}, cfg={build_cfg}");
        }
    }
}
