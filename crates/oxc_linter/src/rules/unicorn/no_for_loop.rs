use oxc_ast::{AstKind, ast::*};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Reference;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::get_declaration_of_variable, context::LintContext, rule::Rule};

fn no_for_loop_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use a `for-of` loop instead of this classical `for` loop.")
        .with_help("Rewrite the loop as `for (const el of array) { … }`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoForLoop;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `for-of` loops over classical counter based `for` loops.
    ///
    /// ### Why is this bad?
    ///
    /// * A `for-of` loop makes intent explicit – _iterate over all elements_.
    /// * It removes boiler-plate (`i`, `arr.length`, manual `++`).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (let i = 0; i < items.length; i++) {
    ///   const item = items[i];
    ///   console.log(item);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// for (const item of items) {
    ///   console.log(item);
    /// }
    /// ```
    NoForLoop,
    unicorn,
    style,
    pending
);

impl Rule for NoForLoop {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ForStatement(for_stmt) = node.kind() else { return };

        let Some(init) = &for_stmt.init else { return };
        let ForStatementInit::VariableDeclaration(init) = init else { return };
        if init.declarations.len() != 1
            || init.declarations[0]
                .init
                .as_ref()
                .is_some_and(|e| !e.get_inner_expression().is_number_0())
        {
            return;
        }
        let BindingPatternKind::BindingIdentifier(ident) = &init.declarations[0].id.kind else {
            return;
        };

        let Some(test) = &for_stmt.test else { return };
        let Expression::BinaryExpression(bin_expr) = test.get_inner_expression() else { return };
        let array = {
            let (lesser, greater) = match bin_expr.operator {
                BinaryOperator::LessThan => (&bin_expr.left, &bin_expr.right),
                BinaryOperator::GreaterThan => (&bin_expr.right, &bin_expr.left),
                _ => return,
            };

            if lesser.get_identifier_reference().is_none_or(|lesser_ident| {
                ctx.scoping().get_reference(lesser_ident.reference_id()).symbol_id()
                    != Some(ident.symbol_id())
            }) {
                return;
            }

            let Some(greater) = greater.get_inner_expression().as_member_expression() else {
                return;
            };
            if greater.static_property_name().is_none_or(|name| name != "length") {
                return;
            }
            let Some(ident) = greater.object().get_identifier_reference() else {
                return;
            };
            ident
        };

        let Some(update) = &for_stmt.update else { return };
        match update {
            Expression::UpdateExpression(u) => {
                if u.operator != UpdateOperator::Increment {
                    return;
                }
                let SimpleAssignmentTarget::AssignmentTargetIdentifier(update_ident) = &u.argument
                else {
                    return;
                };

                if ctx.scoping().get_reference(update_ident.reference_id()).symbol_id()
                    != Some(ident.symbol_id())
                {
                    return;
                }
            }
            Expression::AssignmentExpression(u) => {
                let AssignmentTarget::AssignmentTargetIdentifier(update_ident) = &u.left else {
                    return;
                };
                if ctx.scoping().get_reference(update_ident.reference_id()).symbol_id()
                    != Some(ident.symbol_id())
                {
                    return;
                }

                match u.operator {
                    AssignmentOperator::Addition
                        if u.right.get_inner_expression().is_number_value(1.0) => {}
                    AssignmentOperator::Assign => {
                        let Expression::BinaryExpression(right) = &u.right else { return };
                        if right.operator != BinaryOperator::Addition {
                            return;
                        }
                        if (!(right.left.get_inner_expression().is_number_value(1.0))
                            && (right
                                .right
                                .get_inner_expression()
                                .get_identifier_reference()
                                .is_some_and(|i| {
                                    ctx.scoping().get_reference(i.reference_id()).symbol_id()
                                        == Some(ident.symbol_id())
                                })))
                            && !((right.right.get_inner_expression().is_number_value(1.0))
                                && (right
                                    .left
                                    .get_inner_expression()
                                    .get_identifier_reference()
                                    .is_some_and(|i| {
                                        ctx.scoping().get_reference(i.reference_id()).symbol_id()
                                            == Some(ident.symbol_id())
                                    })))
                        {
                            return;
                        }
                    }
                    _ => return,
                }
            }
            _ => return,
        }

