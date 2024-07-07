use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::{
    ast::{Argument, CallExpression, Expression, VariableDeclarationKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

fn warn() -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-unicorn(no-useless-undefined): Do not use useless `undefined`.",
    )
    .with_help("Consider using `null` instead.")
}
fn no_useless_undefined_diagnostic(span0: Span) -> OxcDiagnostic {
    warn().with_label(span0)
}
fn no_useless_undefined_diagnostic_spans(spans: Vec<Span>) -> OxcDiagnostic {
    warn().with_labels(spans)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessUndefined {
    check_arguments: bool,
    check_arrow_function_body: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Do not use useless `undefined`.
    ///
    /// ### Why is this bad?
    /// `undefined` is the default value for new variables, parameters, return statements, etc… so specifying it doesn't make any difference.
    ///
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// let foo = undefined;
    /// // good:
    /// let foo;
    /// ```
    NoUselessUndefined,
    pedantic,
);

const COMPARE_FUNCTION_NAMES: &[&str] = &[
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
];

fn is_match_ignore_func_name(name: &str) -> bool {
    COMPARE_FUNCTION_NAMES.contains(&name)
        // `array.push(undefined)`
        || name == "push"
        // `array.unshift(undefined)`
        || name == "unshift"
		// `array.includes(undefined)`
        || name == "includes"

		// `set.add(undefined)`
        || name == "add"
		// `set.has(undefined)`
        || name == "has"

		// `map.set(foo, undefined)`
        || name == "set"

        // `React.createContext(undefined)`
        || name == "createContext"
		// `setState(undefined)`
        || name.starts_with("set")

        // https://vuejs.org/api/reactivity-core.html#ref
		|| name == "ref"
}

fn should_ignore(callee: &Expression) -> bool {
    match callee {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            return is_match_ignore_func_name(name);
        }
        Expression::StaticMemberExpression(static_assertions) => {
            let name = static_assertions.property.name.as_str();
            return is_match_ignore_func_name(name);
        }
        _ => {
            return false;
        }
    }
}

fn is_function_bind_call(call_expr: &CallExpression) -> bool {
    return !call_expr.optional
        && matches!(&call_expr.callee, Expression::StaticMemberExpression(member_expr)
        if member_expr.property.name.as_str() == "bind");
}

fn is_undefined(arg: &Argument) -> bool {
    if let Argument::Identifier(undefined_literal) = arg {
        if undefined_literal.name == "undefined" {
            return true;
        }
    }
    false
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
                let Some(parent_node) = ctx.nodes().parent_node(node.id()) else { return };
                let parent_node_kind = parent_node.kind();

                // `return undefined`
                if let AstKind::ReturnStatement(ret_stmt) = parent_node_kind {
                    ctx.diagnostic_with_fix(
                        no_useless_undefined_diagnostic(undefined_literal.span),
                        |fixer| {
                            let comments_range = ctx
                                .semantic()
                                .trivias()
                                .comments_range(ret_stmt.span.start..ret_stmt.span.end);
                            if let Some((_, comment)) = comments_range.last() {
                                fixer.delete_range(Span::new(
                                    comment.end + 2,
                                    undefined_literal.span.end,
                                ))
                            } else {
                                fixer.delete_range(Span::new(
                                    ret_stmt.span().start + 6,
                                    undefined_literal.span.end,
                                ))
                            }
                        },
                    );
                // `yield undefined`
                } else if let AstKind::YieldExpression(yield_expr) = parent_node_kind {
                    if yield_expr.delegate {
                        return;
                    }
                    ctx.diagnostic_with_fix(
                        no_useless_undefined_diagnostic(undefined_literal.span),
                        |fixer| fixer.replace(yield_expr.span, "yield"),
                    );
                // `() => undefined`
                } else if let AstKind::ExpressionStatement(_) = parent_node_kind {
                    if !self.check_arrow_function_body {
                        return;
                    }
                    let Some(grand_parent_node) = ctx.nodes().parent_node(parent_node.id()) else {
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
                    let AstKind::ArrowFunctionExpression(_) = grand_grand_parent_node_kind else {
                        return;
                    };

                    ctx.diagnostic_with_fix(
                        no_useless_undefined_diagnostic(undefined_literal.span),
                        |fixer| fixer.replace(func_body.span, "{}"),
                    );
                // `let foo = undefined` / `var foo = undefined`
                } else if let AstKind::VariableDeclarator(variable_declarator) = parent_node_kind {
                    let Some(grand_parent_node) = ctx.nodes().parent_node(parent_node.id()) else {
                        return;
                    };

                    let grand_parent_node_kind = grand_parent_node.kind();
                    let AstKind::VariableDeclaration(_) = grand_parent_node_kind else {
                        return;
                    };
                    if variable_declarator.kind == VariableDeclarationKind::Const {
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
                // `const {foo = undefined} = {}`
                } else if let AstKind::AssignmentPattern(assign_pattern) = parent_node_kind {
                    let left = &assign_pattern.left;
                    let delete_span = Span::new(left.span().end, undefined_literal.span.end);
                    return ctx.diagnostic_with_fix(
                        no_useless_undefined_diagnostic(undefined_literal.span),
                        |fixer| fixer.delete_range(delete_span),
                    );
                } else {
                    return;
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
                if is_function_bind_call(&call_expr) && arguments.len() != 1 {
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
                    end = call_expr.span.end - 1
                }

                let delete_span = Span::new(start, end);
                return ctx.diagnostic_with_fix(
                    no_useless_undefined_diagnostic_spans(undefined_args_spans),
                    |fixer| fixer.delete_range(delete_span),
                );
            }
            _ => {
                return;
            }
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
    ];

    let fail = vec![(r"let foo = undefined;", None)];

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
        ("return undefined;", "return;", None),
    ];

    Tester::new(NoUselessUndefined::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
