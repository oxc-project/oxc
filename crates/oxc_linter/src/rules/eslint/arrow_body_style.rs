use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{ArrowFunctionExpression, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn arrow_body_style_diagnostic(span: Span, msg: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(msg.to_string()).with_label(span)
}

const EXPECTED_BLOCK_MSG: &str = "Expected block statement surrounding arrow body.";
const UNEXPECTED_BLOCK_SINGLE_MSG: &str = "Unexpected block statement surrounding arrow body; move the returned value immediately after the `=>`.";

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
    /// - Default: `"as-needed"`
    ///
    /// Possible values:
    /// * `never` enforces no braces around the function body (constrains arrow functions to the role of returning an expression)
    /// * `always` enforces braces around the function body
    /// * `as-needed` enforces no braces where they can be omitted (default)
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
    fix,
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

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ArrowFunctionExpression(arrow_func_expr) = node.kind() else {
            return;
        };

        if arrow_func_expr.expression {
            self.run_for_arrow_expression(arrow_func_expr, ctx);
        } else {
            self.run_for_arrow_block(arrow_func_expr, node, ctx);
        }
    }
}

impl ArrowBodyStyle {
    /// Handle concise arrow body: `() => expr`
    /// Reports when `mode` is "always" or when `mode` is "as-needed" with
    /// `requireReturnForObjectLiteral` and the expression is an object literal.
    fn run_for_arrow_expression<'a>(
        &self,
        arrow_func_expr: &ArrowFunctionExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let body = &arrow_func_expr.body;
        let inner_expr = arrow_func_expr.get_expression().map(Expression::get_inner_expression);

        let should_report = self.mode == Mode::Always
            || (self.mode == Mode::AsNeeded
                && self.require_return_for_object_literal
                && matches!(inner_expr, Some(Expression::ObjectExpression(_))));

        if !should_report {
            return;
        }

