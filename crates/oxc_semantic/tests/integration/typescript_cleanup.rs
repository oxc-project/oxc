use oxc_semantic::{Reference, SymbolFlags};
use oxc_syntax::{node::NodeId, reference::ReferenceFlags, scope::ScopeFlags};
use rustc_hash::FxHashSet;

use crate::util::SemanticTester;

#[test]
fn merged_ambient_declarations_promote_runtime_declaration() {
    for source in [
        "declare var x: number; var x = 1; x++;",
        "var x = 1; declare var x: number; x++;",
        "declare var x: number; var x = 1; var x = 2; x++;",
    ] {
        let tester = SemanticTester::ts(source);
        let mut scoping = tester.build().into_scoping();
        let root = scoping.root_scope_id();
        let id = scoping.get_binding(root, "x".into()).unwrap();
        let surviving: Vec<_> = scoping
            .symbol_redeclarations(id)
            .iter()
            .filter(|d| !d.flags.is_ambient())
            .cloned()
            .collect();
        let refs = scoping.get_resolved_reference_ids(id).to_vec();
        scoping.delete_typescript_bindings();
        assert_eq!(scoping.get_binding(root, "x".into()), Some(id));
        assert_eq!(scoping.symbol_flags(id), SymbolFlags::FunctionScopedVariable);
        assert_eq!(scoping.symbol_span(id), surviving[0].span);
        assert_eq!(scoping.symbol_declaration(id), surviving[0].declaration);
        assert_eq!(
            scoping.symbol_redeclarations(id).len(),
            if surviving.len() == 1 { 0 } else { surviving.len() }
        );
        assert_eq!(scoping.get_resolved_reference_ids(id), refs);
        scoping.delete_typescript_bindings();
        assert_eq!(scoping.get_resolved_reference_ids(id), refs);
    }
}

#[test]
fn erased_bindings_resolve_through_all_erased_scopes() {
    let tester = SemanticTester::ts(
        "const x = 1; { declare const x: number; { declare const x: number; x; } } { declare const y: number; { declare const y: number; y++; } }",
    );
    let mut scoping = tester.build().into_scoping();
    let root = scoping.root_scope_id();
    let x = scoping.get_binding(root, "x".into()).unwrap();
    let references = scoping.references_len();
    let symbols = scoping.symbols_len();
    scoping.delete_typescript_bindings();
    let x_refs = scoping.get_resolved_reference_ids(x);
    assert_eq!(x_refs.len(), 1);
    assert_eq!(scoping.get_reference(x_refs[0]).symbol_id(), Some(x));
    let unresolved = scoping.root_unresolved_references().get("y").unwrap();
    assert_eq!(unresolved.len(), 1);
    let y_ref = unresolved[0];
    assert_eq!(scoping.get_reference(y_ref).symbol_id(), None);
    assert!(scoping.get_reference(y_ref).is_write());
    assert_eq!(scoping.references_len(), references);
    assert_eq!(scoping.symbols_len(), symbols);
    scoping.delete_typescript_bindings();
    assert_eq!(scoping.root_unresolved_references().get("y").unwrap().as_slice(), [y_ref]);
}

#[test]
fn erased_bindings_preserve_parameter_lookup_boundaries() {
    for (source, resolves_to_root) in [
        ("declare const x: number; function f(a = x) { let x = 1; }", false),
        (
            "declare const x: number; function f(a = function g(b = x) { let x = 1; }) { let x = 2; }",
            false,
        ),
        ("const x = 0; { declare const x: number; function f(a = x) { let x = 1; } }", true),
        ("function outer() { declare const x: number; function f(a = x) { let x = 1; } }", false),
        (
            "function outer(a = (() => { declare const x: number; return x; })()) { let x = 1; }",
            false,
        ),
        (
            "const x = 0; function outer(a = (() => { declare const x: number; return x; })()) { let x = 1; }",
            true,
        ),
        (
            "const x = 0; function outer(a = (() => { declare const x: number; return () => x; })()) { var x = 1; }",
            true,
        ),
        (
            "function outer(a = function inner(b = (() => { declare const x: number; return x; })()) { let x = 1; }) { let x = 2; }",
            false,
        ),
        (
            "const x = 0; try {} catch ({ [(() => { declare const x: number; return x; })()]: a }) { let x = 1; }",
            true,
        ),
    ] {
        let tester = SemanticTester::ts(source);
        let mut scoping = tester.build().into_scoping();
        let references: Vec<_> = scoping
            .iter_bindings()
            .flat_map(|(_, bindings)| bindings.values())
            .filter(|&&id| scoping.symbol_flags(id).is_ambient())
            .flat_map(|&id| scoping.get_resolved_reference_ids(id).iter().copied())
            .collect();
        assert_eq!(references.len(), 1);
        scoping.delete_typescript_bindings();
        let expected = if resolves_to_root {
            Some(scoping.get_binding(scoping.root_scope_id(), "x".into()).unwrap())
        } else {
            None
        };
        assert_eq!(scoping.get_reference(references[0]).symbol_id(), expected, "{source}");
        if expected.is_none() {
            assert_eq!(
                scoping.root_unresolved_references().get("x").unwrap().as_slice(),
                references
            );
        }
    }
}

