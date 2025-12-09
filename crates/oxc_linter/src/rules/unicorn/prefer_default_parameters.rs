use oxc_ast::{
    AstKind,
    ast::{
        AssignmentOperator, AssignmentTarget, BindingPatternKind, Expression, FormalParameter,
        LogicalOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_default_parameters_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer default parameters over reassignment for '{name}'."))
        .with_help("Replace the reassignment with a default parameter.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDefaultParameters;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Instead of reassigning a function parameter, default parameters should be used. The `foo = foo || 123` statement evaluates to `123` when `foo` is falsy, possibly leading to confusing behavior, whereas default parameters only apply when passed an `undefined` value.
    /// This rule only reports reassignments to literal values.
    ///
    /// You should disable this rule if you want your functions to deal with `null` and other falsy values the same way as `undefined`.
    /// Default parameters are exclusively applied [when `undefined` is received.](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Default_parameters#passing_undefined_vs._other_falsy_values).
    /// However, we recommend [moving away from `null`](https://github.com/sindresorhus/meta/discussions/7).
    ///
    /// ### Why is this bad?
    ///
    /// Using default parameters makes it clear that a parameter has a default value, improving code readability and maintainability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function abc(foo) {
    /// 	foo = foo || 'bar';
    /// }
    ///
    /// function abc(foo) {
    /// 	const bar = foo || 'bar';
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function abc(foo = 'bar') {}
    ///
    /// function abc(bar = 'bar') {}
    ///
    /// function abc(foo) {
    /// 	foo = foo || bar();
    /// }
    /// ```
    PreferDefaultParameters,
    unicorn,
    style,
    pending,
);

fn is_literal(expr: &Expression) -> bool {
    matches!(
        expr.without_parentheses(),
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
    )
}

/// Find enclosing function and function body node IDs
fn find_enclosing_function<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
) -> Option<(NodeId, NodeId)> {
    let mut current = ctx.nodes().parent_node(node.id());

    // First, find the FunctionBody
    while !matches!(current.kind(), AstKind::FunctionBody(_)) {
        // Check if we've reached the root
        if matches!(current.kind(), AstKind::Program(_)) {
            return None;
        }
        current = ctx.nodes().parent_node(current.id());
    }
    let function_body_id = current.id();

    // Then find the Function or ArrowFunctionExpression
    current = ctx.nodes().parent_node(current.id());
    if matches!(current.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
        Some((current.id(), function_body_id))
    } else {
        None
    }
}

fn get_param_name<'a>(param: &FormalParameter<'a>) -> Option<&'a str> {
    match &param.pattern.kind {
        BindingPatternKind::BindingIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

fn is_first_statement<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
    function_body_id: NodeId,
) -> bool {
    let function_body_node = ctx.nodes().get_node(function_body_id);
    let AstKind::FunctionBody(body) = function_body_node.kind() else {
        return false;
    };

    // Get the first statement in the function body
    let Some(first_stmt) = body.statements.first() else {
        return false;
    };

    // The node's span should match the first statement's span
    first_stmt.span() == node.kind().span()
}

fn check_no_extra_references<'a>(
    ctx: &LintContext<'a>,
    param_ident_span: Span,
    param: &FormalParameter<'a>,
) -> bool {
    // Get the symbol for the parameter
    let BindingPatternKind::BindingIdentifier(binding_ident) = &param.pattern.kind else {
        return false;
    };

    let Some(symbol_id) = binding_ident.symbol_id.get() else {
        return false;
    };

    // Check how many times the parameter is referenced
    let references: Vec<_> = ctx.scoping().get_resolved_references(symbol_id).collect();

    // For `const bar = foo || 'bar'`, the parameter should only be referenced once
    // (in the LogicalExpression)
    if references.len() != 1 {
        return false;
    }

    // The single reference should be the one in the LogicalExpression
    let reference = &references[0];
    ctx.semantic().reference_span(reference) == param_ident_span
}

fn check_no_extra_references_assignment<'a>(
    ctx: &LintContext<'a>,
    param_ident_span: Span,
    param: &FormalParameter<'a>,
) -> bool {
    // Get the symbol for the parameter
    let BindingPatternKind::BindingIdentifier(binding_ident) = &param.pattern.kind else {
        return false;
    };

    let Some(symbol_id) = binding_ident.symbol_id.get() else {
        return false;
    };

    // For assignment `foo = foo || 'bar'`, the parameter should have exactly 2 references:
    // 1. The read in the LogicalExpression (foo || 'bar')
    // 2. The write in the AssignmentExpression (foo = ...)
    let references: Vec<_> = ctx.scoping().get_resolved_references(symbol_id).collect();

    if references.len() != 2 {
        return false;
    }

    // One should be a read (in logical expr), one should be a write (assignment target)
    let reads: Vec<_> = references.iter().filter(|r| !r.is_write()).collect();
    let write_count = references.iter().filter(|r| r.is_write()).count();

    if reads.len() != 1 || write_count != 1 {
        return false;
    }

    // The read should be at the same span as param_ident
    ctx.semantic().reference_span(reads[0]) == param_ident_span
}

