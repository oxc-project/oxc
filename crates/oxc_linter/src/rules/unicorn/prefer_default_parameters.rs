use oxc_ast::{
    AstKind,
    ast::{
        AssignmentOperator, AssignmentTarget, BindingPatternKind, Expression, FormalParameter,
        LogicalOperator, Statement,
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

impl Rule for PreferDefaultParameters {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assign_expr) => {
                if assign_expr.operator != AssignmentOperator::Assign {
                    return;
                }
                if let AssignmentTarget::AssignmentTargetIdentifier(left_ident) = &assign_expr.left
                {
                    check_expression(
                        ctx,
                        node,
                        &left_ident.name,
                        &assign_expr.right,
                        true,
                        assign_expr.span,
                    );
                }
            }
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

fn check_expression<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
    left_name: &str,
    right: &Expression<'a>,
    is_assignment: bool,
    stmt_span: Span,
) {
    let Expression::LogicalExpression(logical_expr) = right.without_parentheses() else {
        return;
    };

    if !matches!(logical_expr.operator, LogicalOperator::Or | LogicalOperator::Coalesce) {
        return;
    }

    let Expression::Identifier(param_ident) = logical_expr.left.without_parentheses() else {
        return;
    };

    let param_name = param_ident.name.as_str();

    if !logical_expr.right.get_inner_expression().is_literal() {
        return;
    }

    if is_assignment && left_name != param_name {
        return;
    }

    let Some((function_id, function_body_id)) = find_enclosing_function(ctx, node) else {
        return;
    };

    let function_node = ctx.nodes().get_node(function_id);
    let params = match function_node.kind() {
        AstKind::Function(func) => &func.params,
        AstKind::ArrowFunctionExpression(arrow) => &arrow.params,
        _ => return,
    };

    let Some((param_index, param)) = params.items.iter().enumerate().find(|(_, p)| {
        p.pattern.get_binding_identifier().map(|ident| ident.name.as_str()) == Some(param_name)
    }) else {
        return;
    };

    if param_index != params.items.len() - 1 {
        return;
    }

    if matches!(param.pattern.kind, BindingPatternKind::AssignmentPattern(_)) {
        return;
    }

    if params.rest.is_some() {
        return;
    }

    if !has_no_side_effects_before(ctx, node, function_body_id, param_name) {
        return;
    }

    if is_assignment {
        if !check_no_extra_references_assignment(ctx, param_ident.span, param) {
            return;
        }
    } else if !check_no_extra_references(ctx, param_ident.span, param) {
        return;
    }

    ctx.diagnostic(prefer_default_parameters_diagnostic(stmt_span, param_name));
}

fn find_enclosing_function<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
) -> Option<(NodeId, NodeId)> {
    let mut current = ctx.nodes().parent_node(node.id());

    while !matches!(current.kind(), AstKind::FunctionBody(_)) {
        if matches!(current.kind(), AstKind::Program(_)) {
            return None;
        }
        current = ctx.nodes().parent_node(current.id());
    }
    let function_body_id = current.id();

    current = ctx.nodes().parent_node(current.id());
    if matches!(current.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
        Some((current.id(), function_body_id))
    } else {
        None
    }
}

fn has_no_side_effects_before<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
    function_body_id: NodeId,
    param_name: &str,
) -> bool {
    let function_body_node = ctx.nodes().get_node(function_body_id);
    let AstKind::FunctionBody(body) = function_body_node.kind() else {
        return false;
    };

    let node_span = node.kind().span();

    for stmt in &body.statements {
        let stmt_matches = match stmt {
            Statement::ExpressionStatement(expr_stmt) => expr_stmt.expression.span() == node_span,
            Statement::VariableDeclaration(var_decl) => var_decl.span == node_span,
            _ => stmt.span() == node_span,
        };

        if stmt_matches {
            return true;
        }

        if !is_side_effect_free_statement(stmt, param_name) {
            return false;
        }
    }

    false
}

fn is_side_effect_free_statement(stmt: &oxc_ast::ast::Statement, param_name: &str) -> bool {
    use oxc_ast::ast::Statement;

    match stmt {
        Statement::VariableDeclaration(var_decl) => var_decl.declarations.iter().all(|decl| {
            if let Some(init) = &decl.init {
                is_side_effect_free_expression(init, param_name)
            } else {
                true
            }
        }),
        Statement::ExpressionStatement(expr_stmt) => {
            is_side_effect_free_expression(&expr_stmt.expression, param_name)
        }
        Statement::FunctionDeclaration(_) => true,
        _ => false,
    }
}

