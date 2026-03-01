use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{BinaryOperator, Expression, IfStatement, LogicalOperator, MemberExpression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_same_member_expression,
};

fn prefer_switch_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `switch` instead of multiple `else-if`.")
        .with_help("Prefer switch when branching on repeated equality comparisons.")
        .with_label(span)
}

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EmptyDefaultCase {
    /// Require a comment in the default case to explain why it's empty.
    #[default]
    NoDefaultComment,
    /// Allow an empty default case.
    DoNothingComment,
    /// Disallow a default case.
    NoDefaultCase,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferSwitchConfig {
    /// The minimum number of cases required to trigger the rule.
    minimum_cases: usize,
    /// How to handle an empty default case.
    empty_default_case: EmptyDefaultCase,
}

impl Default for PreferSwitchConfig {
    fn default() -> Self {
        Self { minimum_cases: 3, empty_default_case: EmptyDefaultCase::NoDefaultComment }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferSwitch(Box<PreferSwitchConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `switch` over multiple `else-if`.
    ///
    /// ### Why is this bad?
    ///
    /// A `switch` statement is easier to read than repeated equality checks in
    /// long `if`/`else-if` chains.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (foo === 1) {}
    /// else if (foo === 2) {}
    /// else if (foo === 3) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// switch (foo) {
    ///   case 1:
    ///     break;
    ///   case 2:
    ///     break;
    ///   case 3:
    ///     break;
    /// }
    /// ```
    PreferSwitch,
    unicorn,
    style,
    pending,
    config = PreferSwitchConfig
);

impl Rule for PreferSwitch {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_statement) = node.kind() else {
            return;
        };

        if let AstKind::IfStatement(parent_if_statement) = ctx.nodes().parent_kind(node.id())
            && let Some(Statement::IfStatement(else_if_statement)) =
                parent_if_statement.alternate.as_ref()
            && else_if_statement.span == if_statement.span
        {
            let (parent_chain_count, parent_discriminant) =
                get_statements(parent_if_statement, ctx);
            if parent_discriminant.is_some() && parent_chain_count >= self.0.minimum_cases {
                return;
            }
        }

        let (if_statements_count, discriminant) = get_statements(if_statement, ctx);
        if discriminant.is_none() || if_statements_count < self.0.minimum_cases {
            return;
        }

        ctx.diagnostic(prefer_switch_diagnostic(if_statement.span));
    }
}

#[derive(Debug, Clone, Copy)]
struct Comparison<'a> {
    left: &'a Expression<'a>,
    right: &'a Expression<'a>,
}

fn get_statements<'a>(
    start_statement: &'a IfStatement<'a>,
    ctx: &LintContext<'a>,
) -> (usize, Option<&'a Expression<'a>>) {
    let mut discriminant_candidates: Option<Vec<&Expression<'a>>> = None;
    let mut if_statements_count = 0;
    let mut current_statement = Some(start_statement);

    while let Some(statement) = current_statement {
        let comparisons = get_equality_comparisons(&statement.test);
        if comparisons.is_empty() {
            break;
        }

        if discriminant_candidates.is_none() {
            discriminant_candidates = Some(vec![comparisons[0].left, comparisons[0].right]);
        }

        let candidates = get_common_references(
            &comparisons,
            discriminant_candidates.as_ref().map_or_else(Vec::new, Clone::clone),
            ctx,
        );
        if candidates.is_empty() {
            break;
        }
        discriminant_candidates = Some(candidates);

        if_statements_count += 1;
        current_statement = match statement.alternate.as_ref() {
            Some(Statement::IfStatement(next_statement)) => Some(next_statement),
            _ => None,
        };
    }

    let discriminant = discriminant_candidates.and_then(|candidates| candidates.first().copied());

    (if_statements_count, discriminant)
}