fn check_expression<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
    left_name: &str,
    right: &Expression<'a>,
    is_assignment: bool,
    stmt_span: Span,
) {
    // Right must be a LogicalExpression with || or ??
    let Expression::LogicalExpression(logical_expr) = right.without_parentheses() else {
        return;
    };

    if !matches!(logical_expr.operator, LogicalOperator::Or | LogicalOperator::Coalesce) {
        return;
    }

    // Left side of logical expression must be an identifier
    let Expression::Identifier(param_ident) = logical_expr.left.without_parentheses() else {
        return;
    };

    let param_name = param_ident.name.as_str();

    // Right side of logical expression must be a literal
    if !is_literal(&logical_expr.right) {
        return;
    }

    // For assignment (foo = foo || 'bar'), the left side must match the parameter
    if is_assignment && left_name != param_name {
        return;
    }

    // Find the enclosing function
    let Some((function_id, function_body_id)) = find_enclosing_function(ctx, node) else {
        return;
    };

    // Get the function parameters
    let function_node = ctx.nodes().get_node(function_id);
    let params = match function_node.kind() {
        AstKind::Function(func) => &func.params,
        AstKind::ArrowFunctionExpression(arrow) => &arrow.params,
        _ => return,
    };

    // Find the parameter that matches param_name
    let Some((param_index, param)) =
        params.items.iter().enumerate().find(|(_, p)| get_param_name(p) == Some(param_name))
    else {
        return;
    };

    // Parameter must be the last parameter (no parameters after it)
    if param_index != params.items.len() - 1 {
        return;
    }

    // Parameter must not have a default value already
    if matches!(param.pattern.kind, BindingPatternKind::AssignmentPattern(_)) {
        return;
    }

    // If there's a rest parameter, the last item parameter is not actually last
    if params.rest.is_some() {
        return;
    }

    // Parameter must be a simple identifier (not destructuring or rest)
    if !matches!(param.pattern.kind, BindingPatternKind::BindingIdentifier(_)) {
        return;
    }

    // Check that this statement is the first in the function body
    if !is_first_statement(ctx, node, function_body_id) {
        return;
    }

    // Check references based on assignment type
    if is_assignment {
        // For assignment, check that the parameter is not referenced before this assignment
        // and not referenced elsewhere except in this assignment
        if !check_no_extra_references_assignment(ctx, param_ident.span, param) {
            return;
        }
    } else {
        // For non-assignment (const bar = foo || 'bar'), check that bar is not used elsewhere with foo
        if !check_no_extra_references(ctx, param_ident.span, param) {
            return;
        }
    }

    ctx.diagnostic(prefer_default_parameters_diagnostic(stmt_span, param_name));
}