fn is_side_effect_free_expression(expr: &Expression, param_name: &str) -> bool {
    match expr.without_parentheses() {
        Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_) => true,
        Expression::Identifier(ident) => ident.name.as_str() != param_name,
        Expression::AssignmentExpression(assign) => {
            let target_ok = match &assign.left {
                AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    ident.name.as_str() != param_name
                }
                _ => false,
            };
            target_ok && is_side_effect_free_expression(&assign.right, param_name)
        }
        Expression::BinaryExpression(bin) => {
            is_side_effect_free_expression(&bin.left, param_name)
                && is_side_effect_free_expression(&bin.right, param_name)
        }
        Expression::UnaryExpression(unary) => {
            !matches!(unary.operator, oxc_ast::ast::UnaryOperator::Delete)
                && is_side_effect_free_expression(&unary.argument, param_name)
        }
        _ => false,
    }
}

fn check_no_extra_references<'a>(
    ctx: &LintContext<'a>,
    param_ident_span: Span,
    param: &FormalParameter<'a>,
) -> bool {
    let BindingPatternKind::BindingIdentifier(binding_ident) = &param.pattern.kind else {
        return false;
    };

    let Some(symbol_id) = binding_ident.symbol_id.get() else {
        return false;
    };

    let references: Vec<_> = ctx.scoping().get_resolved_references(symbol_id).collect();

    if references.len() != 1 {
        return false;
    }

    let reference = &references[0];
    ctx.semantic().reference_span(reference) == param_ident_span
}

