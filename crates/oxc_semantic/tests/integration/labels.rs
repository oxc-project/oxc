use oxc_ast::{AstKind, ast::LabelIdentifier};
use oxc_semantic::{LabelId, Scoping, Semantic};

use crate::util::SemanticTester;

fn label_ids<'a>(semantic: &'a Semantic<'_>) -> Vec<(&'a str, Option<LabelId>)> {
    semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::LabeledStatement(stmt) => Some(&stmt.label),
            AstKind::BreakStatement(stmt) => stmt.label.as_ref(),
            AstKind::ContinueStatement(stmt) => stmt.label.as_ref(),
            _ => None,
        })
        .map(|label: &LabelIdentifier| (label.name.as_str(), label.label_id.get()))
        .collect()
}

fn labels(scoping: &Scoping) -> Vec<(&str, u32)> {
    scoping
        .label_ids()
        .map(|label_id| (scoping.label_name(label_id), scoping.label_depth(label_id)))
        .collect()
}

#[test]
fn test_label_ids() {
    let tester = SemanticTester::js(
        "
        a: {
            b: for (;;) {
                d: for (;;) {
                    function f() {
                        c: for (;;) {
                            a: {
                                break a;
                            }
                            continue c;
                        }
                    }
                    break d;
                    continue b;
                }
            }
            break a;
        }
        b: for (;;) break b;
        for (;;) break;
        ",
    );
    let semantic = tester.build();
    let [label0, label1, label2, label3, label4, label5] =
        [0, 1, 2, 3, 4, 5].map(LabelId::from_usize);

    assert_eq!(
        labels(semantic.scoping()),
        [("a", 0), ("b", 1), ("d", 2), ("c", 3), ("a", 4), ("b", 0)]
    );
    assert_eq!(
        label_ids(&semantic),
        [
            ("a", Some(label0)),
            ("b", Some(label1)),
            ("d", Some(label2)),
            ("c", Some(label3)),
            ("a", Some(label4)),
            ("a", Some(label4)),
            ("c", Some(label3)),
            ("d", Some(label2)),
            ("b", Some(label1)),
            ("a", Some(label0)),
            ("b", Some(label5)),
            ("b", Some(label5)),
        ]
    );
}

#[test]
fn test_unresolved_label_id() {
    let tester = SemanticTester::js("for (;;) { break a; }").expect_errors(true);
    let semantic = tester.build_with_errors().semantic;

    assert_eq!(semantic.scoping().labels_len(), 0);
    assert_eq!(label_ids(&semantic), [("a", None)]);
    let cloned = semantic.scoping().clone_in_with_semantic_ids_with_another_arena();
    for scoping in [semantic.scoping(), &cloned] {
        assert!(scoping.has_unresolved_labels());
    }
}

#[test]
fn test_label_id_does_not_cross_jump_boundary() {
    for source in [
        "a: { function f() { break a; } }",
        "a: { () => { break a; }; }",
        "a: { class C { static { break a; } } }",
    ] {
        let tester = SemanticTester::js(source).expect_errors(true);
        let semantic = tester.build_with_errors().semantic;

        assert_eq!(labels(semantic.scoping()), [("a", 0)], "{source}");
        assert_eq!(
            label_ids(&semantic),
            [("a", Some(LabelId::from_usize(0))), ("a", None)],
            "{source}"
        );
        assert!(semantic.scoping().has_unresolved_labels(), "{source}");
    }
}

#[test]
fn test_labels_survive_clone() {
    let tester = SemanticTester::js("a: { b: { break a; } } c: ;");
    let semantic = tester.build();
    let cloned = semantic.scoping().clone_in_with_semantic_ids_with_another_arena();

    assert_eq!(labels(&cloned), labels(semantic.scoping()));
}