        let Some(array_symbol_id) = ctx.scoping().get_reference(array.reference_id()).symbol_id()
        else {
            return;
        };

        if let Some(decl) = get_declaration_of_variable(array, ctx.semantic()) {
            if let AstKind::VariableDeclarator(decl) = decl.kind() {
                if let BindingPatternKind::BindingIdentifier(_) = decl.id.kind {
                    if let Some(
                        Expression::StringLiteral(_)
                        | Expression::NumericLiteral(_)
                        | Expression::BooleanLiteral(_),
                    ) = decl.init
                    {
                        return;
                    }
                }
            }
        }

        let Statement::BlockStatement(body_block) = &for_stmt.body else { return };

        let mut is_reference_used_as_index = false;
        for reference in ctx.symbol_references(ident.symbol_id()) {
            if !body_block.span.contains_inclusive(reference_span(reference, ctx)) {
                continue;
            }
            if reference.is_write() {
                return;
            }
            let node = ctx.nodes().get_node(reference.node_id());
            let parent = ctx.nodes().parent_node(node.id());
            if parent.is_some_and(|node| {
                if let AstKind::MemberExpression(member_expr) = node.kind() {
                    member_expr.is_computed()
                        && member_expr.object().get_identifier_reference().is_some_and(
                            |reference| {
                                ctx.scoping().get_reference(reference.reference_id()).symbol_id()
                                    == Some(array_symbol_id)
                            },
                        )
                } else {
                    false
                }
            }) {
                is_reference_used_as_index = true;
            }
        }

        if !is_reference_used_as_index {
            return;
        }

        for reference in ctx.symbol_references(array_symbol_id) {
            if !body_block.span.contains_inclusive(reference_span(reference, ctx)) {
                continue;
            }
            if reference.is_write() {
                return;
            }
            let Some(parent) = ctx.nodes().parent_node(reference.node_id()) else { continue };
            let AstKind::MemberExpression(member_expr) = parent.kind() else { return };
            let MemberExpression::ComputedMemberExpression(computed_member_expr) = member_expr
            else {
                return;
            };
            if computed_member_expr
                .expression
                .get_inner_expression()
                .get_identifier_reference()
                .is_none_or(|r| {
                    ctx.scoping().get_reference(r.reference_id()).symbol_id()
                        != Some(ident.symbol_id())
                })
            {
                return;
            }
            if let Some(AstKind::SimpleAssignmentTarget(_)) = ctx.nodes().parent_kind(parent.id()) {
                return;
            }
        }

        ctx.diagnostic(no_for_loop_diagnostic(Span::sized(for_stmt.span.start, 3)));
    }
}