fn check_no_extra_references_assignment<'a>(
    ctx: &LintContext<'a>,
    param_ident_span: Span,
    param: &FormalParameter<'a>,
) -> bool {
    let BindingPatternKind::BindingIdentifier(binding_ident) = &param.pattern.kind else {
        return false;
    };

    let Some(symbol_id) = binding_ident.symbol_id.get() else {
        return false;
    };

    let (has_matching_read, writes) = ctx.scoping().get_resolved_references(symbol_id).fold(
        (false, 0usize),
        |(has_matching_read, writes), r| {
            if r.is_write() {
                (has_matching_read, writes + 1)
            } else {
                let is_matching = ctx.semantic().reference_span(r) == param_ident_span;
                (has_matching_read || is_matching, writes)
            }
        },
    );

    writes == 1 && has_matching_read
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
        r"function abc(foo) {
	foo = foo || 123;
}",
        r"function abc(foo) {
	foo = foo || true;
}",
        r"function abc(foo) {
	foo = foo || 123;
	console.log(foo);
}",
        r"function abc(foo) {
	const bar = foo || 'bar';
}",
        r"function abc(foo) {
	let bar = foo || 'bar';
}",
        r"const abc = function(foo) {
	foo = foo || 123;
}",
        r"const abc = (foo) => {
	foo = foo || 'bar';
};",
        r"const abc = foo => {
	foo = foo || 'bar';
};",
        r"const abc = (foo) => {
	const bar = foo || 'bar';
};",
        r"function abc(foo) {
	foo = foo || 'bar';
	bar();
	baz();
}",
        r"function abc(foo) {
	foo = foo ?? 123;
}",
        r"function abc(foo) {
	const bar = foo || 'bar';
	console.log(bar);
}",
        r"const abc = function(foo) {
	const bar = foo || 'bar';
	console.log(bar);
}",
        r"foo = {
	abc(foo) {
		foo = foo || 123;
	}
};",
        r"foo = {
	abc(foo) {
		foo = foo || 123;
	},
	def(foo) { }
};",
        r"class Foo {
	abc(foo) {
		foo = foo || 123;
	}
}",
        r"class Foo {
	abc(foo) {
		foo = foo || 123;
	}
	def(foo) { }
}",
        r"function abc(foo) { foo = foo || 'bar'; }",
        r"function abc(foo) { foo = foo || 'bar';}",
        r"const abc = function(foo) { foo = foo || 'bar';}",
        r"function abc(foo) {
	foo = foo || 'bar'; bar(); baz();
}",
        r"function abc(foo) {
	foo = foo || 'bar';
	function def(bar) {
		bar = bar || 'foo';
	}
}",
        r"function abc(foo) {
	foo += 'bar';
	function def(bar) {
		bar = bar || 'foo';
	}
	function ghi(baz) {
		const bay = baz || 'bar';
	}
	foo = foo || 'bar';
}",
        r"foo = {
	abc(foo) {
		foo = foo || 123;
	},
	def(foo) {
		foo = foo || 123;
	}
};",
        r"class Foo {
	abc(foo) {
		foo = foo || 123;
	}
	def(foo) {
		foo = foo || 123;
	}
}",
        r"function abc(foo) {
	const noSideEffects = 123;
	foo = foo || 123;
}",
        r"const abc = function(foo) {
	let bar = true;
	bar = false;

	foo = foo || 123;
	console.log(foo);
}",
        r"function abc(foo) {
	const bar = function() {};
	foo = foo || 123;
}",
    ];

    let _fix = vec![
        (
            r"function abc(foo) {
	foo = foo || 123;
}",
            r"function abc(foo = 123) {
}",
        ),
        (
            r"function abc(foo) {
	foo = foo || true;
}",
            r"function abc(foo = true) {
}",
        ),
        (
            r"function abc(foo) {
	foo = foo || 123;
	console.log(foo);
}",
            r"function abc(foo = 123) {
	console.log(foo);
}",
        ),
        (
            r"function abc(foo) {
	const bar = foo || 'bar';
}",
            r"function abc(bar = 'bar') {
}",
        ),
        (
            r"function abc(foo) {
	let bar = foo || 'bar';
}",
            r"function abc(bar = 'bar') {
}",
        ),
        (
            r"const abc = function(foo) {
	foo = foo || 123;
}",
            r"const abc = function(foo = 123) {
}",
        ),
        (
            r"const abc = (foo) => {
	foo = foo || 'bar';
};",
            r"const abc = (foo = 'bar') => {
};",
        ),
        (
            r"const abc = foo => {
	foo = foo || 'bar';
};",
            r"const abc = (foo = 'bar') => {
};",
        ),
        (
            r"const abc = (foo) => {
	const bar = foo || 'bar';
};",
            r"const abc = (bar = 'bar') => {
};",
        ),
        (
            r"function abc(foo) {
	foo = foo || 'bar';
	bar();
	baz();
}",
            r"function abc(foo = 'bar') {
	bar();
	baz();
}",
        ),
        (
            r"function abc(foo) {
	foo = foo ?? 123;
}",
            r"function abc(foo = 123) {
}",
        ),
        (
            r"function abc(foo) {
	const bar = foo || 'bar';
	console.log(bar);
}",
            r"function abc(bar = 'bar') {
	console.log(bar);
}",
        ),
        (
            r"const abc = function(foo) {
	const bar = foo || 'bar';
	console.log(bar);
}",
            r"const abc = function(bar = 'bar') {
	console.log(bar);
}",
        ),
        (
            r"foo = {
	abc(foo) {
		foo = foo || 123;
	}
};",
            r"foo = {
	abc(foo = 123) {
	}
};",
        ),
        (
            r"foo = {
	abc(foo) {
		foo = foo || 123;
	},
	def(foo) { }
};",
            r"foo = {
	abc(foo = 123) {
	},
	def(foo) { }
};",
        ),
        (
            r"class Foo {
	abc(foo) {
		foo = foo || 123;
	}
}",
            r"class Foo {
	abc(foo = 123) {
	}
}",
        ),
        (
            r"class Foo {
	abc(foo) {
		foo = foo || 123;
	}
	def(foo) { }
}",
            r"class Foo {
	abc(foo = 123) {
	}
	def(foo) { }
}",
        ),
        (r"function abc(foo) { foo = foo || 'bar'; }", r"function abc(foo = 'bar') { }"),
        (r"function abc(foo) { foo = foo || 'bar';}", r"function abc(foo = 'bar') { }"),
        (
            r"const abc = function(foo) { foo = foo || 'bar';}",
            r"const abc = function(foo = 'bar') { }",
        ),
        (
            r"function abc(foo) {
	foo = foo || 'bar'; bar(); baz();
}",
            r"function abc(foo = 'bar') {
	bar(); baz();
}",
        ),
        (
            r"function abc(foo) {
	foo = foo || 'bar';
	function def(bar) {
		bar = bar || 'foo';
	}
}",
            r"function abc(foo = 'bar') {
	function def(bar) {
		bar = bar || 'foo';
	}
}",
        ),
        (
            r"function abc(foo) {
	foo += 'bar';
	function def(bar) {
		bar = bar || 'foo';
	}
	function ghi(baz) {
		const bay = baz || 'bar';
	}
	foo = foo || 'bar';
}",
            r"function abc(foo) {
	foo += 'bar';
	function def(bar = 'foo') {
	}
	function ghi(baz) {
		const bay = baz || 'bar';
	}
	foo = foo || 'bar';
}",
        ),
        (
            r"foo = {
	abc(foo) {
		foo = foo || 123;
	},
	def(foo) {
		foo = foo || 123;
	}
};",
            r"foo = {
	abc(foo = 123) {
	},
	def(foo) {
		foo = foo || 123;
	}
};",
        ),
        (
            r"class Foo {
	abc(foo) {
		foo = foo || 123;
	}
	def(foo) {
		foo = foo || 123;
	}
}",
            r"class Foo {
	abc(foo = 123) {
	}
	def(foo) {
		foo = foo || 123;
	}
}",
        ),
        (
            r"function abc(foo) {
	const noSideEffects = 123;
	foo = foo || 123;
}",
            r"function abc(foo = 123) {
	const noSideEffects = 123;
}",
        ),
        (
            r"const abc = function(foo) {
	let bar = true;
	bar = false;

	foo = foo || 123;
	console.log(foo);
}",
            r"const abc = function(foo = 123) {
	let bar = true;
	bar = false;

	console.log(foo);
}",
        ),
        (
            r"function abc(foo) {
	const bar = function() {};
	foo = foo || 123;
}",
            r"function abc(foo = 123) {
	const bar = function() {};
}",
        ),
    ];

    Tester::new(PreferDefaultParameters::NAME, PreferDefaultParameters::PLUGIN, pass, fail)
        .test_and_snapshot();
}
