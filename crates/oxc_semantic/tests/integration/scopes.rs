use oxc_ast::AstKind;
use oxc_semantic::{ScopeFlags, SymbolFlags};

use crate::util::{Expect, SemanticTester};

#[test]
fn test_only_program() {
    let tester = SemanticTester::js("let x = 1;");
    tester.has_root_symbol("x").is_in_scope(ScopeFlags::Top).test();

    let semantic = tester.build();
    let scopes = semantic.scopes();
    let root = semantic.scopes().root_scope_id();

    // ScopeTree contains a single root scope
    assert_eq!(scopes.len(), 1);
    assert!(!scopes.is_empty());

    // Root scope is associated with the Program
    let root_node_id = scopes.get_node_id(root);
    let root_node = semantic.nodes().get_node(root_node_id);
    assert!(matches!(root_node.kind(), AstKind::Program(_)));

    // ancestors
    assert_eq!(scopes.ancestors(root).count(), 1);
    assert!(scopes.get_parent_id(root).is_none());
}

#[test]
fn test_top_level_strict() {
    // Module with top-level "use strict"
    SemanticTester::js(
        r#"
    "use strict";
    function foo() {
        return 1
    }
    "#,
    )
    .has_root_symbol("foo")
    .is_in_scope(ScopeFlags::Top | ScopeFlags::StrictMode)
    // .expect(expect_strict)
    .test();

    // Module without top-level "use strict"
    SemanticTester::js(
        r"
    function foo() {
        return 1
    }
    ",
    )
    .has_root_symbol("foo")
    .is_in_scope(ScopeFlags::Top | ScopeFlags::StrictMode)
    .test();

    // Script with top-level "use strict"
    SemanticTester::js(
        r#"
    "use strict";
    function foo() {
        return 1
    }
    "#,
    )
    .with_module(false)
    .has_root_symbol("foo")
    .is_in_scope(ScopeFlags::Top | ScopeFlags::StrictMode)
    .test();

    // Script without top-level "use strict"
    SemanticTester::js(
        r"
    function foo() {
        return 1
    }
    ",
    )
    .with_module(false)
    .has_root_symbol("foo")
    .is_in_scope(ScopeFlags::Top)
    .is_not_in_scope(ScopeFlags::StrictMode)
    .test();
}

#[test]
fn test_function_level_strict() {
    let tester = SemanticTester::js(
        r#"
    function foo() {
        "use strict";
        let x = 1;
        return x
    }
    "#,
    )
    .with_module(false);

    tester.has_some_symbol("x")
        .is_in_scope(ScopeFlags::StrictMode | ScopeFlags::Function)
        .expect(|(semantic, symbol_id)| -> Result<(), &'static str> {
            let scope_id = semantic.symbol_scope(symbol_id);
            let Some(parent_scope_id) = semantic.scopes().get_parent_id(scope_id) else {
                return Err("Expected x's scope to have a parent")
            };
            let parent_flags = semantic.scopes().get_flags(parent_scope_id);
            if parent_flags.contains(ScopeFlags::Top) {
                Ok(())
            } else {
                Err("Expected x to be in a top-level function declaration, but its parent scope has flags {parent_flags:?}")
            }
        })
        .test();
    tester.has_some_symbol("foo").is_not_in_scope(ScopeFlags::StrictMode).test();
}

#[test]
fn test_switch_case() {
    SemanticTester::js(
        "
            const foo = 1;
            switch (foo) {
                case 1:
                    const foo = 2;
            }
        ",
    )
    .has_root_symbol("foo")
    .has_number_of_references(1)
    .test();
}

#[test]
fn test_function_parameters() {
    let tester = SemanticTester::js(
        "
            const foo = 2;
            const c = 0;
            function func(a = foo, b = c, c = 0) {
                const foo = 0;
            }
        ",
    );

    tester.has_root_symbol("foo").has_number_of_references(1).test();
    // b = c should reference the third parameter, so root symbol `c`` should have 0 reference
    tester.has_root_symbol("c").has_number_of_references(0).test();
}

#[test]
fn test_catch_clause_parameters() {
    SemanticTester::js(
        "
            const a = 0;
            try {
            } catch ({ [a]: b }) {
                const a = 1;
            }
        ",
    )
    .has_root_symbol("a")
    .has_number_of_references(1)
    .test();
}

#[test]
fn test_enums() {
    let test = SemanticTester::ts(
        "
        enum A {
            X,
            Y,
            Z
        }",
    );
    test.has_root_symbol("A").contains_flags(SymbolFlags::RegularEnum).test();
    test.has_some_symbol("X").contains_flags(SymbolFlags::EnumMember).test();

    let semantic = test.build();
    let program = semantic
        .nodes()
        .iter()
        .find(|node| matches!(node.kind(), AstKind::Program(_)))
        .expect("No program node found");
    assert_eq!(program.scope_id(), semantic.scopes().root_scope_id());

    let (enum_node, enum_decl) = semantic
        .nodes()
        .iter()
        .find_map(|node| {
            let e = node.kind().as_ts_enum_declaration()?;
            Some((node, e))
        })
        .expect("Expected TS test case to have an enum declaration for A.");

    assert_eq!(
        enum_node.scope_id(),
        program.scope_id(),
        "Expected `enum A` to be created in the top-level scope."
    );
    let enum_decl_scope_id = enum_decl.scope_id.get().expect("Enum declaration has no scope id");
    assert_ne!(
        enum_node.scope_id(),
        enum_decl_scope_id,
        "Enum declaration nodes should contain the scope ID they create, not the scope ID they're created in."
    );
    assert_eq!(enum_decl.members.len(), 3);
}

#[test]
fn var_hoisting() {
    SemanticTester::js(
        "
            try {} catch (e) {
                var e = 0;
            }
        ",
    )
    .has_root_symbol("e")
    // `e` was hoisted to the top scope so the symbol's scope is also the top scope
    .is_in_scope(ScopeFlags::Top)
    .test();
}

#[test]
fn get_child_ids() {
    let test = SemanticTester::js(
        "
            function foo() {
            }
        ",
    )
    .with_scope_tree_child_ids(true);
    let semantic = test.build();
    let (_symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

    let child_scope_ids = scopes.get_child_ids(scopes.root_scope_id());
    assert_eq!(child_scope_ids.len(), 1);
    let child_scope_ids = scopes.get_child_ids(child_scope_ids[0]);
    assert!(child_scope_ids.is_empty());
}

#[test]
fn test_ts_conditional_types() {
    SemanticTester::ts("type A<T> = T extends string ? T : false;")
        .has_some_symbol("T")
        .has_number_of_references(2)
        .test();

    // Conditional types create a new scope after check_type.
    SemanticTester::ts(
        "type S<A> = A extends (infer B extends number ? string : never) ? B : false;",
    )
    .has_some_symbol("B")
    .has_number_of_references(1)
    .test();

    // Inferred type parameter is only available within true branch
    SemanticTester::ts("type S<A> = A extends infer R ? never : R")
        .has_some_symbol("R")
        .has_number_of_references(0)
        .test();
}
