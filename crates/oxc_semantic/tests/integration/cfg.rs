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
fn do_while_continue_reaches_condition_and_exit() {
    for source in [
        "do { continue; } while (a); reachable();",
        "do { continue; } while (a && b); reachable();",
        "outer: do { continue outer; } while (a ? b : c); reachable();",
    ] {
        let tester = SemanticTester::js(source).with_cfg(true);
        let semantic = tester.build();
        let cfg = semantic.cfg().unwrap();
        for node in semantic.nodes().iter() {
            if matches!(node.kind(), oxc_ast::AstKind::ExpressionStatement(_)) {
                assert!(
                    !cfg.basic_block(semantic.nodes().cfg_id(node.id())).is_unreachable(),
                    "{source}"
                );
            }
        }
    }
}