fn get_equality_comparisons<'a>(node: &'a Expression<'a>) -> Vec<Comparison<'a>> {
    fn collect<'a>(node: &'a Expression<'a>, comparisons: &mut Vec<Comparison<'a>>) -> bool {
        let node = node.get_inner_expression();

        if let Expression::LogicalExpression(logical_expression) = node
            && logical_expression.operator == LogicalOperator::Or
        {
            return collect(&logical_expression.left, comparisons)
                && collect(&logical_expression.right, comparisons);
        }

        let Expression::BinaryExpression(binary_expression) = node else {
            return false;
        };
        if binary_expression.operator != BinaryOperator::StrictEquality {
            return false;
        }

        comparisons.push(Comparison {
            left: binary_expression.left.get_inner_expression(),
            right: binary_expression.right.get_inner_expression(),
        });
        true
    }

    let mut comparisons = vec![];
    if collect(node, &mut comparisons) { comparisons } else { vec![] }
}

fn get_common_references<'a>(
    expressions: &[Comparison<'a>],
    mut candidates: Vec<&'a Expression<'a>>,
    ctx: &LintContext<'a>,
) -> Vec<&'a Expression<'a>> {
    for comparison in expressions {
        candidates.retain(|candidate| {
            is_same_reference(candidate, comparison.left, ctx)
                || is_same_reference(candidate, comparison.right, ctx)
        });
        if candidates.is_empty() {
            break;
        }
    }
    candidates
}

fn is_same_reference<'a>(
    left: &'a Expression<'a>,
    right: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let left = left.get_inner_expression();
    let right = right.get_inner_expression();

    if std::ptr::eq(left, right) {
        return true;
    }

    match (left, right) {
        (Expression::Super(_), Expression::Super(_))
        | (Expression::ThisExpression(_), Expression::ThisExpression(_))
        | (Expression::NullLiteral(_), Expression::NullLiteral(_)) => true,
        (Expression::Identifier(left), Expression::Identifier(right)) => left.name == right.name,
        (Expression::StringLiteral(left), Expression::StringLiteral(right)) => {
            left.value == right.value
        }
        (Expression::NumericLiteral(left), Expression::NumericLiteral(right)) => {
            left.value.to_bits() == right.value.to_bits()
        }
        (Expression::BooleanLiteral(left), Expression::BooleanLiteral(right)) => {
            left.value == right.value
        }
        (Expression::BigIntLiteral(left), Expression::BigIntLiteral(right)) => {
            left.raw == right.raw
        }
        (Expression::RegExpLiteral(left), Expression::RegExpLiteral(right)) => {
            left.regex.pattern.text == right.regex.pattern.text
                && left.regex.flags == right.regex.flags
        }
        (Expression::ChainExpression(_), Expression::ChainExpression(_)) => {
            let left_member = match left {
                Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
                _ => None,
            };
            let right_member = match right {
                Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
                _ => None,
            };

            match (left_member, right_member) {
                (Some(left), Some(right)) => is_same_member_expression(left, right, ctx),
                _ => false,
            }
        }
        _ => {
            let left_member = match left {
                Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
                _ => left.as_member_expression(),
            };
            let right_member = match right {
                Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
                _ => right.as_member_expression(),
            };

            match (left_member, right_member) {
                (Some(left), Some(right)) => {
                    if let (Some(left_name), Some(right_name)) =
                        (member_static_property_name(left), member_static_property_name(right))
                    {
                        return left_name == right_name
                            && is_same_reference(
                                left.object().get_inner_expression(),
                                right.object().get_inner_expression(),
                                ctx,
                            );
                    }

                    is_same_member_expression(left, right, ctx)
                }
                _ => false,
            }
        }
    }
}

fn member_static_property_name(member: &MemberExpression<'_>) -> Option<String> {
    if let Some(name) = member.static_property_name() {
        return Some(name.to_string());
    }

    let MemberExpression::ComputedMemberExpression(computed) = member else {
        return None;
    };

    static_string_value(computed.expression.get_inner_expression())
}