        ctx.diagnostic_with_fix(
            arrow_body_style_diagnostic(body.span, EXPECTED_BLOCK_MSG),
            |fixer| Self::fix_concise_to_block(arrow_func_expr, fixer, ctx),
        );
    }

    /// Handle block arrow body: `() => { ... }`
    /// Reports when `mode` is "never" or when `mode` is "as-needed" and the body
    /// contains a single return statement (unless `requireReturnForObjectLiteral`
    /// is true and the returned value is an object literal).
    fn run_for_arrow_block<'a>(
        &self,
        arrow_func_expr: &ArrowFunctionExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let body = &arrow_func_expr.body;

        match self.mode {
            Mode::Never => {
                // Mode::Never: report any block body
                if body.statements.is_empty() {
                    // TODO: implement a fix for empty block bodies
                    ctx.diagnostic(arrow_body_style_diagnostic(
                        body.span,
                        "Unexpected block statement surrounding arrow body; put a value of `undefined` immediately after the `=>`.",
                    ));
                    return;
                }

                // Check if we can fix (single return with argument)
                if body.statements.len() == 1
                    && let Statement::ReturnStatement(return_statement) = &body.statements[0]
                    && let Some(return_arg) = &return_statement.argument
                {
                    ctx.diagnostic_with_fix(
                        arrow_body_style_diagnostic(body.span, UNEXPECTED_BLOCK_SINGLE_MSG),
                        |fixer| {
                            Self::fix_block_to_concise(
                                arrow_func_expr,
                                return_arg,
                                node,
                                fixer,
                                ctx,
                            )
                        },
                    );
                    return;
                }

                // Cannot auto-fix other cases
                ctx.diagnostic(arrow_body_style_diagnostic(
                    body.span,
                    "Unexpected block statement surrounding arrow body.",
                ));
            }
            Mode::AsNeeded if body.statements.len() == 1 => {
                if let Statement::ReturnStatement(return_statement) = &body.statements[0] {
                    // Skip if requireReturnForObjectLiteral and returning an object
                    if self.require_return_for_object_literal
                        && matches!(
                            return_statement.argument,
                            Some(Expression::ObjectExpression(_))
                        )
                    {
                        return;
                    }

                    // Cannot fix if return has no argument (undefined return)
                    let Some(return_arg) = &return_statement.argument else {
                        // TODO: implement a fix for undefined return
                        ctx.diagnostic(arrow_body_style_diagnostic(
                            body.span,
                            UNEXPECTED_BLOCK_SINGLE_MSG,
                        ));
                        return;
                    };

                    ctx.diagnostic_with_fix(
                        arrow_body_style_diagnostic(body.span, UNEXPECTED_BLOCK_SINGLE_MSG),
                        |fixer| {
                            Self::fix_block_to_concise(
                                arrow_func_expr,
                                return_arg,
                                node,
                                fixer,
                                ctx,
                            )
                        },
                    );
                }
            }
            _ => {}
        }
    }

    /// Fix: Convert concise body to block body
    /// `() => expr` → `() => { return expr }`
    fn fix_concise_to_block<'a>(
        arrow_func_expr: &ArrowFunctionExpression<'a>,
        fixer: RuleFixer<'_, 'a>,
        ctx: &LintContext<'a>,
    ) -> RuleFix {
        let body = &arrow_func_expr.body;

        // Get the expression from the concise body
        let Some(expr) = arrow_func_expr.get_expression() else {
            return fixer.noop();
        };

        let expr_text = ctx.source_range(expr.span());

        // Check if the expression is a parenthesized object literal: `() => ({ ... })`
        // In that case, we need to remove the outer parens when converting to block:
        // `() => ({ ... })` → `() => { return { ... } }`
        let inner_expr = expr.get_inner_expression();
        if matches!(inner_expr, Expression::ObjectExpression(_)) {
            let inner_text = ctx.source_range(inner_expr.span());
            return fixer.replace(body.span, format!("{{return {inner_text}}}"));
        }

        // For all other expressions, just wrap in `{ return ... }`
        fixer.replace(body.span, format!("{{return {expr_text}}}"))
    }

    /// Fix: Convert block body to concise body
    /// `() => { return expr }` → `() => expr`
    fn fix_block_to_concise<'a>(
        arrow_func_expr: &ArrowFunctionExpression<'a>,
        return_arg: &Expression<'a>,
        node: &AstNode<'a>,
        fixer: RuleFixer<'_, 'a>,
        ctx: &LintContext<'a>,
    ) -> RuleFix {
        let body = &arrow_func_expr.body;
        let return_arg_text = ctx.source_range(return_arg.span());

        // Get the inner expression to handle cases like `return ({ ... })`
        // where the return value is already parenthesized
        let inner_expr = return_arg.get_inner_expression();
        let is_already_parenthesized = matches!(return_arg, Expression::ParenthesizedExpression(_));

        // Check if expression starts with `{` - needs parens to avoid ambiguity with blocks
        let starts_with_object_literal = Self::starts_with_object_literal(inner_expr);
        let is_sequence_expr = matches!(inner_expr, Expression::SequenceExpression(_));

        if starts_with_object_literal {
            if is_already_parenthesized {
                // Already parenthesized object: `{ return ({ ... }) }` → `({ ... })`
                // Use the full parenthesized text to preserve comments
                return fixer.replace(body.span, return_arg_text.to_string());
            }
            // Bare object literal or expression starting with object needs parentheses
            let inner_text = ctx.source_range(inner_expr.span());
            return fixer.replace(body.span, format!("({inner_text})"));
        }

        if is_sequence_expr {
            if is_already_parenthesized {
                // Already parenthesized sequence: use full text
                return fixer.replace(body.span, return_arg_text.to_string());
            }
            // Sequence expressions need parentheses: `{ return a, b }` → `(a, b)`
            let inner_text = ctx.source_range(inner_expr.span());
            return fixer.replace(body.span, format!("({inner_text})"));
        }

        // Check if we need to wrap in parentheses for `in` operator in for-loop init
        let needs_parens = Self::needs_parens_for_concise_body(return_arg, node, ctx);

        if needs_parens && !is_already_parenthesized {
            // Expression contains `in` and is in for-loop init, needs parentheses
            fixer.replace(body.span, format!("({return_arg_text})"))
        } else {
            // Simple case: just use the return value directly
            // (including if it's already parenthesized)
            fixer.replace(body.span, return_arg_text.to_string())
        }
    }

    /// Check if an expression starts with an object literal.
    /// This includes direct ObjectExpression and expressions that have an
    /// object literal as their leftmost child (e.g., `{ a: 1 }.b`, `{a: 1}.b + c`)
    fn starts_with_object_literal(expr: &Expression) -> bool {
        match expr {
            Expression::ObjectExpression(_) => true,
            Expression::StaticMemberExpression(member) => {
                Self::starts_with_object_literal(&member.object)
            }
            Expression::ComputedMemberExpression(member) => {
                Self::starts_with_object_literal(&member.object)
            }
            Expression::CallExpression(call) => Self::starts_with_object_literal(&call.callee),
            Expression::TaggedTemplateExpression(tagged) => {
                Self::starts_with_object_literal(&tagged.tag)
            }
            // Binary/logical expressions: check the leftmost operand
            Expression::BinaryExpression(bin) => Self::starts_with_object_literal(&bin.left),
            Expression::LogicalExpression(log) => Self::starts_with_object_literal(&log.left),
            // Conditional expression: check the test (leftmost part)
            Expression::ConditionalExpression(cond) => Self::starts_with_object_literal(&cond.test),
            _ => false,
        }
    }

    /// Check if the expression needs parentheses when converting to concise body.
    /// This is needed when the expression contains the `in` operator and the
    /// arrow function is inside a for-loop initializer.
    fn needs_parens_for_concise_body<'a>(
        return_arg: &Expression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        // Check if the expression contains an `in` operator
        if !Self::contains_in_operator(return_arg) {
            return false;
        }

        // Check if the arrow function is inside a for-loop initializer
        Self::is_inside_for_loop_init(node, ctx)
    }

    /// Recursively check if an expression contains the `in` binary operator
    fn contains_in_operator(expr: &Expression) -> bool {
        match expr.get_inner_expression() {
            Expression::BinaryExpression(bin) => {
                if bin.operator == BinaryOperator::In {
                    return true;
                }
                Self::contains_in_operator(&bin.left) || Self::contains_in_operator(&bin.right)
            }
            Expression::ConditionalExpression(cond) => {
                Self::contains_in_operator(&cond.test)
                    || Self::contains_in_operator(&cond.consequent)
                    || Self::contains_in_operator(&cond.alternate)
            }
            Expression::LogicalExpression(log) => {
                Self::contains_in_operator(&log.left) || Self::contains_in_operator(&log.right)
            }
            Expression::AssignmentExpression(assign) => Self::contains_in_operator(&assign.right),
            Expression::SequenceExpression(seq) => {
                seq.expressions.iter().any(Self::contains_in_operator)
            }
            _ => false,
        }
    }

    /// Check if the node is inside a for-loop initializer
    fn is_inside_for_loop_init<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        for ancestor in ctx.nodes().ancestors(node.id()).skip(1) {
            match ancestor.kind() {
                AstKind::ForStatement(for_stmt) => {
                    // Check if the arrow function is in the init part
                    if let Some(init) = &for_stmt.init
                        && let AstKind::ArrowFunctionExpression(arrow) = node.kind()
                        && init.span().contains_inclusive(arrow.span)
                    {
                        return true;
                    }
                    return false;
                }
                // Stop at function boundaries
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::Program(_) => {
                    return false;
                }
                _ => {}
            }
        }
        false
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
        ("var foo = () => { return {a: 1}.b + c };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return {a: 1}.b && c };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return {a: 1}.b || c };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return {a: 1}.b ? c : d };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return {a: 1}.b + c && d };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return {a: 1}.b.c + d };", Some(serde_json::json!(["as-needed"]))),
        ("var foo = () => { return {a: 1}.b() + c };", Some(serde_json::json!(["as-needed"]))),
    ];

    let fix = vec![
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
            "for (let a = (b, c, d) => (vb in dd); ;);",
            None,
        ),
        (
            "for (let a = (b, c, d) => { return vb in c in dd ; }; ;);",
            "for (let a = (b, c, d) => (vb in c in dd); ;);",
            None,
        ),
        (
            "do{let a = () => {return f in ff}}while(true){}",
            "do{let a = () => f in ff}while(true){}",
            None,
        ),
        (
            "do{for (let a = (b, c, d) => { return vb in c in dd ; }; ;);}while(true){}",
            "do{for (let a = (b, c, d) => (vb in c in dd); ;);}while(true){}",
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
            "var foo = () => {return {}};",
            Some(serde_json::json!(["always"])),
        ),
        ("(() => ({}))", "(() => {return {}})", Some(serde_json::json!(["always"]))),
        ("(() => ( {}))", "(() => {return {}})", Some(serde_json::json!(["always"]))),
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
            "var foo = /* a */ ( /* b */ ) /* c */ => /* d */ 5 /* i */ ;",
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
            "var foo = () => {return ({foo: 1}).foo()};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => ({foo: 1}.foo());",
            "var foo = () => {return ({foo: 1}.foo())};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => ( {foo: 1} ).foo();",
            "var foo = () => {return ( {foo: 1} ).foo()};",
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
                      parsedYears = _map(years, (year) => {return {
                              index : year,
                              title : splitYear(year)
                          }});
                    ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "const createMarker = (color) => ({ latitude, longitude }, index) => {};",
            "const createMarker = (color) => {return ({ latitude, longitude }, index) => {}};",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var foo = () => { return {a: 1}.b + c };",
            "var foo = () => ({a: 1}.b + c);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return {a: 1}.b && c };",
            "var foo = () => ({a: 1}.b && c);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return {a: 1}.b || c };",
            "var foo = () => ({a: 1}.b || c);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return {a: 1}.b ? c : d };",
            "var foo = () => ({a: 1}.b ? c : d);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return {a: 1}.b + c && d };",
            "var foo = () => ({a: 1}.b + c && d);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return {a: 1}.b.c + d };",
            "var foo = () => ({a: 1}.b.c + d);",
            Some(serde_json::json!(["as-needed"])),
        ),
        (
            "var foo = () => { return {a: 1}.b() + c };",
            "var foo = () => ({a: 1}.b() + c);",
            Some(serde_json::json!(["as-needed"])),
        ),
    ];

    Tester::new(ArrowBodyStyle::NAME, ArrowBodyStyle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
