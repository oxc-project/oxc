use oxc_ast::{
    ast::{BinaryOperator, Expression, UnaryOperator, VariableDeclarationKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct ConsistentExistenceIndexCheck;

fn consistent_existence_index_check_diagnostic(
    replacement: &GetReplacementOutput,
    span: Span,
) -> OxcDiagnostic {
    let existence_or_non_existence =
        if replacement.replacement_value == "-1" { "non-existence" } else { "existence" };

    let label = format!(
        "Prefer `{replacement_operator} {replacement_value}` over `{original_operator} {original_value}` to check {existenceOrNonExistence}.",
        replacement_operator = replacement.replacement_operator,
        replacement_value = replacement.replacement_value,
        original_operator = replacement.original_operator,
        original_value = replacement.original_value,
        existenceOrNonExistence = existence_or_non_existence,
    );

    OxcDiagnostic::warn(label).with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent style for element existence checks with `indexOf()`, `lastIndexOf()`, `findIndex()`, and `findLastIndex()`
    ///
    /// ### Why is this bad?
    ///
    /// This rule is only meant to enforce a specific style and make comparisons more clear.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```javascript
    /// const index = foo.indexOf('bar');
    /// if (index < 0) {}
    /// ```
    ///
    /// ``` javascript
    /// const index = foo.indexOf('bar');
    /// if (index >= 0) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```javascript
    /// const index = foo.indexOf('bar');
    /// if (index === -1) {}
    /// ```
    ///
    /// ``` javascript
    /// const index = foo.indexOf('bar');
    /// if (index !== -1) {}
    /// ```
    ConsistentExistenceIndexCheck,
    unicorn,
    style,
    fix,
);

const METHOD_NAMES: [&str; 4] = ["indexOf", "lastIndexOf", "findIndex", "findLastIndex"];

impl Rule for ConsistentExistenceIndexCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expression) = node.kind() else {
            return;
        };

        let left = binary_expression.left.get_inner_expression();
        let right = binary_expression.right.get_inner_expression();
        let operator = binary_expression.operator;

        let Expression::Identifier(identifier) = left else {
            return;
        };

        let Some(symbol_id) = ctx.symbols().get_reference(identifier.reference_id()).symbol_id()
        else {
            return;
        };

        let declaration_node_id = ctx.symbols().get_declaration(symbol_id);
        let node = ctx.nodes().get_node(declaration_node_id);

        if let AstKind::VariableDeclarator(variables_declarator) = node.kind() {
            if variables_declarator.kind != VariableDeclarationKind::Const {
                return;
            }

            let Some(Expression::CallExpression(call)) = &variables_declarator.init else {
                return;
            };

            if !call.callee.is_member_expression() {
                return;
            }

            let Some(callee_name) = call.callee_name() else {
                return;
            };

            if !METHOD_NAMES.contains(&callee_name) {
                return;
            }

            let replacement = get_replacement(right, operator);

            let Some(replacement) = &replacement else {
                return;
            };

            ctx.diagnostic_with_fix(
                consistent_existence_index_check_diagnostic(replacement, binary_expression.span),
                |fixer| {
                    let operator_start = binary_expression.left.span().end;
                    let operator_end = binary_expression.right.span().start;
                    let operator_span = Span::new(operator_start, operator_end);
                    let operator_source = ctx.source_range(operator_span);

                    let operator_matches =
                        operator_source.match_indices(replacement.original_operator);
                    let mut operator_replacement_text = operator_source.to_string();

                    for (index, text) in operator_matches {
                        let comments = ctx.semantic().comments_range(operator_start..operator_end);

                        let start = operator_start + u32::try_from(index).unwrap_or(0);
                        let length = u32::try_from(text.len()).unwrap_or(0);
                        let span = Span::sized(start, length);

                        let mut is_in_comment = false;

                        for comment in comments {
                            if comment.span.contains_inclusive(span) {
                                is_in_comment = true;
                                break;
                            }
                        }

                        if !is_in_comment {
                            let head = &operator_source[..index];
                            let tail = &operator_source[index + text.len()..];

                            operator_replacement_text =
                                format!("{}{}{}", head, replacement.replacement_operator, tail);
                        }
                    }

                    let fixer = fixer.for_multifix();
                    let mut rule_fixes = fixer.new_fix_with_capacity(2);

                    rule_fixes.push(fixer.replace(operator_span, operator_replacement_text));
                    rule_fixes.push(fixer.replace(right.span(), replacement.replacement_value));

                    rule_fixes
                },
            );
        };
    }
}

