use oxc_ast::{
    ast::{Argument, CallExpression, Expression, VariableDeclarationKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn warn() -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use useless `undefined`.")
        .with_help("Consider removing `undefined` or using `null` instead.")
}
fn no_useless_undefined_diagnostic(span: Span) -> OxcDiagnostic {
    warn().with_label(span)
}
fn no_useless_undefined_diagnostic_spans(spans: Vec<Span>) -> OxcDiagnostic {
    warn().with_labels(spans)
}

#[derive(Debug, Clone)]
pub struct NoUselessUndefined {
    check_arguments: bool,
    check_arrow_function_body: bool,
}

impl Default for NoUselessUndefined {
    fn default() -> Self {
        Self { check_arguments: true, check_arrow_function_body: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Do not use useless `undefined`.
    ///
    /// ### Why is this bad?
    ///
    /// `undefined` is the default value for new variables, parameters, return statements, etc… so specifying it doesn't make any difference.
    ///
    /// ### Examples
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let foo = undefined;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let foo;
    /// ```
    NoUselessUndefined,
    pedantic,
    fix
);

// Create a static set for all function names
static FUNCTION_NAMES: phf::Set<&'static str> = phf::phf_set! {
    // Compare function names
    "is",
    "equal",
    "notEqual",
    "strictEqual",
    "notStrictEqual",
    "propertyVal",
    "notPropertyVal",
    "not",
    "include",
    "property",
    "toBe",
    "toHaveBeenCalledWith",
    "toContain",
    "toContainEqual",
    "toEqual",
    "same",
    "notSame",
    "strictSame",
    "strictNotSame",
    // `array.push(undefined)`
    "push",
    // `array.unshift(undefined)`
    "unshift",
    // `array.includes(undefined)`
    "includes",
    // `set.add(undefined)`
    "add",
    // `set.has(undefined)`
    "has",
    // `map.set(foo, undefined)`
    "set",
    // `React.createContext(undefined)`
    "createContext",
    // https://vuejs.org/api/reactivity-core.html#ref
    "ref",
};

fn is_match_ignore_func_name(name: &str) -> bool {
    // Check if the name is in the static set
    FUNCTION_NAMES.contains(name)
        // `setState(undefined)`
        || name.starts_with("set")
}

fn should_ignore(callee: &Expression) -> bool {
    match callee {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            is_match_ignore_func_name(name)
        }
        Expression::StaticMemberExpression(static_assertions) => {
            let name = static_assertions.property.name.as_str();
            is_match_ignore_func_name(name)
        }
        _ => false,
    }
}

fn is_function_bind_call(call_expr: &CallExpression) -> bool {
    !call_expr.optional && is_method_call(call_expr, None, Some(&["bind"]), None, None)
}

fn is_undefined(arg: &Argument) -> bool {
    if !arg.is_expression() {
        return false;
    };
    let expr: &Expression = arg.to_expression();
    if let Expression::Identifier(_) = expr {
        return expr.is_undefined();
    }
    false
}

fn is_has_function_return_type(node: &AstNode, ctx: &LintContext<'_>) -> bool {
    let Some(parent_node) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };
    match parent_node.kind() {
        AstKind::ArrowFunctionExpression(arrow_func_express) => {
            arrow_func_express.return_type.is_some()
        }
        AstKind::Function(func) => func.return_type.is_some(),
        _ => is_has_function_return_type(parent_node, ctx),
    }
}

