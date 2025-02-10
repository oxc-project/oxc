use oxc_ast::{
    ast::{ChainElement, Expression, UnaryOperator},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_unused_expressions_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow unused expressions")
        .with_help("Consider removing this expression")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnusedExpressions(Box<NoUnusedExpressionsConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoUnusedExpressionsConfig {
    allow_short_circuit: bool,
    allow_ternary: bool,
    allow_tagged_templates: bool,
    enforce_for_jsx: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows unused expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Unused expressions are usually a mistake. They can be a symptom of a bug or a misunderstanding of the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// Set<number>;
    /// 1 as number;
    /// window!;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const foo = new Set<number>();
    /// ```
    NoUnusedExpressions,
    eslint,
    restriction
);

impl Rule for NoUnusedExpressions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExpressionStatement(expression_stmt) = node.kind() else {
            return;
        };

        if self.is_disallowed(&expression_stmt.expression)
            && !is_parent_arrow_function_expression(node, ctx)
        {
            ctx.diagnostic(no_unused_expressions_diagnostic(expression_stmt.span));
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(NoUnusedExpressionsConfig {
            allow_short_circuit: value
                .get(0)
                .and_then(|x| x.get("allowShortCircuit"))
                .and_then(Value::as_bool)
                .unwrap_or_default(),
            allow_ternary: value
                .get(0)
                .and_then(|x| x.get("allowTernary"))
                .and_then(Value::as_bool)
                .unwrap_or_default(),
            allow_tagged_templates: value
                .get(0)
                .and_then(|x| x.get("allowTaggedTemplates"))
                .and_then(Value::as_bool)
                .unwrap_or_default(),
            enforce_for_jsx: value
                .get(0)
                .and_then(|x| x.get("enforceForJSX"))
                .and_then(Value::as_bool)
                .unwrap_or_default(),
        }))
    }
}

fn is_parent_arrow_function_expression<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(parent) = ctx.nodes().parent_node(node.id()) else { return false };

    let AstKind::FunctionBody(_) = parent.kind() else { return false };

    let Some(grand_parent) = ctx.nodes().parent_node(parent.id()) else { return false };

    let AstKind::ArrowFunctionExpression(arrow_function_expression) = grand_parent.kind() else {
        return false;
    };

    arrow_function_expression.expression
}

