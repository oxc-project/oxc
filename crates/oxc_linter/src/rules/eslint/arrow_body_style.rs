use serde_json::Value;

use oxc_allocator::Box as OxcBox;
use oxc_ast::{
    AstKind,
    ast::{ArrowFunctionExpression, FunctionBody, ReturnStatement},
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn arrow_body_style_diagnostic(span: Span, msg: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(msg.to_string()).with_label(span)
}

fn diagnostic_expected_block(ctx: &LintContext, span: Span) {
    ctx.diagnostic(arrow_body_style_diagnostic(
        span,
        "Expected block statement surrounding arrow body.",
    ));
}

#[derive(Debug, Default, PartialEq, Clone)]
enum Mode {
    #[default]
    AsNeeded,
    Always,
    Never,
}

impl Mode {
    pub fn from(raw: &str) -> Self {
        match raw {
            "always" => Self::Always,
            "never" => Self::Never,
            _ => Self::AsNeeded,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ArrowBodyStyle {
    mode: Mode,
    require_return_for_object_literal: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule can enforce or disallow the use of braces around arrow function body.
    /// Arrow functions can use either:
    /// - a block body `() => { ... }`
    /// - or a concise body `() => expression` with an implicit return.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent use of block vs. concise bodies makes code harder to read.
    /// Concise bodies are limited to a single expression, whose value is implicitly returned.
    ///
    /// ### Options
    ///
    /// First option:
    /// - Type: `string`
    /// - Enum: `"always"`, `"as-needed"`, `"never"`
    /// - Default: `"never"`
    ///
    /// Possible values:
    /// * `never` enforces no braces where they can be omitted (default)
    /// * `always` enforces braces around the function body
    /// * `as-needed` enforces no braces around the function body (constrains arrow functions to the role of returning an expression)
    ///
    /// Second option:
    /// - Type: `object`
    /// - Properties:
    ///     - `requireReturnForObjectLiteral`: `boolean` (default: `false`) - requires braces and an explicit return for object literals.
    ///
    /// Note: This option only applies when the first option is `"as-needed"`.
    ///
    /// Example configuration:
    /// ```json
    /// {
    ///     "arrow-body-style": ["error", "as-needed", { "requireReturnForObjectLiteral": true }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// #### `"never"` (default)
    ///
    /// Examples of **incorrect** code for this rule with the `never` option:
    /// ```js
    /// /* arrow-body-style: ["error", "never"] */
    ///
    /// /* ✘ Bad: */
    /// const foo = () => {
    ///     return 0;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `never` option:
    /// ```js
    /// /* arrow-body-style: ["error", "never"] */
    ///
    /// /* ✔ Good: */
    /// const foo = () => 0;
    /// const bar = () => ({ foo: 0 });
    /// ```
    ///
    /// #### `"always"`
    ///
    /// Examples of **incorrect** code for this rule with the `always` option:
    /// ```js
    /// /* arrow-body-style: ["error", "always"] */
    ///
    /// /* ✘ Bad: */
    /// const foo = () => 0;
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `always` option:
    /// ```js
    /// /* arrow-body-style: ["error", "always"] */
    ///
    /// /* ✔ Good: */
    /// const foo = () => {
    ///     return 0;
    /// };
    /// ```
    ///
    /// #### `"as-needed"`
    ///
    /// Examples of **incorrect** code for this rule with the `as-needed` option:
    /// ```js
    /// /* arrow-body-style: ["error", "as-needed"] */
    ///
    /// /* ✘ Bad: */
    /// const foo = () => {
    ///     return 0;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `as-needed` option:
    /// ```js
    /// /* arrow-body-style: ["error", "as-needed"] */
    ///
    /// /* ✔ Good: */
    /// const foo1 = () => 0;
    ///
    /// const foo2 = (retv, name) => {
    ///     retv[name] = true;
    ///     return retv;
    /// };
    ///
    /// const foo3 = () => {
    ///     bar();
    /// };
    /// ```
    ///
    /// #### `"as-needed"` with `requireReturnForObjectLiteral`
    ///
    /// Examples of **incorrect** code for this rule with the `{ "requireReturnForObjectLiteral": true }` option:
    /// ```js
    /// /* arrow-body-style: ["error", "as-needed", { "requireReturnForObjectLiteral": true }]*/
    ///
    /// /* ✘ Bad: */
    /// const foo = () => ({});
    /// const bar = () => ({ bar: 0 });
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "requireReturnForObjectLiteral": true }` option:
    /// ```js
    /// /* arrow-body-style: ["error", "as-needed", { "requireReturnForObjectLiteral": true }]*/
    ///
    /// /* ✔ Good: */
    /// const foo = () => {};
    /// const bar = () => { return { bar: 0 }; };
    /// ```
    ArrowBodyStyle,
    eslint,
    style,
    pending,
);

impl Rule for ArrowBodyStyle {
    fn from_configuration(value: Value) -> Self {
        let mode = value.get(0).and_then(Value::as_str).map(Mode::from).unwrap_or_default();

        let require_return_for_object_literal = value
            .get(1)
            .and_then(|v| v.get("requireReturnForObjectLiteral"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        Self { mode, require_return_for_object_literal }
    }

    fn run(&self, node: &AstNode, ctx: &LintContext) {
        let AstKind::ArrowFunctionExpression(arrow_func_expr) = node.kind() else {
            return;
        };

        if arrow_func_expr.expression {
            self.run_for_arrow_expression(arrow_func_expr, ctx);
        } else {
            self.run_for_arrow_block(&arrow_func_expr.body, ctx);
        }
    }
}

impl ArrowBodyStyle {
    fn run_for_arrow_expression(
        &self,
        arrow_func_expr: &ArrowFunctionExpression,
        ctx: &LintContext,
    ) {
        let body = &arrow_func_expr.body;

        match (
            &self.mode,
            &self.require_return_for_object_literal,
            arrow_func_expr.get_expression().map(Expression::get_inner_expression),
        ) {
            (Mode::Always, _, _) => diagnostic_expected_block(ctx, body.span),
            (Mode::AsNeeded, true, Some(Expression::ObjectExpression(_))) => {
                diagnostic_expected_block(ctx, body.span);
            }
            _ => {}
        }
    }

    fn run_for_arrow_block_return_statement(
        &self,
        return_statement: &OxcBox<ReturnStatement>,
        body: &FunctionBody,
        ctx: &LintContext,
    ) {
        if self.require_return_for_object_literal
            && matches!(return_statement.argument, Some(Expression::ObjectExpression(_)))
        {
            return;
        }

        ctx.diagnostic(arrow_body_style_diagnostic(
            body.span,
            "Unexpected block statement surrounding arrow body; move the returned value immediately after the `=>`.",
        ));
    }

    fn run_for_arrow_block(&self, body: &FunctionBody, ctx: &LintContext) {
        match self.mode {
            Mode::Never => {
                let msg = if body.statements.is_empty() {
                    "Unexpected block statement surrounding arrow body; put a value of `undefined` immediately after the `=>`."
                } else {
                    "Unexpected block statement surrounding arrow body."
                };
                ctx.diagnostic(arrow_body_style_diagnostic(body.span, msg));
            }
            Mode::AsNeeded if body.statements.len() == 1 => {
                if let Statement::ReturnStatement(return_statement) = &body.statements[0] {
                    self.run_for_arrow_block_return_statement(return_statement, body, ctx);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = () => {};", None),
        ("var foo = () => 0;", None),
        ("var addToB = (a) => { b =  b + a };", None),
        ("var foo = () => { /* do nothing */ };", None),
        (
            "var foo = () => {
			 /* do nothing */
			};",
            None,
        ),
        (
            "var foo = (retv, name) => {
			retv[name] = true;
			return retv;
			};",
            None,
        ),
        ("var foo = () => ({});", None),
        ("var foo = () => bar();", None),
        ("var foo = () => { bar(); };", None),
        ("var foo = () => { b = a };", None),
        ("var foo = () => { bar: 1 };", None),
        ("var foo = () => { return 0; };", Some(serde_json::json!(["always"]))),
        ("var foo = () => { return bar(); };", Some(serde_json::json!(["always"]))),
        ("var foo = () => 0;", Some(serde_json::json!(["never"]))),
        ("var foo = () => ({ foo: 0 });", Some(serde_json::json!(["never"]))),
        (
            "var foo = () => {};",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => 0;",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var addToB = (a) => { b =  b + a };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => { /* do nothing */ };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => {
			 /* do nothing */
			};",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = (retv, name) => {
			retv[name] = true;
			return retv;
			};",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => bar();",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => { bar(); };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => { return { bar: 0 }; };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
    ];

    let fail = vec![
        (
            "for (var foo = () => { return a in b ? bar : () => {} } ;;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        ("a in b; for (var f = () => { return c };;);", Some(serde_json::json!(["as-needed"]))),
        ("for (a = b => { return c in d ? e : f } ;;);", Some(serde_json::json!(["as-needed"]))),
        ("for (var f = () => { return a };;);", Some(serde_json::json!(["as-needed"]))),
        ("for (var f;f = () => { return a };);", Some(serde_json::json!(["as-needed"]))),
        ("for (var f = () => { return a in c };;);", Some(serde_json::json!(["as-needed"]))),
        ("for (var f;f = () => { return a in c };);", Some(serde_json::json!(["as-needed"]))),
        ("for (;;){var f = () => { return a in c }}", Some(serde_json::json!(["as-needed"]))),
        ("for (a = b => { return c = d in e } ;;);", Some(serde_json::json!(["as-needed"]))),
        ("for (var a;;a = b => { return c = d in e } );", Some(serde_json::json!(["as-needed"]))),
        ("for (let a = (b, c, d) => { return vb && c in d; }; ;);", None),
        ("for (let a = (b, c, d) => { return v in b && c in d; }; ;);", None),
        ("function foo(){ for (let a = (b, c, d) => { return v in b && c in d; }; ;); }", None),
        ("for ( a = (b, c, d) => { return v in b && c in d; }; ;);", None),
        ("for ( a = (b) => { return (c in d) }; ;);", None),
        ("for (let a = (b, c, d) => { return vb in dd ; }; ;);", None),
        ("for (let a = (b, c, d) => { return vb in c in dd ; }; ;);", None),
        ("do{let a = () => {return f in ff}}while(true){}", None),
        ("do{for (let a = (b, c, d) => { return vb in c in dd ; }; ;);}while(true){}", None),
        ("scores.map(score => { return x in +(score / maxScore).toFixed(2)});", None),
        ("const fn = (a, b) => { return a + x in Number(b) };", None),
        ("var foo = () => 0", Some(serde_json::json!(["always"]))),
        ("var foo = () => 0;", Some(serde_json::json!(["always"]))),
        ("var foo = () => ({});", Some(serde_json::json!(["always"]))),
        ("var foo = () => (  {});", Some(serde_json::json!(["always"]))),
        ("(() => ({}))", Some(serde_json::json!(["always"]))),
        ("(() => ( {}))", Some(serde_json::json!(["always"]))),
        ("var foo = () => { return 0; };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return 0 };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return bar(); };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => {};", Some(serde_json::json!(["never"]))),
        (
            "var foo = () => {
			return 0;
			};",
            Some(serde_json::json!(["never"])),
        ),
        ("var foo = () => { return { bar: 0 }; };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return ({ bar: 0 }); };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return a, b }", None),
        (
            "var foo = () => { return };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => { return; };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => { return ( /* a */ {ok: true} /* b */ ) };",
            Some(serde_json::json!(["as-needed"])),
        ),
        ("var foo = () => { return '{' };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return { bar: 0 }.bar; };", Some(serde_json::json!(["as-needed"]))),
        (
            "var foo = (retv, name) => {
			retv[name] = true;
			return retv;
			};",
            Some(serde_json::json!(["never"])),
        ),
        ("var foo = () => { bar };", Some(serde_json::json!(["never"]))),
        (
            "var foo = () => { return 0; };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => { return bar(); };",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => ({});",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => ({ bar: 0 });",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        ("var foo = () => (((((((5)))))));", Some(serde_json::json!(["always"]))),
        (
            "var foo = /* a */ ( /* b */ ) /* c */ => /* d */ { /* e */ return /* f */ 5 /* g */ ; /* h */ } /* i */ ;",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = /* a */ ( /* b */ ) /* c */ => /* d */ ( /* e */ 5 /* f */ ) /* g */ ;",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => {
			return bar;
			};",
            None,
        ),
        (
            "var foo = () => {
			return bar;};",
            None,
        ),
        (
            "var foo = () => {return bar;
			};",
            None,
        ),
        (
            "
			              var foo = () => {
			                return foo
			                  .bar;
			              };
			            ",
            None,
        ),
        (
            "
			              var foo = () => {
			                return {
			                  bar: 1,
			                  baz: 2
			                };
			              };
			            ",
            None,
        ),
        ("var foo = () => ({foo: 1}).foo();", Some(serde_json::json!(["always"]))),
        ("var foo = () => ({foo: 1}.foo());", Some(serde_json::json!(["always"]))),
        ("var foo = () => ( {foo: 1} ).foo();", Some(serde_json::json!(["always"]))),
        (
            "
			              var foo = () => ({
			                  bar: 1,
			                  baz: 2
			                });
			            ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "
			              parsedYears = _map(years, (year) => (
			                  {
			                      index : year,
			                      title : splitYear(year)
			                  }
			              ));
			            ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "const createMarker = (color) => ({ latitude, longitude }, index) => {};",
            Some(serde_json::json!(["always"])),
        ),
    ];

    let _fix = vec![
        (
            "for (var foo = () => { return a in b ? bar : () => {} } ;;);",
            "for (var foo = () => (a in b ? bar : () => {}) ;;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "a in b; for (var f = () => { return c };;);",
            "a in b; for (var f = () => c;;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (a = b => { return c in d ? e : f } ;;);",
            "for (a = b => (c in d ? e : f) ;;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (var f = () => { return a };;);",
            "for (var f = () => a;;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (var f;f = () => { return a };);",
            "for (var f;f = () => a;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (var f = () => { return a in c };;);",
            "for (var f = () => (a in c);;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (var f;f = () => { return a in c };);",
            "for (var f;f = () => a in c;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (;;){var f = () => { return a in c }}",
            "for (;;){var f = () => a in c}",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (a = b => { return c = d in e } ;;);",
            "for (a = b => (c = d in e) ;;);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (var a;;a = b => { return c = d in e } );",
            "for (var a;;a = b => c = d in e );",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "for (let a = (b, c, d) => { return vb && c in d; }; ;);",
            "for (let a = (b, c, d) => (vb && c in d); ;);",
            None,
        ),
        (
            "for (let a = (b, c, d) => { return v in b && c in d; }; ;);",
            "for (let a = (b, c, d) => (v in b && c in d); ;);",
            None,
        ),
        (
            "function foo(){ for (let a = (b, c, d) => { return v in b && c in d; }; ;); }",
            "function foo(){ for (let a = (b, c, d) => (v in b && c in d); ;); }",
            None,
        ),
        (
            "for ( a = (b, c, d) => { return v in b && c in d; }; ;);",
            "for ( a = (b, c, d) => (v in b && c in d); ;);",
            None,
        ),
        ("for ( a = (b) => { return (c in d) }; ;);", "for ( a = (b) => (c in d); ;);", None),
        (
            "for (let a = (b, c, d) => { return vb in dd ; }; ;);",
            "for (let a = (b, c, d) => (vb in dd ); ;);",
            None,
        ),
        (
            "for (let a = (b, c, d) => { return vb in c in dd ; }; ;);",
            "for (let a = (b, c, d) => (vb in c in dd ); ;);",
            None,
        ),
        (
            "do{let a = () => {return f in ff}}while(true){}",
            "do{let a = () => f in ff}while(true){}",
            None,
        ),
        (
            "do{for (let a = (b, c, d) => { return vb in c in dd ; }; ;);}while(true){}",
            "do{for (let a = (b, c, d) => (vb in c in dd ); ;);}while(true){}",
            None,
        ),
        (
            "scores.map(score => { return x in +(score / maxScore).toFixed(2)});",
            "scores.map(score => x in +(score / maxScore).toFixed(2));",
            None,
        ),
        (
            "const fn = (a, b) => { return a + x in Number(b) };",
            "const fn = (a, b) => a + x in Number(b);",
            None,
        ),
        ("var foo = () => 0", "var foo = () => {return 0}", Some(serde_json::json!(["always"]))),
        ("var foo = () => 0;", "var foo = () => {return 0};", Some(serde_json::json!(["always"]))),
        (
            "var foo = () => ({});",
            "var foo = () => {return {}};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => (  {});",
            "var foo = () => {return   {}};",
            Some(serde_json::json!(["always"])),
        ),
        ("(() => ({}))", "(() => {return {}})", Some(serde_json::json!(["always"]))),
        ("(() => ( {}))", "(() => {return  {}})", Some(serde_json::json!(["always"]))),
        (
            "var foo = () => { return 0; };",
            "var foo = () => 0;",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return 0 };",
            "var foo = () => 0;",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return bar(); };",
            "var foo = () => bar();",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => {
    	return 0;
    	};",
            "var foo = () => 0;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var foo = () => { return { bar: 0 }; };",
            "var foo = () => ({ bar: 0 });",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return ({ bar: 0 }); };",
            "var foo = () => ({ bar: 0 });",
            Some(serde_json::json!(["as-needed"])),
        ),
        ("var foo = () => { return a, b }", "var foo = () => (a, b)", None),
        (
            "var foo = () => { return ( /* a */ {ok: true} /* b */ ) };",
            "var foo = () => ( /* a */ {ok: true} /* b */ );",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return '{' };",
            "var foo = () => '{';",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return { bar: 0 }.bar; };",
            "var foo = () => ({ bar: 0 }.bar);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return 0; };",
            "var foo = () => 0;",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => { return bar(); };",
            "var foo = () => bar();",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => ({});",
            "var foo = () => {return {}};",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => ({ bar: 0 });",
            "var foo = () => {return { bar: 0 }};",
            Some(serde_json::json!(["as-needed", { "requireReturnForObjectLiteral": true }])),
        ),
        (
            "var foo = () => (((((((5)))))));",
            "var foo = () => {return (((((((5)))))))};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = /* a */ ( /* b */ ) /* c */ => /* d */ { /* e */ return /* f */ 5 /* g */ ; /* h */ } /* i */ ;",
            "var foo = /* a */ ( /* b */ ) /* c */ => /* d */  /* e */  /* f */ 5 /* g */  /* h */  /* i */ ;",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = /* a */ ( /* b */ ) /* c */ => /* d */ ( /* e */ 5 /* f */ ) /* g */ ;",
            "var foo = /* a */ ( /* b */ ) /* c */ => /* d */ {return ( /* e */ 5 /* f */ )} /* g */ ;",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => {
    	return bar;
    	};",
            "var foo = () => bar;",
            None,
        ),
        (
            "var foo = () => {
    	return bar;};",
            "var foo = () => bar;",
            None,
        ),
        (
            "var foo = () => {return bar;
    	};",
            "var foo = () => bar;",
            None,
        ),
        (
            "
    	              var foo = () => {
    	                return foo
    	                  .bar;
    	              };
    	            ",
            "
    	              var foo = () => foo
    	                  .bar;
    	            ",
            None,
        ),
        (
            "
    	              var foo = () => {
    	                return {
    	                  bar: 1,
    	                  baz: 2
    	                };
    	              };
    	            ",
            "
    	              var foo = () => ({
    	                  bar: 1,
    	                  baz: 2
    	                });
    	            ",
            None,
        ),
        (
            "var foo = () => ({foo: 1}).foo();",
            "var foo = () => {return {foo: 1}.foo()};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => ({foo: 1}.foo());",
            "var foo = () => {return {foo: 1}.foo()};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => ( {foo: 1} ).foo();",
            "var foo = () => {return  {foo: 1} .foo()};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "
    	              var foo = () => ({
    	                  bar: 1,
    	                  baz: 2
    	                });
    	            ",
            "
    	              var foo = () => {return {
    	                  bar: 1,
    	                  baz: 2
    	                }};
    	            ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "
    	              parsedYears = _map(years, (year) => (
    	                  {
    	                      index : year,
    	                      title : splitYear(year)
    	                  }
    	              ));
    	            ",
            "
    	              parsedYears = _map(years, (year) => {
    	                  return {
    	                      index : year,
    	                      title : splitYear(year)
    	                  }
    	              });
    	            ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "const createMarker = (color) => ({ latitude, longitude }, index) => {};",
            "const createMarker = (color) => {return ({ latitude, longitude }, index) => {}};",
            Some(serde_json::json!(["always"])),
        ),
    ];

    Tester::new(ArrowBodyStyle::NAME, ArrowBodyStyle::PLUGIN, pass, fail).test_and_snapshot();
}