impl Rule for PreferDefaultParameters {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Handle: foo = foo || 'bar'
            AstKind::ExpressionStatement(expr_stmt) => {
                if let Expression::AssignmentExpression(assign_expr) = &expr_stmt.expression {
                    if assign_expr.operator != AssignmentOperator::Assign {
                        return;
                    }
                    if let AssignmentTarget::AssignmentTargetIdentifier(left_ident) =
                        &assign_expr.left
                    {
                        check_expression(
                            ctx,
                            node,
                            &left_ident.name,
                            &assign_expr.right,
                            true,
                            expr_stmt.span,
                        );
                    }
                }
            }
            // Handle: const bar = foo || 'bar'
            AstKind::VariableDeclaration(var_decl) => {
                if var_decl.declarations.len() != 1 {
                    return;
                }
                let declarator = &var_decl.declarations[0];
                let Some(init) = &declarator.init else {
                    return;
                };
                if let BindingPatternKind::BindingIdentifier(left_ident) = &declarator.id.kind {
                    check_expression(ctx, node, &left_ident.name, init, false, var_decl.span);
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
        "function abc(foo = { bar: 123 }) { }",
        "function abc({ bar } = { bar: 123 }) { }",
        "function abc({ bar = 123 } = { bar }) { }",
        "function abc(foo = fooDefault) { }",
        "function abc(foo = {}) { }",
        "function abc(foo = 'bar') { }",
        "function abc({ bar = 123 } = {}) { }",
        "const abc = (foo = 'bar') => { };",
        "foo = foo || 'bar';",
        "const bar = foo || 'bar';",
        "const abc = function(foo = { bar: 123 }) { }",
        "const abc = function({ bar } = { bar: 123 }) { }",
        "const abc = function({ bar = 123 } = {}) { }",
        "function abc(foo) {
				foo = foo || bar();
			}",
        "function abc(foo) {
				foo = foo || {bar};
			}",
        "function abc(foo) {
				const {bar} = foo || 123;
			}",
        "function abc(foo, bar) {
				bar = foo || 'bar';
			}",
        "function abc(foo, bar) {
				foo = foo || 'bar';
				baz();
			}",
        "function abc(foo) {
				foo = foo && 'bar';
			}",
        "function abc(foo) {
				foo = foo || 1 && 2 || 3;
			}",
        "function abc(foo) {
				foo = !foo || 'bar';
			}",
        "function abc(foo) {
				foo = (foo && bar) || baz;
			}",
        "function abc(foo = 123) {
				foo = foo || 'bar';
			}",
        "function abc() {
				let foo = 123;
				foo = foo || 'bar';
			}",
        "function abc() {
				let foo = 123;
				const bar = foo || 'bar';
			}",
        "const abc = (foo, bar) => {
				bar = foo || 'bar';
			};",
        "const abc = function(foo, bar) {
				bar = foo || 'bar';
			}",
        "const abc = function(foo) {
				foo = foo || bar();
			}",
        "function abc(foo) {
				function def(bar) {
					foo = foo || 'bar';
				}
			}",
        "function abc(foo) {
				const bar = foo = foo || 123;
			}",
        "function abc(foo) {
				bar(foo = foo || 1);
				baz(foo);
			}",
        "function abc(foo) {
				console.log(foo);
				foo = foo || 123;
			}",
        "function abc(foo) {
				console.log(foo);
				foo = foo || 'bar';
			}",
        "function abc(foo) {
				const bar = foo || 'bar';
				console.log(foo, bar);
			}",
        "function abc(foo) {
				let bar = 123;
				bar = foo;
				foo = foo || 123;
			}",
        "function abc(foo) {
				bar();
				foo = foo || 123;
			}",
        "const abc = (foo) => {
				bar();
				foo = foo || 123;
			};",
        "const abc = function(foo) {
				bar();
				foo = foo || 123;
			};",
        "function abc(foo) {
				sideEffects();
				foo = foo || 123;
				function sideEffects() {
					foo = 456;
				}
			}",
        "function abc(foo) {
				const bar = sideEffects();
				foo = foo || 123;
				function sideEffects() {
					foo = 456;
				}
			}",
        "function abc(foo) {
				const bar = sideEffects() + 123;
				foo = foo || 123;
				function sideEffects() {
					foo = 456;
				}
			}",
        "function abc(foo) {
				const bar = !sideEffects();
				foo = foo || 123;
				function sideEffects() {
					foo = 456;
				}
			}",
        "function abc(foo) {
				const bar = function() {
					foo = 456;
				}
				foo = foo || 123;
			}",
        "function abc(...foo) {
				foo = foo || 'bar';
			}",
        "function abc(foo = 'bar') {
				foo = foo || 'baz';
			}",
        "function abc(foo, bar) {
				const { baz, ...rest } = bar;
				foo = foo || 123;
			}",
        "function abc(foo, bar) {
				const baz = foo?.bar;
				foo = foo || 123;
			}",
        "function abc(foo, bar) {
				import('foo');
				foo = foo || 123;
			}",
    ];

    let fail = vec![
        // Assignment pattern: foo = foo || 'bar'
        "function abc(foo) {
				foo = foo || 'bar';
			}",
        // Assignment pattern with number literal
        "function abc(foo) {
				foo = foo || 123;
			}",
        // Variable declaration pattern: const bar = foo || 'bar'
        "function abc(foo) {
				const bar = foo || 'bar';
			}",
        // Arrow function
        "const abc = (foo) => {
				foo = foo || 'bar';
			};",
        // Function expression
        "const abc = function(foo) {
				foo = foo || 'bar';
			}",
        // Nullish coalescing operator
        "function abc(foo) {
				foo = foo ?? 'bar';
			}",
        // Boolean literal
        "function abc(foo) {
				foo = foo || false;
			}",
        // Null literal
        "function abc(foo) {
				foo = foo || null;
			}",
        // Template literal
        "function abc(foo) {
				foo = foo || `bar`;
			}",
    ];

    Tester::new(PreferDefaultParameters::NAME, PreferDefaultParameters::PLUGIN, pass, fail)
        .test_and_snapshot();
}
