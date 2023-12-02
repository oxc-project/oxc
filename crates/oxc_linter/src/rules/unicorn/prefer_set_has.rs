use oxc_ast::{
    ast::{Argument, Expression, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-set-has):")]
#[diagnostic(severity(warning), help(""))]
struct PreferSetHasDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferSetHas;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    PreferSetHas,
    correctness
);

impl Rule for PreferSetHas {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BindingIdentifier(binding_ident) = node.kind() else {
            return;
        };

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        let AstKind::VariableDeclarator(var_declarator) = parent.kind() else {
            return;
        };
        let Some(var_init) = &var_declarator.init else {
            return;
        };
        let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) else {
            return;
        };
        let AstKind::VariableDeclaration(_var_declaration) = grand_parent.kind() else {
            return;
        };
        if let Some(grand_grand_parent) = ctx.nodes().parent_node(grand_parent.id()) {
            if matches!(
                grand_grand_parent.kind(),
                AstKind::ModuleDeclaration(ModuleDeclaration::ExportNamedDeclaration(_))
            ) {
                return;
            }
        };

        match var_init {
            Expression::ArrayExpression(_) => {}
            Expression::NewExpression(new_expr) => {
                if let Expression::Identifier(ident_ref) = &new_expr.callee {
                    if ident_ref.name != "Array" {
                        return;
                    }
                } else {
                    return;
                }
            }
            Expression::CallExpression(call_expr) => {
                if !(call_expr.callee.is_specific_id("Array")
                    || is_method_call(
                        call_expr,
                        Some(&["Array"]),
                        Some(&["from", "of"]),
                        None,
                        None,
                    )
                    || is_method_call(
                        call_expr,
                        None,
                        Some(&[
                            "concat",
                            "copyWithin",
                            "fill",
                            "filter",
                            "flat",
                            "flatMap",
                            "map",
                            "reverse",
                            "slice",
                            "sort",
                            "splice",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ]),
                        None,
                        None,
                    ))
                    || call_expr
                        .callee
                        .get_member_expr()
                        .is_some_and(oxc_ast::ast::MemberExpression::is_computed)
                {
                    return;
                }
            }
            _ => return,
        }

        let aaa = ctx
            .semantic()
            .symbol_references(binding_ident.symbol_id.get().unwrap())
            .collect::<Vec<_>>();

        if aaa.is_empty() {
            return;
        }

        let some_not_includes_call: bool = aaa.iter().any(|refx| {
            let m = ctx.nodes().get_node(refx.node_id());

            !is_include_call(m, ctx)
        });
        if some_not_includes_call {
            return;
        }

        if aaa.len() == 1
            && aaa.iter().any(|refx| {
                let m = ctx.nodes().get_node(refx.node_id());
                &m;

                let k = !is_multiple_call(m, ctx);
                k;
                k
            })
        {
            return;
        }

        ctx.diagnostic(PreferSetHasDiagnostic(binding_ident.span));
    }
}

