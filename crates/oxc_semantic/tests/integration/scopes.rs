use oxc_semantic::ScopeFlags;

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

#[test]
#[ignore = "We need to fix in another way"]
fn test_function_parameters() {
    SemanticTester::js(
        "
            const foo = 2;
            function func(a = foo, b = a) {
                const foo = 0;
            }
        ",
    )
    .has_root_symbol("foo")
    .has_number_of_references(1)
    .test();
}

#[test]
#[ignore = "We need to fix in another way"]
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