#[test]
fn erased_bindings_preserve_parameter_environments_after_cloning() {
    for source in [
        "function outer(x, a = (() => { declare const x: number; return x; })()) {}",
        "const x = 0; function outer(a = (() => { declare const x: number; return x; })()) { let x = 1; }",
        "const outer = function x(a = (() => { declare const x: number; return x; })()) {};",
        "function outer(a = (() => { declare const x: number; return x; })(), ...x) {}",
        "function enclosing() { const x = 0; function outer(a = (() => { declare const x: number; return x; })()) { let x = 1; } }",
    ] {
        let tester = SemanticTester::ts(source);
        let scoping = tester.build().into_scoping();
        let parameter = scoping
            .iter_bindings()
            .find_map(|(_, bindings)| {
                bindings.get("x").copied().filter(|&id| !scoping.symbol_flags(id).is_ambient())
            })
            .unwrap();
        let cloned = scoping.clone_in_with_semantic_ids_with_another_arena();
        for mut scoping in [scoping, cloned] {
            scoping.delete_typescript_bindings();
            let references = scoping.get_resolved_reference_ids(parameter);
            assert_eq!(references.len(), 1, "{source}");
            assert_eq!(scoping.get_reference(references[0]).symbol_id(), Some(parameter));
            assert!(scoping.root_unresolved_references().is_empty());
        }
    }
}

#[test]
fn generated_references_preserve_enclosing_parameter_environments() {
    let tester = SemanticTester::ts(
        "function outer(a = (() => { declare const x: number; })()) { let x = 1; }",
    );
    let mut scoping = tester.build().into_scoping();
    let erased = scoping
        .iter_bindings()
        .flat_map(|(_, bindings)| bindings.values().copied())
        .find(|&id| scoping.symbol_flags(id).is_ambient())
        .unwrap();
    // Model a transform inserting an intermediate scope and adding a runtime
    // reference inside the parameter's closure.
    let closure_scope = scoping.symbol_scope_id(erased);
    let parent = scoping.scope_parent_id(closure_scope);
    let inserted = scoping.add_scope(parent, NodeId::DUMMY, ScopeFlags::empty());
    scoping.set_scope_parent_id(closure_scope, Some(inserted));
    let reference = scoping.create_reference(Reference::new_with_symbol_id(
        NodeId::DUMMY,
        erased,
        scoping.symbol_scope_id(erased),
        ReferenceFlags::Read,
    ));
    scoping.add_resolved_reference(erased, reference);
    scoping.delete_typescript_bindings();
    assert_eq!(scoping.get_reference(reference).symbol_id(), None);
    assert_eq!(scoping.root_unresolved_references().get("x").unwrap().as_slice(), [reference]);
}

#[test]
fn removing_references_cleans_both_indexes() {
    let tester = SemanticTester::ts("let x = 1; x; missing;");
    let mut scoping = tester.build().into_scoping();
    let id = scoping.get_binding(scoping.root_scope_id(), "x".into()).unwrap();
    let resolved = scoping.get_resolved_reference_ids(id)[0];
    let unresolved = scoping.root_unresolved_references().get("missing").unwrap()[0];
    let removed = FxHashSet::from_iter([resolved, unresolved]);
    scoping.remove_references(&removed);
    assert!(scoping.get_resolved_reference_ids(id).is_empty());
    assert!(scoping.root_unresolved_references().is_empty());
    scoping.remove_references(&removed);
}

#[test]
fn retaining_no_declarations_removes_last_binding() {
    let tester = SemanticTester::ts("function f(): void; f();");
    let mut scoping = tester.build().into_scoping();
    let root = scoping.root_scope_id();
    let id = scoping.get_binding(root, "f".into()).unwrap();
    let reference = scoping.get_resolved_reference_ids(id)[0];
    assert!(!scoping.retain_symbol_declarations(id, |_, _| false));
    let removed = FxHashSet::from_iter([id]);
    scoping.remove_bindings_and_resolve_references(&removed);
    assert!(scoping.get_binding(root, "f".into()).is_none());
    assert_eq!(scoping.get_reference(reference).symbol_id(), None);
    assert_eq!(scoping.root_unresolved_references().get("f").unwrap().as_slice(), [reference]);
    scoping.remove_bindings_and_resolve_references(&removed);
    assert_eq!(scoping.root_unresolved_references().get("f").unwrap().len(), 1);
}

#[test]
fn cleanup_preserves_flags_introduced_by_lowering() {
    let tester = SemanticTester::ts("type f = number; function f() {} f();");
    let mut scoping = tester.build().into_scoping();
    let id = scoping.get_binding(scoping.root_scope_id(), "f".into()).unwrap();
    let runtime = scoping.symbol_redeclarations(id)[1].clone();
    // Model a transform replacing a declaration with a lexical runtime binding.
    *scoping.symbol_flags_mut(id) = SymbolFlags::BlockScopedVariable;
    scoping.delete_typescript_bindings();
    assert_eq!(scoping.symbol_flags(id), SymbolFlags::BlockScopedVariable);
    assert_eq!(scoping.symbol_span(id), runtime.span);
    assert_eq!(scoping.symbol_declaration(id), runtime.declaration);
    assert!(scoping.symbol_redeclarations(id).is_empty());
    assert_eq!(scoping.get_resolved_reference_ids(id).len(), 1);
}
