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

#[test]
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
fn test_multiple_ts_type_alias_declaration() {
    let tester = SemanticTester::ts(
        "
        type A<AB> = AB
        type B<AB> = AB
    ",
    );

    tester
        .has_symbol("AB")
        .contains_flags(SymbolFlags::TypeParameter)
        .has_number_of_references(1)
        .test();
}

#[test]
fn test_function_with_type_parameter() {
    let tester = SemanticTester::ts(
        "
        function mdl<M>() {
            function mdl2() {
                function mdl3() {
                    return '' as M;
                }
            }
        }
        function kfc<K>() {
            function kfc2<K>() {
                function kfc3<K>() {
                    return '' as K;
                }
            }
        }
        ",
    );

    tester.has_symbol("M").has_number_of_references(1).test();
    tester.has_symbol("K").has_number_of_references(0).test();
}

#[test]
fn test_class_with_type_parameter() {
    let tester = SemanticTester::ts(
        "
        class Cls<T, D, F = Array<T>> {
            a: T;
            method<K>(b: T | K) {}
            func() {
                type D = T;
                return null as D
            }
        }
        ",
    );

    tester.has_symbol("T").has_number_of_references(4).test();
    tester.has_symbol("K").has_number_of_references(1).test();
    tester.has_symbol("D").has_number_of_references(0).test();
}

#[test]
fn test_ts_mapped_type() {
    let tester = SemanticTester::ts(
        "
        type M<T> = { [K in keyof T]: T[K] };
        ",
    );

    tester.has_symbol("T").has_number_of_references(2).test();
    tester.has_symbol("K").has_number_of_references(1).test();
}

#[test]
fn test_ts_interface_declaration_with_type_parameter() {
    let tester = SemanticTester::ts(
        "
        type A = any;
        interface ITA<A> {
            a: A;
        }
        interface ITB<B> {
            b: B;
        }
        ",
    );

    tester.has_symbol("A").has_number_of_references(0).test();
    tester.has_symbol("B").has_number_of_references(1).test();
}

#[test]
fn test_ts_infer_type() {
    let tester = SemanticTester::ts(
        "
        type T = T extends infer U ? U : never;
        ",
    );

    tester.has_symbol("T").has_number_of_references(1).test();
    tester.has_symbol("U").has_number_of_references(1).test();
}