impl NoUnusedExpressions {
    fn is_disallowed(&self, expr: &Expression) -> bool {
        match expr {
            Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::SequenceExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::MetaProperty(_)
            | Expression::ObjectExpression(_)
            | Expression::PrivateFieldExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::BinaryExpression(_)
            | Expression::PrivateInExpression(_)
            | Expression::ThisExpression(_)
            | Expression::Identifier(_) => true,
            Expression::ChainExpression(chain_expression) => match &chain_expression.expression {
                ChainElement::CallExpression(_) => false,
                ChainElement::TSNonNullExpression(ts_non_null_expression) => {
                    self.is_disallowed(&ts_non_null_expression.expression)
                }
                ChainElement::ComputedMemberExpression(_)
                | ChainElement::StaticMemberExpression(_)
                | ChainElement::PrivateFieldExpression(_) => true,
            },
            Expression::AssignmentExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::NewExpression(_)
            | Expression::ImportExpression(_)
            | Expression::Super(_)
            | Expression::CallExpression(_)
            | Expression::UpdateExpression(_)
            | Expression::YieldExpression(_) => false,
            Expression::ConditionalExpression(conditional_expression) => {
                if self.0.allow_ternary {
                    return self.is_disallowed(&conditional_expression.alternate)
                        || self.is_disallowed(&conditional_expression.consequent);
                }
                true
            }
            Expression::LogicalExpression(logical_expression) => {
                if self.0.allow_short_circuit {
                    return self.is_disallowed(&logical_expression.right);
                }
                true
            }
            Expression::ParenthesizedExpression(parenthesized_expression) => {
                self.is_disallowed(&parenthesized_expression.expression)
            }
            Expression::TaggedTemplateExpression(_) => !self.0.allow_tagged_templates,
            Expression::UnaryExpression(unary_expression) => {
                !matches!(unary_expression.operator, UnaryOperator::Delete | UnaryOperator::Void)
            }
            Expression::JSXElement(_) | Expression::JSXFragment(_) => self.0.enforce_for_jsx,
            Expression::TSAsExpression(ts_as_expression) => {
                self.is_disallowed(&ts_as_expression.expression)
            }
            Expression::TSSatisfiesExpression(ts_satisfies_expression) => {
                self.is_disallowed(&ts_satisfies_expression.expression)
            }
            Expression::TSTypeAssertion(ts_type_assertion) => {
                self.is_disallowed(&ts_type_assertion.expression)
            }
            Expression::TSNonNullExpression(ts_non_null_expression) => {
                self.is_disallowed(&ts_non_null_expression.expression)
            }
            Expression::TSInstantiationExpression(ts_instantiation_expression) => {
                self.is_disallowed(&ts_instantiation_expression.expression)
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // https://github.com/eslint/eslint/blob/946ae00457265eb298eb169d6d48ca7ec71b9eef/tests/lib/rules/no-unused-expressions.js#L21
        ("function f(){}", None),
        ("a = b", None),
        ("new a", None),
        ("{}", None),
        ("f(); g()", None),
        ("i++", None),
        ("a()", None),
        ("a && a()", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        ("a() || (b = c)", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        ("a ? b() : c()", Some(serde_json::json!([{ "allowTernary": true }]))),
        (
            "a ? b() || (c = d) : e()",
            Some(serde_json::json!([{ "allowShortCircuit": true, "allowTernary": true }])),
        ),
        ("delete foo.bar", None),
        ("void new C", None),
        (r#""use strict";"#, None),
        (r#""directive one"; "directive two"; f();"#, None),
        (r#"function foo() {"use strict"; return true; }"#, None),
        (r#"var foo = () => {"use strict"; return true; }"#, None), // { "ecmaVersion": 6 },
        (r#"function foo() {"directive one"; "directive two"; f(); }"#, None),
        (r#"function foo() { var foo = "use strict"; return true; }"#, None),
        ("function* foo(){ yield 0; }", None), // { "ecmaVersion": 6 },
        ("async function foo() { await 5; }", None), // { "ecmaVersion": 8 },
        ("async function foo() { await foo.bar; }", None), // { "ecmaVersion": 8 },
        (
            "async function foo() { bar && await baz; }",
            Some(serde_json::json!([{ "allowShortCircuit": true }])),
        ), // { "ecmaVersion": 8 },
        (
            "async function foo() { foo ? await bar : await baz; }",
            Some(serde_json::json!([{ "allowTernary": true }])),
        ), // { "ecmaVersion": 8 },
        (
            "tag`tagged template literal`",
            Some(serde_json::json!([{ "allowTaggedTemplates": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "shouldNotBeAffectedByAllowTemplateTagsOption()",
            Some(serde_json::json!([{ "allowTaggedTemplates": true }])),
        ), // { "ecmaVersion": 6 },
        (r#"import("foo")"#, None),            // { "ecmaVersion": 11 },
        (r#"func?.("foo")"#, None),            // { "ecmaVersion": 11 },
        (r#"obj?.foo("bar")"#, None),          // { "ecmaVersion": 11 },
        ("<div />", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<></>", None),   // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("var partial = <div />", None), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("var partial = <div />", Some(serde_json::json!([{ "enforceForJSX": true }]))), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("var partial = <></>", Some(serde_json::json!([{ "enforceForJSX": true }]))), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } }
        // https://github.com/typescript-eslint/typescript-eslint/blob/32a7a7061abba5bbf1403230526514768d3e2760/packages/eslint-plugin/tests/rules/no-unused-expressions.test.ts#L29
        (
            "
			      test.age?.toLocaleString();
			    ",
            None,
        ),
        (
            "
			      let a = (a?.b).c;
			    ",
            None,
        ),
        (
            "
			      let b = a?.['b'];
			    ",
            None,
        ),
        (
            "
			      let c = one[2]?.[3][4];
			    ",
            None,
        ),
        (
            "
			      one[2]?.[3][4]?.();
			    ",
            None,
        ),
        (
            "
			      a?.['b']?.c();
			    ",
            None,
        ),
        (
            "
			      module Foo {
			        'use strict';
			      }
			    ",
            None,
        ),
        (
            "
			      namespace Foo {
			        'use strict';
			
			        export class Foo {}
			        export class Bar {}
			      }
			    ",
            None,
        ),
        (
            "
			      function foo() {
			        'use strict';
			
			        return null;
			      }
			    ",
            None,
        ),
        (
            "
			      import('./foo');
			    ",
            None,
        ),
        (
            "
			      import('./foo').then(() => {});
			    ",
            None,
        ),
        (
            "
			      class Foo<T> {}
			      new Foo<string>();
			    ",
            None,
        ),
        ("foo && foo?.();", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        ("foo && import('./foo');", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        (
            "foo ? import('./foo') : import('./bar');",
            Some(serde_json::json!([{ "allowTernary": true }])),
        ),
        ("const _func = (value: number) => value + 1;", None),
    ];

    let fail = vec![
        // https://github.com/eslint/eslint/blob/946ae00457265eb298eb169d6d48ca7ec71b9eef/tests/lib/rules/no-unused-expressions.js#L111
        ("0", None),
        ("a", None),
        ("f(), 0", None),
        ("{0}", None),
        ("[]", None),
        ("a && b();", None),
        ("a() || false", None),
        ("a || (b = c)", None),
        ("a ? b() || (c = d) : e", None),
        ("`untagged template literal`", None), // { "ecmaVersion": 6 },
        ("tag`tagged template literal`", None), // { "ecmaVersion": 6 },
        ("a && b()", Some(serde_json::json!([{ "allowTernary": true }]))),
        ("a ? b() : c()", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        ("a || b", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        ("a() && b", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        ("a ? b : 0", Some(serde_json::json!([{ "allowTernary": true }]))),
        ("a ? b : c()", Some(serde_json::json!([{ "allowTernary": true }]))),
        ("foo.bar;", None),
        ("!a", None),
        ("+a", None),
        (r#""directive one"; f(); "directive two";"#, None),
        (r#"function foo() {"directive one"; f(); "directive two"; }"#, None),
        (r#"if (0) { "not a directive"; f(); }"#, None),
        (r#"function foo() { var foo = true; "use strict"; }"#, None),
        (r#"var foo = () => { var foo = true; "use strict"; }"#, None), // { "ecmaVersion": 6 },
        (
            "`untagged template literal`",
            Some(serde_json::json!([{ "allowTaggedTemplates": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "`untagged template literal`",
            Some(serde_json::json!([{ "allowTaggedTemplates": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "tag`tagged template literal`",
            Some(serde_json::json!([{ "allowTaggedTemplates": false }])),
        ), // { "ecmaVersion": 6 },
        ("obj?.foo", None),                                             // { "ecmaVersion": 2020 },
        ("obj?.foo.bar", None),                                         // { "ecmaVersion": 2020 },
        ("obj?.foo().bar", None),                                       // { "ecmaVersion": 2020 },
        ("<div />", Some(serde_json::json!([{ "enforceForJSX": true }]))), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("<></>", Some(serde_json::json!([{ "enforceForJSX": true }]))), // { "parserOptions": { "ecmaFeatures": { "jsx": true } } },
        ("class C { static { 'use strict'; } }", None),                  // { "ecmaVersion": 2022 },
        (
            "class C { static { 
			'foo'
			'bar'
			 } }",
            None,
        ), // { "ecmaVersion": 2022 }
        // https://github.com/typescript-eslint/typescript-eslint/blob/32a7a7061abba5bbf1403230526514768d3e2760/packages/eslint-plugin/tests/rules/no-unused-expressions.test.ts#L91
        (
            "
			if (0) 0;
			      ",
            None,
        ),
        (
            "
			f(0), {};
			      ",
            None,
        ),
        (
            "
			a, b();
			      ",
            None,
        ),
        (
            "
			a() &&
			  function namedFunctionInExpressionContext() {
			    f();
			  };
			      ",
            None,
        ),
        (
            "
			a?.b;
			      ",
            None,
        ),
        (
            "
			(a?.b).c;
			      ",
            None,
        ),
        (
            "
			a?.['b'];
			      ",
            None,
        ),
        (
            "
			(a?.['b']).c;
			      ",
            None,
        ),
        (
            "
			a?.b()?.c;
			      ",
            None,
        ),
        (
            "
			(a?.b()).c;
			      ",
            None,
        ),
        (
            "
			one[2]?.[3][4];
			      ",
            None,
        ),
        (
            "
			one.two?.three.four;
			      ",
            None,
        ),
        (
            "
			module Foo {
			  const foo = true;
			  'use strict';
			}
			      ",
            None,
        ),
        (
            "
			namespace Foo {
			  export class Foo {}
			  export class Bar {}
			
			  'use strict';
			}
			      ",
            None,
        ),
        (
            "
			function foo() {
			  const foo = true;
			
			  'use strict';
			}
			      ",
            None,
        ),
        ("foo && foo?.bar;", Some(serde_json::json!([{ "allowShortCircuit": true }]))),
        ("foo ? foo?.bar : bar.baz;", Some(serde_json::json!([{ "allowTernary": true }]))),
        (
            "
			class Foo<T> {}
			Foo<string>;
			      ",
            None,
        ),
        ("Map<string, string>;", None),
        (
            "
			declare const foo: number | undefined;
			foo;
			      ",
            None,
        ),
        (
            "
			declare const foo: number | undefined;
			foo as any;
			      ",
            None,
        ),
        (
            "
			declare const foo: number | undefined;
			<any>foo;
			      ",
            None,
        ),
        (
            "
			declare const foo: number | undefined;
			foo!;
			      ",
            None,
        ),
        ("const _func = (value: number) => { value + 1; }", None),
    ];

    Tester::new(NoUnusedExpressions::NAME, NoUnusedExpressions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