impl Rule for NoUselessUndefined {
    fn from_configuration(value: serde_json::Value) -> Self {
        let check_arguments =
            value.get("checkArguments").and_then(serde_json::Value::as_bool).unwrap_or(true);
        let check_arrow_function_body = value
            .get("checkArrowFunctionBody")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        Self { check_arguments, check_arrow_function_body }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IdentifierReference(undefined_literal)
                if undefined_literal.name == "undefined" =>
            {
                let mut parent_node: &AstNode<'a> = node;
                while let Some(parent) = ctx.nodes().parent_node(parent_node.id()) {
                    let parent_kind = parent.kind();

                    if let AstKind::ParenthesizedExpression(_) = parent_kind {
                        parent_node = parent;
                    } else {
                        break;
                    }
                }
                let Some(parent_node) = ctx.nodes().parent_node(parent_node.id()) else { return };
                let parent_node_kind = parent_node.kind();

                match parent_node_kind {
                    // `return undefined`
                    AstKind::ReturnStatement(ret_stmt) => {
                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }
                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| {
                                let delete_span = if let Some(comment) = ctx
                                    .semantic()
                                    .comments_range(ret_stmt.span.start..ret_stmt.span.end)
                                    .last()
                                {
                                    Span::new(comment.span.end + 2, undefined_literal.span.end)
                                } else {
                                    Span::new(ret_stmt.span().start + 6, undefined_literal.span.end)
                                };
                                fixer.delete_range(delete_span)
                            },
                        );
                    }
                    // `yield undefined`
                    AstKind::YieldExpression(yield_expr) => {
                        if yield_expr.delegate {
                            return;
                        }
                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| fixer.replace(yield_expr.span, "yield"),
                        );
                    }
                    // `() => undefined`
                    AstKind::ExpressionStatement(_) => {
                        if !self.check_arrow_function_body {
                            return;
                        }
                        let Some(grand_parent_node) = ctx.nodes().parent_node(parent_node.id())
                        else {
                            return;
                        };
                        let grand_parent_node_kind = grand_parent_node.kind();
                        let AstKind::FunctionBody(func_body) = grand_parent_node_kind else {
                            return;
                        };
                        let Some(grand_grand_parent_node) =
                            ctx.nodes().parent_node(grand_parent_node.id())
                        else {
                            return;
                        };
                        let grand_grand_parent_node_kind = grand_grand_parent_node.kind();
                        let AstKind::ArrowFunctionExpression(_) = grand_grand_parent_node_kind
                        else {
                            return;
                        };

                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }

                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| fixer.replace(func_body.span, "{}"),
                        );
                    }
                    // `let foo = undefined` / `var foo = undefined`
                    AstKind::VariableDeclarator(variable_declarator) => {
                        let Some(grand_parent_node) = ctx.nodes().parent_node(parent_node.id())
                        else {
                            return;
                        };
                        let grand_parent_node_kind = grand_parent_node.kind();
                        let AstKind::VariableDeclaration(_) = grand_parent_node_kind else {
                            return;
                        };
                        if variable_declarator.kind == VariableDeclarationKind::Const {
                            return;
                        }
                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }
                        return ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| {
                                fixer.delete_range(Span::new(
                                    variable_declarator.id.span().end,
                                    undefined_literal.span.end,
                                ))
                            },
                        );
                    }
                    // `const {foo = undefined} = {}`
                    AstKind::AssignmentPattern(assign_pattern) => {
                        let left = &assign_pattern.left;
                        let delete_span = Span::new(left.span().end, undefined_literal.span.end);
                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }
                        return ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| fixer.delete_range(delete_span),
                        );
                    }
                    _ => {}
                }
            }
            AstKind::CallExpression(call_expr) => {
                if !self.check_arguments {
                    return;
                }

                if should_ignore(&call_expr.callee) {
                    return;
                }

                let arguments = &call_expr.arguments;

                // Ignore arguments in `Function#bind()`, but not `this` argument
                if is_function_bind_call(call_expr) && arguments.len() != 1 {
                    return;
                }
                let mut undefined_args_spans = Vec::new();
                for i in (0..arguments.len()).rev() {
                    let arg = &arguments[i];
                    if is_undefined(arg) {
                        let span = arg.span();
                        undefined_args_spans.insert(0, span);
                    } else {
                        break;
                    }
                }

                if undefined_args_spans.is_empty() {
                    return;
                }
                let first_undefined_span = undefined_args_spans[0];
                let last_undefined_span = undefined_args_spans[undefined_args_spans.len() - 1];
                let mut start = first_undefined_span.start;
                let mut end = last_undefined_span.end;

                let remaining_count = arguments.len() - undefined_args_spans.len();

                if remaining_count > 0 {
                    let previous_argument = &arguments[remaining_count - 1];
                    start = previous_argument.span().end;
                }
                // If all arguments removed, and there is trailing comma, we need remove it.
                if remaining_count == 0 {
                    end = call_expr.span.end - 1;
                }

                let delete_span = Span::new(start, end);
                return ctx.diagnostic_with_fix(
                    no_useless_undefined_diagnostic_spans(undefined_args_spans),
                    |fixer| fixer.delete_range(delete_span),
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let options_ignore_arguments = || Some(serde_json::json!({ "checkArguments": false }));
    let options_ignore_arrow_function_body =
        || Some(serde_json::json!({"checkArrowFunctionBody": false}));
    let pass = vec![
        (r"function foo() {return;}", None),
        (r"const foo = () => {};", None),
        (r"let foo;", None),
        (r"var foo;", None),
        (r"const foo = undefined;", None),
        (r"foo();", None),
        (r"foo(bar,);", None),
        (r"foo(undefined, bar);", None),
        (r"const {foo} = {};", None),
        (r"function foo({bar} = {}) {}", None),
        (r"function foo(bar) {}", None),
        // I guess nobody uses this, but `yield* undefined;` is valid code, and `yield*;` is not
        (r"function* foo() {yield* undefined;}", None),
        // Ignored
        (r"if (Object.is(foo, undefined)){}", None),
        (r"t.is(foo, undefined)", None),
        (r"assert.equal(foo, undefined, message)", None),
        (r"assert.notEqual(foo, undefined, message)", None),
        (r"assert.strictEqual(foo, undefined, message)", None),
        (r"assert.notStrictEqual(foo, undefined, message)", None),
        (r"assert.propertyVal(foo, 'bar', undefined, message)", None),
        (r"assert.notPropertyVal(foo, 'bar', undefined, message)", None),
        (r"expect(foo).not(undefined)", None),
        (r"expect(foo).to.have.property('bar', undefined)", None),
        (r"expect(foo).toBe(undefined)", None),
        (r"expect(foo).toContain(undefined)", None),
        (r"expect(foo).toContainEqual(undefined)", None),
        (r"expect(foo).toEqual(undefined)", None),
        (r"t.same(foo, undefined)", None),
        (r"t.notSame(foo, undefined)", None),
        (r"t.strictSame(foo, undefined)", None),
        (r"t.strictNotSame(foo, undefined)", None),
        (r"expect(someFunction).toHaveBeenCalledWith(1, 2, undefined);", None),
        (r"set.add(undefined);", None),
        (r"map.set(foo, undefined);", None),
        (r"array.push(foo, undefined);", None),
        (r"array.push(undefined);", None),
        (r"array.unshift(foo, undefined);", None),
        (r"array.unshift(undefined);", None),
        (r"createContext(undefined);", None),
        (r"React.createContext(undefined);", None),
        (r"setState(undefined)", None),
        (r"setState?.(undefined)", None),
        (r"props.setState(undefined)", None),
        (r"props.setState?.(undefined)", None),
        (r"array.includes(undefined)", None),
        (r"set.has(undefined)", None),
        // `Function#bind()`
        (r"foo.bind(bar, undefined);", None),
        (r"foo.bind(...bar, undefined);", None),
        (r"foo.bind(...[], undefined);", None),
        (r"foo.bind(...[undefined], undefined);", None),
        (r"foo.bind(bar, baz, undefined);", None),
        (r"foo?.bind(bar, undefined);", None),
        // `checkArguments: false`
        (r"foo(undefined, undefined);", options_ignore_arguments()),
        (r"foo.bind(undefined);", options_ignore_arguments()),
        // `checkArrowFunctionBody: false`
        (r"const foo = () => undefined", options_ignore_arrow_function_body()),
        (r"const x = { a: undefined }", None),
        // https://github.com/zeit/next.js/blob/3af0fe5cf2542237f34d106872d104c3606b1858/packages/next/build/utils.ts#L620
        (r"prerenderPaths?.add(entry)", None),
        (
            r#"
            function getThing(): string | undefined {
                if (someCondition) {
                    return "hello world";
                }

                return undefined;
            }
        "#,
            None,
        ),
        (
            r#"
            function getThing(): string | undefined {
                if (someCondition) {
                    return "hello world";
                } else if (anotherCondition) {
                    return undefined;
                }

                return undefined;
            }
        "#,
            None,
        ),
        (r"const foo = (): undefined => {return undefined;}", None),
        (r"const foo = (): undefined => undefined;", None),
        (r"const foo = (): string => undefined;", None),
        (r"const foo = function (): undefined {return undefined}", None),
        (r"export function foo(): undefined {return undefined}", None),
        (
            r"
                    const object = {
                        method(): undefined {
                            return undefined;
                        }
                    }
                ",
            None,
        ),
        (
            r"
            class A {
                method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            const A = class A {
                method(): undefined {
                    return undefined
                }
            };
        ",
            None,
        ),
        (
            r"
            class A {
                static method(): undefined {
                    return undefined
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                get method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                static get method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                #method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                private method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (r"createContext<T>(undefined);", None),
        (r"React.createContext<T>(undefined);", None),
        (r"const x = { a: undefined }", None),
        (
            r"
            const y: any = {}
            y.foo = undefined
        ",
            None,
        ),
        (
            r"
            class Foo {
                public x: number | undefined = undefined
            }
        ",
            None,
        ),
    ];

    let fail = vec![
        (
            r"
        foo(
            undefined,
            bar,
            undefined,
            undefined,
            undefined,
            undefined,
        )
        ",
            None,
        ),
        (r"function foo([bar = undefined] = []) {}", None),
        (
            r"
			foo(
				undefined,
				bar,
				undefined,
				undefined,
				undefined,
				undefined,
			)
		",
            None,
        ),
        ("function foo([bar = undefined] = []) {}", None),
        ("foo(bar, undefined, undefined);", None),
        ("let a = undefined, b = 2;", None),
        (
            r"
        function foo() {
            return /* */ (
                /* */
                (
                    /* */
                    undefined
                    /* */
                )
                /* */
            ) /* */ ;
        }
        ",
            None,
        ),
        (
            r"
        function * foo() {
            yield /* */ (
                /* */
                (
                    /* */
                    undefined
                    /* */
                )
                /* */
            ) /* */ ;
        }
        ",
            None,
        ),
        (
            r"
        const foo = () => /* */ (
            /* */
            (
                /* */
                undefined
                /* */
            )
            /* */
        );
        ",
            None,
        ),
        ("foo.bind(undefined)", None),
        ("bind(foo, undefined)", None),
        ("foo.bind?.(bar, undefined)", None),
        ("foo[bind](bar, undefined)", None),
        ("foo.notBind(bar, undefined)", None),
    ];

    let fix = vec![
        (r"function foo() {return undefined;}", r"function foo() {return;}", None),
        ("const foo = () => undefined;", "const foo = () => {};", None),
        ("const foo = () => {return undefined;};", "const foo = () => {return;};", None),
        ("function foo() {return       undefined;}", "function foo() {return;}", None),
        (
            "function foo() {return /* comment */ undefined;}",
            "function foo() {return /* comment */;}",
            None,
        ),
        ("function* foo() {yield undefined;}", "function* foo() {yield;}", None),
        ("function* foo() {yield                 undefined;}", "function* foo() {yield;}", None),
        ("let a = undefined;", "let a;", None),
        ("let a = undefined, b = 2;", "let a, b = 2;", None),
        ("var a = undefined;", "var a;", None),
        ("var a = undefined, b = 2;", "var a, b = 2;", None),
        ("foo(undefined);", "foo();", None),
        ("foo(undefined, undefined);", "foo();", None),
        ("foo(undefined,);", "foo();", None),
        ("foo(undefined, undefined,);", "foo();", None),
        ("foo(bar, undefined);", "foo(bar);", None),
        ("foo(bar, undefined, undefined);", "foo(bar);", None),
        ("foo(undefined, bar, undefined);", "foo(undefined, bar);", None),
        ("foo(bar, undefined,);", "foo(bar,);", None),
        ("foo(undefined, bar, undefined,);", "foo(undefined, bar,);", None),
        ("foo(bar, undefined, undefined,);", "foo(bar,);", None),
        ("foo(undefined, bar, undefined, undefined,);", "foo(undefined, bar,);", None),
        (
            "foo(undefined, bar, undefined, undefined, undefined, undefined,);",
            "foo(undefined, bar,);",
            None,
        ),
        ("const {foo = undefined} = {};", "const {foo} = {};", None),
        ("const [foo = undefined] = [];", "const [foo] = [];", None),
        ("function foo(bar = undefined) {}", "function foo(bar) {}", None),
        ("function foo({bar = undefined}) {}", "function foo({bar}) {}", None),
        ("function foo({bar = undefined} = {}) {}", "function foo({bar} = {}) {}", None),
        ("function foo([bar = undefined]) {}", "function foo([bar]) {}", None),
        ("function foo([bar = undefined] = []) {}", "function foo([bar] = []) {}", None),
        (
            "function foo(x: string | undefined = undefined) { }",
            "function foo(x: string | undefined) { }",
            None,
        ),
        ("return undefined;", "return;", None),
        (
            r"
            function foo():undefined {
					function nested() {
						return undefined;
					}

					return nested();
				}
        ",
            r"
            function foo():undefined {
					function nested() {
						return;
					}

					return nested();
				}
        ",
            None,
        ),
    ];

    Tester::new(NoUselessUndefined::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
