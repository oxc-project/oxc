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
    restriction
);

impl Rule for NoUnusedExpressions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExpressionStatement(expression_stmt) = node.kind() else {
            return;
        };

        if self.is_disallowed(&expression_stmt.expression) {
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
    ];

    let fail = vec![
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
    ];

    Tester::new(NoUnusedExpressions::NAME, NoUnusedExpressions::CATEGORY, pass, fail)
        .test_and_snapshot();
}