fn is_include_call(node: &AstNode, ctx: &LintContext) -> bool {
    if let Some(parent) = ctx.nodes().parent_node(node.id()) {
        if let Some(parent) = ctx.nodes().parent_node(parent.id()) {
            if let AstKind::CallExpression(call_expr) = parent.kind() {
                if let Expression::MemberExpression(callee) = &call_expr.callee {
                    if callee.object().span() != node.kind().span() {
                        return false;
                    }

                    if call_expr.arguments.len() != 1 {
                        return false;
                    }
                    if matches!(call_expr.arguments[0], Argument::SpreadElement(_)) {
                        return false;
                    }

                    if !callee.is_computed() && !callee.optional() && !call_expr.optional {
                        if let Some(name) = callee.static_property_name() {
                            if name == "includes" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn is_multiple_call<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let mut node = node;

    loop {
        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
            if matches!(
                parent.kind(),
                AstKind::ForOfStatement(_)
                    | AstKind::ForStatement(_)
                    | AstKind::ForInStatement(_)
                    | AstKind::WhileStatement(_)
                    | AstKind::DoWhileStatement(_)
                    | AstKind::Function(_)
                    | AstKind::ArrowExpression(_)
            ) {
                return true;
            }
            node = parent;
        } else {
            return false;
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
            const foo = new Set([1, 2, 3]);
            function unicorn() {
                return foo.has(1);
            }
        ",
        // Only called once
        r"
                const foo = [1, 2, 3];
                const isExists = foo.includes(1);
            ",
        // r"
        //         while (a) {
        //             const foo = [1, 2, 3];
        //             const isExists = foo.includes(1);
        //         }
        //     ",
        r"
                const foo = [1, 2, 3];
                (() => {})(foo.includes(1));
            ",
        // Not `VariableDeclarator`
        r"
                foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1);
                }
            ",
        r"
        const exists = foo.includes(1);",
        r"
        const exists = [1, 2, 3].includes(1);",
        // Didn't call `includes()`
        r"
        const foo = [1, 2, 3];",
        // Not `CallExpression`
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes;
                }
            ",
        // Not `foo.includes()`
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return includes(foo);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return bar.includes(foo);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo[includes](1);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.indexOf(1) !== -1;
                }
            ",
        // Not only `foo.includes()`
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    foo.includes(1);
                    foo.length = 1;
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    if (foo.includes(1)) {}
                    return foo;
                }
            ",
        // Declared more than once
        // r"
        //         var foo = [1, 2, 3];
        //         var foo = [4, 5, 6];
        //         function unicorn() {
        //             return foo.includes(1);
        //         }
        //     ",
        r"
                const foo = bar;
                function unicorn() {
                    return foo.includes(1);
                }
            ",
        // Extra arguments
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes();
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1, 1);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1, 0);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1, undefined);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(...[1]);
                }
            ",
        // Optional
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo?.includes(1);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes?.(1);
                }
            ",
        r"
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo?.includes?.(1);
                }
            ",
        // Different scope
        r"
                function unicorn() {
                    const foo = [1, 2, 3];
                }
                function unicorn2() {
                    return foo.includes(1);
                }
            ",
        // `export`
        r"
                export const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1);
                }
            ",
        r"
                module.exports = [1, 2, 3];
                function unicorn() {
                    return module.exports.includes(1);
                }
            ",
        //     r"
        //         const foo = [1, 2, 3];
        //         export {foo};
        //         function unicorn() {
        //             return foo.includes(1);
        //         }
        //     ",
        //     r"
        //         const foo = [1, 2, 3];
        //         export default foo;
        //         function unicorn() {
        //             return foo.includes(1);
        //         }
        //     ",
        //     r"
        //         const foo = [1, 2, 3];
        //         export {foo as bar};
        //         function unicorn() {
        //             return foo.includes(1);
        //         }
        //     ",
        //     r"
        //         const foo = [1, 2, 3];
        //         module.exports = foo;
        //         function unicorn() {
        //             return foo.includes(1);
        //         }
        //     ",
        //     r"
        //         const foo = [1, 2, 3];
        //         exports = foo;
        //         function unicorn() {
        //             return foo.includes(1);
        //         }
        //     ",
        //     r"
        //         const foo = [1, 2, 3];
        //         module.exports.foo = foo;
        //         function unicorn() {
        //             return foo.includes(1);
        //         }
        //     ",
        // `Array()`
        r"
                const foo = NotArray(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        // `new Array()`
        r"
                const foo = new NotArray(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        // `Array.from()` / `Array.of()`
        // Not `Array`
        r"
                const foo = NotArray.from({length: 1}, (_, index) => index);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = NotArray.of(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        // Not `Listed`
        r"
            const foo = Array.notListed();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        // Computed
        r"
            const foo = Array[from]({length: 1}, (_, index) => index);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = Array[of](1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        // Not Identifier
        r"
            const foo = 'Array'.from({length: 1}, (_, index) => index);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = 'Array'.of(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = Array['from']({length: 1}, (_, index) => index);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = Array['of'](1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = of(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = from({length: 1}, (_, index) => index);
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.concat;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.copyWithin;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.fill;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.filter;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.flat;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.flatMap;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.map;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.reverse;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.slice;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.sort;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.splice;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.toReversed;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.toSorted;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.toSpliced;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
        r"
                const foo = bar.with;
                function unicorn() {
                    return foo.includes(1);
                }
        ",
    ];

    let fail = vec![
        r"
    const foo = [1, 2, 3];
    function unicorn() {
        return foo.includes(1);
    }
        ",
        // Called multiple times
        r"
    const foo = [1, 2, 3];
    const isExists = foo.includes(1);
    const isExists2 = foo.includes(2);
        ",
        // `ForOfStatement`
        r"
    const foo = [1, 2, 3];
    for (const a of b) {
        foo.includes(1);
    }
        ",
        r"
    async function unicorn() {
        const foo = [1, 2, 3];
        for await (const a of b) {
            foo.includes(1);
        }
    }
        ",
        // `ForStatement`
        r"
    const foo = [1, 2, 3];
    for (let i = 0; i < n; i++) {
        foo.includes(1);
    }
        ",
        // `ForInStatement`
        r"
    const foo = [1, 2, 3];
    for (let a in b) {
        foo.includes(1);
    }
        ",
        // `WhileStatement`
        r"
    const foo = [1, 2, 3];
    while (a)  {
        foo.includes(1);
    }
        ",
        // `DoWhileStatement`
        r"
    const foo = [1, 2, 3];
    do {
        foo.includes(1);
    } while (a)
        ",
        r"
    const foo = [1, 2, 3];
    do {
        // …
    } while (foo.includes(1))
        ",
        // `function` https://github.com/estools/esquery/blob/master/esquery.js#L216
        // `FunctionDeclaration`
        r"
    const foo = [1, 2, 3];
    function unicorn() {
        return foo.includes(1);
    }
        ",
        r"
    const foo = [1, 2, 3];
    function * unicorn() {
        return foo.includes(1);
    }
        ",
        r"
    const foo = [1, 2, 3];
    async function unicorn() {
        return foo.includes(1);
    }
        ",
        r"
    const foo = [1, 2, 3];
    async function * unicorn() {
        return foo.includes(1);
    }
        ",
        // `FunctionExpression`
        r"
    const foo = [1, 2, 3];
    const unicorn = function () {
        return foo.includes(1);
    }
        ",
        // `ArrowFunctionExpression`
        r"
    const foo = [1, 2, 3];
    const unicorn = () => foo.includes(1);
        ",
        r"
    const foo = [1, 2, 3];
    const a = {
        b() {
            return foo.includes(1);
        }
    };
        ",
        r"
    const foo = [1, 2, 3];
    class A {
        b() {
            return foo.includes(1);
        }
    }
        ",
        // SpreadElement
        r"
    const foo = [...bar];
    function unicorn() {
        return foo.includes(1);
    }
    bar.pop();
        ",
        // Multiple references
        r"
    const foo = [1, 2, 3];
    function unicorn() {
        const exists = foo.includes(1);
        function isExists(find) {
            return foo.includes(find);
        }
    }
        ",
        r"
            function wrap() {
                const foo = [1, 2, 3];

                function unicorn() {
                    return foo.includes(1);
                }
            }

            const bar = [4, 5, 6];

            function unicorn() {
                return bar.includes(1);
            }
        ",
        // Different scope
        r"
            const foo = [1, 2, 3];
            function wrap() {
                const exists = foo.includes(1);
                const bar = [1, 2, 3];

                function outer(find) {
                    const foo = [1, 2, 3];
                    while (a) {
                        foo.includes(1);
                    }

                    function inner(find) {
                        const bar = [1, 2, 3];
                        while (a) {
                            const exists = bar.includes(1);
                        }
                    }
                }
            }
        ",
        // `Array()`
        r"
            const foo = Array(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        // `new Array()`
        r"
            const foo = new Array(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        // `Array.from()`
        r"
            const foo = Array.from({length: 1}, (_, index) => index);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        // `Array.of()`
        r"
            const foo = Array.of(1, 2);
            function unicorn() {
                return foo.includes(1);
            }
        ",
        // Methods
        r"
            const foo = bar.concat();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.copyWithin();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.fill();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.filter();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.flat();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.flatMap();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.map();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.reverse();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.slice();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.sort();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.splice();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.toReversed();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.toSorted();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.toSpliced();
            function unicorn() {
                return foo.includes(1);
            }
        ",
        r"
            const foo = bar.with();
            function unicorn() {
                return foo.includes(1);
            }
        ",
    ];

    Tester::new_without_config(PreferSetHas::NAME, pass, fail).test_and_snapshot();
}