fn reference_span(reference: &Reference, ctx: &LintContext<'_>) -> Span {
    ctx.nodes().get_node(reference.node_id()).span()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "for (;;);",
        "for (;;) {}",
        "for (a;; c) { d }",
        "for (a; b;) { d }",
        "for (the; love; of) { god }",
        "for ([a] = b; f(c); d--) { arr[d] }",
        "for (var a = b; c < arr.length; d++) { arr[e] }",
        "for (const x of xs) {}",
        "for (var j = 0; j < 10; j++) {}",
        "for (i = 0; i < arr.length; i++) {
				el = arr[i];
				console.log(i, el);
			}",
        "var foo = function () {
				for (var i = 0; i < bar.length; i++) {
				}
			};",
        "for (let i = 0, j = 0; i < arr.length; i++) {
				const el = arr[i];
				console.log(i, el);
			}",
        "for (let {i} = 0; i < arr.length; i++) {
				const el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; f(i, arr.length); i++) {
				const el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; i < arr.size; i++) {
				const el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; j < arr.length; i++) {
				const el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; i <= arr.length; i++) {
				const el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; arr.length > i;) {
				let el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; arr.length > i; i--) {
				let el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; arr.length > i; f(i)) {
				let el = arr[i];
				console.log(i, el);
			}",
        "for (let i = 0; arr.length > i; i = f(i)) {
				let el = arr[i];
				console.log(i, el);
			}",
        "const arr = []; for (let i = 0; arr.length > i; i ++);",
        "const arr = []; for (let i = 0; arr.length > i; i ++) console.log(NaN)",
        "const arr = []; for (let i = 0; i < arr.length; ++i) {
				const el = f(i);
				console.log(i, el);
			}",
        "const xs = [];
        for (var j = 0; j < xs.length; j++) {
				var x;
			}",
        "const xs = [];
        for (var j = 0; j < xs.length; j++) {
				var {x} = y;
			}",
        "const arr = [];
        for (let i = 0; i < arr.length; i++) {
				console.log(i);
			}",
        "const input = [];
        for (let i = 0; i < input.length; i++) {
				const el = input[i];
				i++;
				console.log(i, el);
			}",
        "const input = [];
        for (let i = 0; i < input.length; i++) {
				const el = input[i];
				i = 4;
				console.log(i, el);
			}",
        "const arr = [];
        for (let i = 0; i < arr.length;i++) {
				console.log(arr[i]);
				arr.reverse();
			}",
        "const arr = [];
        for (let i = 0; i < arr.length; i++) {
				arr[i] = i + 2;
			}",
        "const cities = [];
        for (let i = 0; i < cities.length; i++) {
				const foo = function () {
					console.log(cities)
				}
			}",
        r#"const notArray = "abc"; for (let i = 0; i < notArray.length; i++) { console.log(notArray[i]); }"#,
        "const notArray = 123; for (let i = 0; i < notArray.length; i++) { console.log(notArray[i]); }",
        "const notArray = true; for (let i = 0; i < notArray.length; i++) { console.log(notArray[i]); }",
        "for (;;);",
        "for (;;) {}",
        "for (var j = 0; j < 10; j++) {}",
        "const arr = [];
        for (i = 0; i < arr.length; i++) {
				el = arr[i];
				console.log(i, el);
			}",
        "const bar = [];
        var foo = function () {
				for (var i = 0; i < bar.length; i++) {
				}
			};",
    ];

    let fail = vec![
        "const positions = []; 
        for (let i = 0; i < positions.length; i++) {
        		let last: vscode.Position | vscode.Range = positions[i];
        		let selectionRange = allProviderRanges[i];
        	}",
        "const positions = [];
        for (let i = 0; i < positions.length; i++) {
				const    last   /* comment */    : /* comment */ Position = positions[i];
				console.log(i);
			}",
        "const positions = [];
        for (let i = 0; i < positions.length; i++) {
				let last: vscode.Position | vscode.Range = positions[i];
			}",
        "const arr = [];
        for (let i = 0; i < arr.length; i += 1) {
				console.log(arr[i])
			}",
        "const plugins = [];
        for (let i = 0; i < plugins.length; i++) {
				let plugin = plugins[i];
				plugin = calculateSomeNewValue();
				// ...
			}",
        "const array = [];
        for (
				let i = 0;
				i < array.length;
				i++
			)
			// comment (foo)
				{
					var foo = array[i];
					foo = bar();
				}",
        "const array = [];
        for (let i = 0; i < array.length; i++) {
				let foo = array[i];
			}",
        "const array = [];
        for (let i = 0; i < array.length; i++) {
				const foo = array[i];
			}",
        "const array = [];
        for (let i = 0; i < array.length; i++) {
				var foo = array[i], bar = 1;
			}",
    ];

    Tester::new(NoForLoop::NAME, NoForLoop::PLUGIN, pass, fail).test_and_snapshot();
}
