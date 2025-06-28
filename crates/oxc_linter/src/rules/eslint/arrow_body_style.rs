use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn arrow_body_style_diagnostic(span: Span, msg: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(msg.to_string()).with_label(span)
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

    pub fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }

    pub fn is_never(&self) -> bool {
        matches!(self, Self::Never)
    }

    pub fn is_as_needed(&self) -> bool {
        matches!(self, Self::AsNeeded)
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
    ///
    /// ### Why is this bad?
    ///
    /// Arrow functions have two syntactic forms for their function bodies.
    /// They may be defined with a block body (denoted by curly braces) () => { ... }
    /// or with a single expression () => ..., whose value is implicitly returned.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the `always` option:
    /// ```js
    /// const foo = () => 0;
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `always` option:
    /// ```js
    /// const foo = () => {
    ///     return 0;
    /// };
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `as-needed` option:
    /// ```js
    /// const foo = () => {
    ///     return 0;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `as-needed` option:
    /// ```js
    /// const foo1 = () => 0;
    ///
    /// const foo2 = (retv, name) => {
    ///     retv[name] = true;
    ///     return retv;
    /// };
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the { "requireReturnForObjectLiteral": true } option:
    /// ```js
    /// /* arrow-body-style: ["error", "as-needed", { "requireReturnForObjectLiteral": true }]*/
    /// const foo = () => ({});
    /// const bar = () => ({ bar: 0 });
    /// ```
    ///
    /// Examples of **correct** code for this rule with the { "requireReturnForObjectLiteral": true } option:
    /// ```js
    /// /* arrow-body-style: ["error", "as-needed", { "requireReturnForObjectLiteral": true }]*/
    /// const foo = () => {};
    /// const bar = () => { return { bar: 0 }; };
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `never` option:
    /// ```js
    /// const foo = () => {
    ///     return 0;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `never` option:
    /// ```js
    /// const foo = () => 0;
    /// const bar = () => ({ foo: 0 });
    /// ```
    ///
    /// ### Options
    ///
    /// The rule takes one or two options. The first is a string, which can be:
    ///
    /// * `always` enforces braces around the function body
    /// * `never` enforces no braces where they can be omitted (default)
    /// * `as-needed` enforces no braces around the function body (constrains arrow functions to the role of returning an expression)
    ///
    /// The second one is an object for more fine-grained configuration
    /// when the first option is "as-needed". Currently,
    /// the only available option is requireReturnForObjectLiteral, a boolean property.
    /// It’s false by default. If set to true, it requires braces and an explicit return for object literals.
    ///
    /// ```json
    /// {
    ///     "arrow-body-style": ["error", "as-needed", { "requireReturnForObjectLiteral": true }]
    /// }
    /// ```
    ArrowBodyStyle,
    eslint,
    style,
    pending,
);

impl Rule for ArrowBodyStyle {
    fn from_configuration(value: Value) -> Self {
        let obj1 = value.get(0);
        let obj2 = value.get(1);

        Self {
            mode: obj1.and_then(Value::as_str).map(Mode::from).unwrap_or_default(),
            require_return_for_object_literal: obj2
                .and_then(|v| v.get("requireReturnForObjectLiteral"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ArrowFunctionExpression(arrow_func_expr) = node.kind() else {
            return;
        };
        let body = &arrow_func_expr.body;
        let statements = &body.statements;

        if arrow_func_expr.expression {
            if self.mode.is_always() {
                ctx.diagnostic(arrow_body_style_diagnostic(
                    body.span,
                    "Expected block statement surrounding arrow body.",
                ));
            }
            if self.mode.is_as_needed() && self.require_return_for_object_literal {
                if let Some(Expression::ObjectExpression(_)) =
                    arrow_func_expr.get_expression().map(Expression::get_inner_expression)
                {
                    ctx.diagnostic(arrow_body_style_diagnostic(
                        body.span,
                        "Expected block statement surrounding arrow body.",
                    ));
                }
            }
        } else {
            if self.mode.is_never() {
                let msg = if statements.is_empty() {
                    "Unexpected block statement surrounding arrow body; put a value of `undefined` immediately after the `=>`."
                } else {
                    "Unexpected block statement surrounding arrow body."
                };
                ctx.diagnostic(arrow_body_style_diagnostic(body.span, msg));
            }
            if self.mode.is_as_needed() {
                // check is there only one `ReturnStatement`
                if statements.len() != 1 {
                    return;
                }
                let inner_statement = &statements[0];

                if let Statement::ReturnStatement(return_statement) = inner_statement {
                    let return_val = &return_statement.argument;
                    if self.require_return_for_object_literal
                        && return_val
                            .as_ref()
                            .is_some_and(|v| matches!(v, Expression::ObjectExpression(_)))
                    {
                        return;
                    }
                    ctx.diagnostic(arrow_body_style_diagnostic(body.span, "Unexpected block statement surrounding arrow body; move the returned value immediately after the `=>`."));
                }
            }
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
