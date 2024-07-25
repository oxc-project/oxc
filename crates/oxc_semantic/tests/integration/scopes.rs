use oxc_ast::AstKind;
use oxc_semantic::{ScopeFlags, SymbolFlags};

use crate::util::{Expect, SemanticTester};

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

#[allow(clippy::disallowed_names)]
#[test]
fn test_function_scopes() {
    let test = SemanticTester::ts("function foo() { return foo() }");
    let foo_id = test
        .has_root_symbol("foo")
        .contains_flags(SymbolFlags::Function)
        .contains_flags(SymbolFlags::BlockScopedVariable)
        .has_number_of_reads(1)
        .test();

    let semantic = test.build();
    let root_id = semantic.scopes().root_scope_id();
    let foo_scope_id = semantic.symbols().get_scope_id(foo_id);
    assert_eq!(foo_scope_id, root_id, "Expected fn foo to be in the root scope.");

    let foo_node = semantic.nodes().get_node(semantic.symbols().get_declaration(foo_id));
    let AstKind::Function(foo) = foo_node.kind() else {
        panic!("Expected foo's declaration node to be a FunctionDeclaration");
    };
    assert!(foo.is_declaration(), "Expected foo's declaration node to be a FunctionDeclaration");
    assert_eq!(foo_node.scope_id(), root_id, "Expected fn foo to be in the root scope.");
    assert_ne!(
        foo.scope_id.get().unwrap(),
        root_id,
        "function bodies should not be the root scope"
    );

    let binding_id = semantic
        .scopes()
        .get_binding(root_id, "foo")
        .expect("Expected to find a binding for fn foo");
    assert_eq!(binding_id, foo_id);

    // =========================================================================

    let test = SemanticTester::ts("(function foo() { return foo() })()");
    let foo_id = test
        .has_some_symbol("foo")
        .contains_flags(SymbolFlags::Function)
        .does_not_contain_flags(SymbolFlags::BlockScopedVariable)
        .has_number_of_reads(1)
        .test();

    let semantic = test.build();
    let root_id = semantic.scopes().root_scope_id();

    let foo_node = semantic.nodes().get_node(semantic.symbols().get_declaration(foo_id));
    let foo_scope_id = semantic.symbols().get_scope_id(foo_id);
    assert_eq!(foo_node.scope_id(), root_id);
    // FIXME: These should be equal
    assert_ne!(foo_node.scope_id(), foo_scope_id);
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