#[derive(Debug, Clone)]
struct GetReplacementOutput {
    pub replacement_operator: &'static str,
    pub replacement_value: &'static str,
    pub original_operator: &'static str,
    pub original_value: &'static str,
}

fn get_replacement(right: &Expression, operator: BinaryOperator) -> Option<GetReplacementOutput> {
    match operator {
        BinaryOperator::LessThan => {
            if right.is_number_0() {
                return Some(GetReplacementOutput {
                    replacement_operator: "===",
                    replacement_value: "-1",
                    original_operator: "<",
                    original_value: "0",
                });
            }

            None
        }
        BinaryOperator::GreaterThan => {
            if is_negative_one(right.get_inner_expression()) {
                return Some(GetReplacementOutput {
                    replacement_operator: "!==",
                    replacement_value: "-1",
                    original_operator: ">",
                    original_value: "-1",
                });
            }

            None
        }
        BinaryOperator::GreaterEqualThan => {
            if right.is_number_0() {
                return Some(GetReplacementOutput {
                    replacement_operator: "!==",
                    replacement_value: "-1",
                    original_operator: ">=",
                    original_value: "0",
                });
            }

            None
        }
        _ => None,
    }
}

fn is_negative_one(expression: &Expression) -> bool {
    if let Expression::UnaryExpression(unary_expression) = expression {
        if let UnaryOperator::UnaryNegation = unary_expression.operator {
            if let Expression::NumericLiteral(value) =
                &unary_expression.argument.get_inner_expression()
            {
                return value.raw.as_ref().unwrap() == "1";
            }
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<&str> = vec![
        // Skip checking if indexOf() method is not a method call from a object
        r"
              const index = indexOf('bar');
              if (index > -1) {}
        ",
        r"
              const index = foo.indexOf('bar');
              if (index === -1) {}
        ",
        r"
              const index = foo.indexOf('bar');
              if (-1 === index) {}
        ",
        r"
              const index = foo.indexOf('bar');
              if (index !== -1) {}
        ",
        r"
              const index = foo.indexOf('bar');
              if (-1 !== index) {}
        ",
        // // Variable index is not from indexOf
        r"
              const index = 0;
              if (index < 0) {}
        ",
        // If index is not declared via VariableDeclarator, it will not be check here.
        r"
              function foo (index) {
                  if (index < 0) {}
              }
        ",
        // Since the variable is references from function parameter, it will not be checked here
        r"
          	const index = foo.indexOf('bar');
          	function foo (index) {
          		if (index < 0) {}
          	}
        ",
        // To prevent false positives, it will not check if the index is not declared via const
        r"
          	let index = foo.indexOf('bar');

          	index < 0
        ",
        // To prevent false positives, it will not check if the index is not declared via const
        r"
          	var index = foo.indexOf('bar');
          	index < 0
        ",
        // To prevent false positives, it will not check if the index is not declared via const
        r"
          	let index;

          	// do stuff

          	index = arr.findLastIndex(element => element > 10);
          	index < 0;
        ",
        r"const indexOf = 'indexOf'; const index = foo[indexOf](foo); index < 0;",
        r"const index = foo.indexOf?.(foo); index < 0;",
        r"const index = foo?.indexOf(foo); index < 0;",
    ];

    let fail = vec![
        r"const index = foo.indexOf('bar'); if (index < 0) {}",
        r"const index = foo.lastIndexOf('bar'); if (index < 0) {}",
        r"const index = foo.findIndex('bar'); if (index < 0) {}",
        r"const index = foo.findLastIndex('bar'); if (index < 0) {}",
        r"const index = foo.indexOf('bar'); if (index >= 0) {}",
        r"const index = foo.lastIndexOf('bar'); if (index >= 0) {}",
        r"const index = foo.findIndex('bar'); if (index >= 0) {}",
        r"const index = foo.findLastIndex('bar'); if (index >= 0) {}",
        r"const index = foo.indexOf('bar'); if (index > -1) {}",
        r"const index = foo.lastIndexOf('bar'); if (index > -1) {}",
        r"const index = foo.findIndex('bar'); if (index > -1) {}",
        r"const index = foo.findLastIndex('bar'); if (index > -1) {}",
        r"
        	const index = foo.indexOf(bar);

        	function foo () {
        		if (index < 0) {}
        	}
        ",
        r"
        	const index1 = foo.indexOf('1'),
        		index2 = foo.indexOf('2');
        	index1 < 0;
        	index2 >= 0;
        ",
        r"
              	const index = foo.indexOf('1');
              	((
              		/* comment 1 */
              		((
              			/* comment 2 */
              			index
              			/* comment 3 */
              		))
              		/* comment 4 */
              		<
              		/* comment 5 */
              		((
              			/* comment 6 */
              			0
              			/* comment 7 */
              		))
              		/* comment 8 */
              	));
              ",
        r"
        	const index = foo.indexOf('1');
        	((
        		/* comment 1 */
        		((
        			/* comment 2 */
        			index
        			/* comment 3 */
        		))
        		/* comment 4 */
        		>
        		((
        			/* comment 5 */
        			- /* comment 6 */ (( /* comment 7 */ 1 /* comment 8 */ ))
        			/* comment 9 */
        		))
        	));
        ",
        r"const index = _.indexOf([1, 2, 1, 2], 2); index < 0;",
    ];

    let fix = vec![
        (
            r"const index = foo.indexOf('bar'); if (index < 0) {}",
            r"const index = foo.indexOf('bar'); if (index === -1) {}",
            None,
        ),
        (
            r"const index = foo.lastIndexOf('bar'); if (index < 0) {}",
            r"const index = foo.lastIndexOf('bar'); if (index === -1) {}",
            None,
        ),
        (
            r"const index = foo.findIndex('bar'); if (index < 0) {}",
            r"const index = foo.findIndex('bar'); if (index === -1) {}",
            None,
        ),
        (
            r"const index = foo.findLastIndex('bar'); if (index < 0) {}",
            r"const index = foo.findLastIndex('bar'); if (index === -1) {}",
            None,
        ),
        (
            r"const index = foo.indexOf('bar'); if (index >= 0) {}",
            r"const index = foo.indexOf('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.lastIndexOf('bar'); if (index >= 0) {}",
            r"const index = foo.lastIndexOf('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.findIndex('bar'); if (index >= 0) {}",
            r"const index = foo.findIndex('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.findLastIndex('bar'); if (index >= 0) {}",
            r"const index = foo.findLastIndex('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.indexOf('bar'); if (index > -1) {}",
            r"const index = foo.indexOf('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.lastIndexOf('bar'); if (index > -1) {}",
            r"const index = foo.lastIndexOf('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.findIndex('bar'); if (index > -1) {}",
            r"const index = foo.findIndex('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.findLastIndex('bar'); if (index > -1) {}",
            r"const index = foo.findLastIndex('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"
                    const index = foo.indexOf(bar);
        
                    function foo () {
                        if (index < 0) {}
                    }
                    ",
            r"
                    const index = foo.indexOf(bar);
        
                    function foo () {
                        if (index === -1) {}
                    }
                    ",
            None,
        ),
        (
            r"
                    const index1 = foo.indexOf('1'),
                        index2 = foo.indexOf('2');
                    index1 < 0;
                    index2 >= 0;
                    ",
            r"
                    const index1 = foo.indexOf('1'),
                        index2 = foo.indexOf('2');
                    index1 === -1;
                    index2 !== -1;
                    ",
            None,
        ),
        (
            r"
                    const index = foo.indexOf('1');
                    ((
                        /* comment 1 */
                        ((
                            /* comment 2 */
                            index
                            /* comment 3 */
                        ))
                        /* comment 4 */
                        <
                        /* comment 5 */
                        ((
                            /* comment 6 */
                            0
                            /* comment 7 */
                        ))
                        /* comment 8 */
                    ));
                    ",
            r"
                    const index = foo.indexOf('1');
                    ((
                        /* comment 1 */
                        ((
                            /* comment 2 */
                            index
                            /* comment 3 */
                        ))
                        /* comment 4 */
                        ===
                        /* comment 5 */
                        ((
                            /* comment 6 */
                            -1
                            /* comment 7 */
                        ))
                        /* comment 8 */
                    ));
                    ",
            None,
        ),
        (
            r"
                const index = foo.indexOf('1');
                ((
                    /* comment 1 */
                    ((
                        /* comment 2 */
                        index
                        /* comment 3 */
                    ))
                    /* comment 4 */
                    >
                    ((
                        /* comment 5 */
                        - /* comment 6 */ (( /* comment 7 */ 1 /* comment 8 */ ))
                        /* comment 9 */
                    ))
                ));
            ",
            r"
                const index = foo.indexOf('1');
                ((
                    /* comment 1 */
                    ((
                        /* comment 2 */
                        index
                        /* comment 3 */
                    ))
                    /* comment 4 */
                    !==
                    ((
                        /* comment 5 */
                        -1
                        /* comment 9 */
                    ))
                ));
            ",
            None,
        ),
        (
            r"const index = _.indexOf([1, 2, 1, 2], 2); index < 0;",
            r"const index = _.indexOf([1, 2, 1, 2], 2); index === -1;",
            None,
        ),
        (
            r"const i = foo.indexOf('bar'); if (i /* < */ < 0) {}",
            r"const i = foo.indexOf('bar'); if (i /* < */ === -1) {}",
            None,
        ),
        // make sure to not replace the wrong operator
        (
            r"
                  const index = foo.indexOf('bar');
                  if (
                    index
                    /* >= */
                    >=
                    0
                  ) {}
            ",
            r"
                  const index = foo.indexOf('bar');
                  if (
                    index
                    /* >= */
                    !==
                    -1
                  ) {}
            ",
            None,
        ),
        // make sure to not replace the wrong operator
        (
            r"
                  const index = foo.indexOf('bar');
                  if (
                    index
                    /* >= */
                    >= // >=
                    0
                  ) {}
            ",
            r"
                  const index = foo.indexOf('bar');
                  if (
                    index
                    /* >= */
                    !== // >=
                    -1
                  ) {}
            ",
            None,
        ),
        (
            r"
               const index = foo.indexOf('1');
               ((
                   /* comment 1 */
                   ((
                       /* comment 2 */
                       index
                       /* comment 3 */
                   ))
                   /* comment 4 < */
                   <
                   /* comment 5 */
                   ((
                       /* comment 6 */
                       0
                       /* comment 7 */
                   ))
                   /* comment 8 */
               ));
            ",
            r"
               const index = foo.indexOf('1');
               ((
                   /* comment 1 */
                   ((
                       /* comment 2 */
                       index
                       /* comment 3 */
                   ))
                   /* comment 4 < */
                   ===
                   /* comment 5 */
                   ((
                       /* comment 6 */
                       -1
                       /* comment 7 */
                   ))
                   /* comment 8 */
               ));
            ",
            None,
        ),
        (
            r"const index = foo.indexOf('bar'); if (index >= 0) {}",
            r"const index = foo.indexOf('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.lastIndexOf('bar'); if (index >= 0) {}",
            r"const index = foo.lastIndexOf('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.findIndex('bar'); if (index >= 0) {}",
            r"const index = foo.findIndex('bar'); if (index !== -1) {}",
            None,
        ),
        (
            r"const index = foo.findLastIndex('bar'); if (index >= 0) {}",
            r"const index = foo.findLastIndex('bar'); if (index !== -1) {}",
            None,
        ),
    ];

    Tester::new(
        ConsistentExistenceIndexCheck::NAME,
        ConsistentExistenceIndexCheck::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