fn static_string_value(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(literal) => {
            literal.single_quasi().map(|quasi| quasi.to_string())
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = static_string_value(binary.left.get_inner_expression())?;
            let right = static_string_value(binary.right.get_inner_expression())?;
            Some(format!("{left}{right}"))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "if (foo === 1) {}
            else if (foo === 2) {}",
            None,
        ),
        (
            "if (foo === 1) {
                if (foo === 2) {}
            }
            else if (foo === 2) {}",
            None,
        ),
        (
            "if (foo === 1 || foo === 2) {}
            else if (foo === 3 || foo === 4) {}",
            None,
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (bar === 1) {}
            else if (foo === 3) {}
            else if (foo === 4) {}
            else if (bar === 1) {}
            else if (foo === 5) {}
            else if (foo === 6) {}",
            None,
        ),
        (
            "if (foo() === 1) {}
            else if (foo() === 2) {}
            else if (foo() === 3) {}",
            None,
        ),
        ("foo === 1 ? 1 : foo === 2 ? 2 : foo === 3 ? 3 : 0", None),
        (
            "if (foo === 1) {}
            else if (foo !== 2) {}
            else if (foo === 3) {}",
            None,
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2 && foo === 4) {}
            else if (foo === 3) {}",
            None,
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2 || foo !== 4) {}
            else if (foo === 3) {}",
            None,
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}",
            Some(serde_json::json!([{"minimumCases": 4}])),
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}
            else {}",
            Some(serde_json::json!([{"minimumCases": 4}])),
        ),
    ];

    let fail = vec![
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}",
            None,
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}
            else {
                // default
            }",
            None,
        ),
        (
            "if (foo === 1) (notBlock())
            else if (foo === 2) (notBlock())
            else if (foo === 3) (notBlock())
            else (notBlock())",
            None,
        ),
        (
            "if (bar = 1) {}
            else if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}",
            None,
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}
            else if (bar === 3) {}",
            None,
        ),
        (
            "if (foo === 0) {
                if (foo === 1) {}
                else if (foo === 2) {}
                else if (foo === 3) {}
            }",
            None,
        ),
        (
            "if (1 === foo) {}
            else if (foo === 2) {}
            else if (3 === foo) {}",
            None,
        ),
        (
            "if (true === foo) {}
            else if (bar.bar === true) {}
            else if (true === baz()) {}",
            None,
        ),
        (
            "if (foo === ((0, 1))) {}
            else if (foo === (bar + 2)) {}
            else if (foo === (baz = 2)) {}",
            None,
        ),
        (
            r#"// Should use "foo" as discriminant
            if (foo === bar) {}
            else if (foo === bar) {}
            else if (foo === bar) {}
            // Should use "bar" as discriminant
            if (bar === foo) {}
            else if (bar === foo) {}
            else if (foo === bar) {}"#,
            None,
        ),
        ("if (foo === 1) {}", None),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}
            else if (foo === 4) {}
            else if (foo === 5) {}
            else if (foo === 6) {}
            else if (foo === 7) {}
            else if (foo === 8) {}
            else if (foo === 9) {}
            else if (foo === 10) {}
            else {}",
            None,
        ),
        (
            "if (foo === 1) {}
            else if ((foo === 2 || foo === 3) || (foo === 4)) {}
            else if (foo === 5) {}",
            None,
        ),
        (
            "function foo() {
                for (const a of b) {
                    if (foo === 1) {
                        return 1;
                    } else if (foo === 2) {
                        throw new Error();
                    } else if (foo === 3) {
                        alert(foo);
                    } else {
                        console.log('wow');
                    }
                }
            }",
            None,
        ),
        (
            "function foo() {
                return bar.map(foo => {
                    if (foo === 1) return foo;
                    else if (foo === 2) throw new Error();
                    else if (foo === 3) foo++
                    else console.log('wow');
                })
            }",
            None,
        ),
        (
            "if (one) {}
            else if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}
            else if (two) {}
            else if (bar === 1) {}
            else if (bar === 2) {}
            else if (bar === 3) {}
            else if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {}",
            None,
        ),
        (
            r#"if (foo.baz === 1) {}
            else if (foo['baz'] === 2) {}
            else if (foo["ba" + 'z'] === 3) {}"#,
            None,
        ),
        (
            "while (bar) {
                if (foo === 1) {
                    for (const foo of bar) {
                        break;
                    }
                } else if (foo === 2) {
                } else if (foo === 3) {
                }
            }",
            None,
        ),
        (
            "while (bar) {
                if (foo === 1) {
                    break;
                } else if (foo === 2) {
                } else if (foo === 3) {
                }
            }",
            None,
        ),
        (
            "while (bar) {
                if (foo === 1) {
                } else if (foo === 2) {
                    break;
                } else if (foo === 3) {
                }
            }",
            None,
        ),
        (
            "while (bar) {
                if (foo === 1) {
                } else if (foo === 2) {
                } else if (foo === 3) {
                    if (a) {
                        if (b) {
                            if (c) {
                                break;
                            }
                        }
                    }
                }
            }",
            None,
        ),
        (
            "switch (bar) {
                case 'bar':
                    if (foo === 1) {
                    } else if (foo === 2) {
                    } else if (foo === 3) {
                        break;
                    }
            }",
            None,
        ),
        (
            r#"function unicorn() {
                if (foo === 1) return 1;
                else if (foo === 2) throw new Error("");
                else if (foo === 3) process.exit(1);
                else if (foo === 4) {}
                else if (foo === 5) ;
                else if (foo === 6) {
                    return 6;
                    // Already unreachable
                    call();
                }
                else if (foo === 7) {
                    return 7;
                    // EmptyStatement after return
                    ;;;;;;
                }
                else if (foo === 8) {
                    return 8;
                    // FunctionDeclaration after return
                    function afterReturn() {}
                }
                else if (foo === 9) {
                    return 9;
                    // FunctionExpression after return
                    const afterReturn = function afterReturn() {return 9}
                }
                else if (foo === 10) {
                    {{{
                        return 10;
                    };};};
                }
                else if (foo === 11) {
                    return 11;
                    {{{
                        ;;;
                        function afterReturn() {}
                        ;;;
                        function afterReturn2() {}
                        ;;;
                    }}}
                }
                else if (foo === 12) {
                    return twelve;
                    var twelve = 12;
                }
                else return 'default';
            }"#,
            None,
        ),
        (
            "function unicorn() {
                if (foo === 1) {
                    if (true) {
                        throw error;
                    } else {
                        return false;
                    }
                }
                else if (foo === 2) {
                    if (true) {
                        throw error;
                    }
                // no else, need break
                }
                else if (foo === 3) {
                    if (a) {
                        return a;
                    } else if (b) {
                        return b;
                    } else if (c) {
                        return c;
                    } else if (d) {
                        if (dd) {
                            return dd;
                        } else {
                            return dd;
                        }
                    } else {
                        return f;
                    }
                }
                else if (foo === 4) {
                    if (a) {
                        return a;
                    } else if (b) {
                        return b;
                    } else if (c) {
                        return c;
                    } else if (d) {
                        return e;
                    } // here
                // missing else deep inside, need break
                }
                else if (foo === 5) {
                    if (a) {
                        return a;
                    } else if (b) {
                        return b;
                    } else if (c) {
                        return c;
                    } else if (d) {
                        if (dd) {
                            return dd;
                        } else if (de) {
                            return de;
                        } // here
                    } else {
                        return f;
                    }
                // missing else deep inside, need break
                }
                else if (foo === 6) {
                    if (a) {
                        return a;
                    } else if (b) {
                        return b;
                    } else if (c) {
                        // here
                    } else if (d) {
                        return e;
                    } else {
                        return f;
                    }
                // missing one return, need break
                }
                else if (foo === 7) {
                    if (a) return a;
                    else if (b) {
                        return b;;;;;
                    } else if (c) {
                        return c;
                        function x() {}
                    } else if (d) {
                        return e;
                    } else {
                        return f;
                    }
                }
            }",
            None,
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}",
            Some(serde_json::json!([{"minimumCases": 2}])),
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else {}",
            Some(serde_json::json!([{"minimumCases": 2}])),
        ),
        (
            "function foo() {
                if (foo === 1) {}
                else if (foo === 2) {}
                else if (foo === 3) {}
            }",
            Some(serde_json::json!([{"emptyDefaultCase": "no-default-comment"}])),
        ),
        (
            "function foo() {
                if (foo === 1) {}
                else if (foo === 2) {}
                else if (foo === 3) {}
            }",
            Some(serde_json::json!([{"emptyDefaultCase": "do-nothing-comment"}])),
        ),
        (
            "function foo() {
                if (foo === 1) {}
                else if (foo === 2) {}
                else if (foo === 3) {}
            }",
            Some(serde_json::json!([{"emptyDefaultCase": "no-default-case"}])),
        ),
        (
            "if (foo === 1) {}
            else if (foo === 2) {}
            else if (foo === 3) {break;}",
            None,
        ),
    ];

    Tester::new(PreferSwitch::NAME, PreferSwitch::PLUGIN, pass, fail).test_and_snapshot();
}
