use oxc_ast::{
    AstKind,
    ast::{AssignmentOperator, AssignmentTarget, Expression, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_empty_array_expression};

fn no_array_concat_in_loop_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `Array#concat()` to accumulate an array in a loop.")
        .with_help("Accumulate with `Array#push()` instead, so the array is not copied on every iteration.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayConcatInLoop;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows accumulating an array with `Array#concat()` inside a loop.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#concat()` returns a new array, so reassigning an accumulator to `accumulator.concat(...)`
    /// inside a loop copies everything accumulated so far on every iteration, making the loop
    /// quadratic in the number of accumulated items.
    ///
    /// This rule only reports local `let`/`var` bindings initialized with an empty array literal. It
    /// does not try to infer array-like values or custom `concat` methods, and it provides no fix:
    /// `push()` is not always equivalent, because `concat()` accepts (and spreads) multiple
    /// arguments and aliasing the previous array can be observable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let result = [];
    /// for (const chunk of chunks) {
    ///     result = result.concat(chunk);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const result = [];
    /// for (const chunk of chunks) {
    ///     result.push(chunk);
    /// }
    ///
    /// // declared inside the loop: a new array is created every iteration anyway
    /// for (const chunk of chunks) {
    ///     let result = [];
    ///     result = result.concat(chunk);
    /// }
    /// ```
    NoArrayConcatInLoop,
    unicorn,
    perf,
    version = "next",
    short_description = "Disallow accumulating an array with `Array#concat()` in a loop.",
);

impl Rule for NoArrayConcatInLoop {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assignment) = node.kind() else {
            return;
        };

        if assignment.operator != AssignmentOperator::Assign {
            return;
        }

        let AssignmentTarget::AssignmentTargetIdentifier(accumulator) = &assignment.left else {
            return;
        };

        // `accumulator = accumulator.concat(...)`
        let Expression::CallExpression(call_expr) = assignment.right.get_inner_expression() else {
            return;
        };

        if call_expr.optional || call_expr.arguments.is_empty() {
            return;
        }

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if member_expr.is_computed() || member_expr.optional() {
            return;
        }

        let Some((property_span, property_name)) = member_expr.static_property_info() else {
            return;
        };
        if property_name != "concat" {
            return;
        }

        // The called method must be on the accumulator itself, e.g. not `other.concat(chunk)`.
        let Expression::Identifier(accumulator_reference) =
            member_expr.object().get_inner_expression()
        else {
            return;
        };

        let scoping = ctx.scoping();
        let Some(symbol_id) = scoping.get_reference(accumulator.reference_id()).symbol_id() else {
            return;
        };
        if scoping.get_reference(accumulator_reference.reference_id()).symbol_id()
            != Some(symbol_id)
        {
            return;
        }

        // The accumulator must be a reassignable binding that starts out as an empty array:
        // `let result = []` or `var result = []`.
        let declaration_id = scoping.symbol_declaration(symbol_id);
        let AstKind::VariableDeclarator(declarator) = ctx.nodes().get_node(declaration_id).kind()
        else {
            return;
        };
        let AstKind::VariableDeclaration(declaration) =
            ctx.nodes().parent_node(declaration_id).kind()
        else {
            return;
        };

        if !matches!(declaration.kind, VariableDeclarationKind::Let | VariableDeclarationKind::Var)
            || scoping.symbol_declarations(symbol_id).count() != 1
            || !declarator
                .init
                .as_ref()
                .is_some_and(|init| is_empty_array_expression(init.get_inner_expression()))
        {
            return;
        }

        // Note: upstream additionally skips bindings declared in the global scope. Oxc lints some
        // files as scripts, where such an accumulator is still quadratic, so the diagnostic is
        // kept.

        // The accumulator must be declared outside of the loop that reassigns it, otherwise it is
        // recreated on every iteration and nothing accumulates.
        for ancestor in ctx.nodes().ancestors(node.id()) {
            match ancestor.kind() {
                // A function boundary means the assignment is not run as part of the loop body.
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => return,
                kind => {
                    let Some(body_span) = loop_body_span(kind) else {
                        continue;
                    };

                    if body_span.contains_inclusive(declarator.span) {
                        return;
                    }

                    ctx.diagnostic(no_array_concat_in_loop_diagnostic(property_span));
                    return;
                }
            }
        }
    }
}

/// The span of a loop statement's body.
fn loop_body_span(kind: AstKind<'_>) -> Option<Span> {
    match kind {
        AstKind::ForStatement(statement) => Some(statement.body.span()),
        AstKind::ForInStatement(statement) => Some(statement.body.span()),
        AstKind::ForOfStatement(statement) => Some(statement.body.span()),
        AstKind::WhileStatement(statement) => Some(statement.body.span()),
        AstKind::DoWhileStatement(statement) => Some(statement.body.span()),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let result = [];
            result = result.concat(chunk);",
        "let result = [];
            for (const chunk of chunks) {
                result = other.concat(chunk);
            }",
        "let result = [];
            for (const chunk of chunks) {
                other = result.concat(chunk);
            }",
        "let result = [initial];
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
        "let result;
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
        "const result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
        "const text = '';
            for (const part of parts) {
                text.concat(part);
            }",
        "let text = '';
            for (const part of parts) {
                text = text.concat(part);
            }",
        "let result = [];
            for (const chunk of chunks) {
                result = result?.concat(chunk);
            }",
        "let result = [];
            for (const chunk of chunks) {
                result = result['concat'](chunk);
            }",
        "let result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk).filter(Boolean);
            }",
        "let result = [];
            for (const chunk of chunks) {
                result = result.concat();
            }",
        "for (const chunk of chunks) {
                let result = [];
                result = result.concat(chunk);
            }",
        "let result = [];
            for (const chunk of chunks) {
                function append() {
                    result = result.concat(chunk);
                }
            }",
        "let result = [];
            const append = () => {
                result = result.concat(chunk);
            };
            for (const chunk of chunks) {
                append(chunk);
            }",
        "this.result = [];
            for (const chunk of chunks) {
                this.result = this.result.concat(chunk);
            }",
        "const result = chunks.reduce((result, chunk) => result.concat(chunk), []);",
    ];

    let fail = vec![
        "let result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
        "let result = [], other = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
                other.push(chunk);
            }",
        "let result = [];
            for (let index = 0; index < chunks.length; index++) {
                result = result.concat(chunks[index]);
            }",
        "let result = [];
            for (const index in chunks) {
                result = result.concat(chunks[index]);
            }",
        "let result = [];
            while (chunks.length > 0) {
                result = result.concat(chunks.pop());
            }",
        "let result = [];
            do {
                result = result.concat(getChunk());
            } while (hasMoreChunks());",
        "let result = [];
            for (let index = 0; index < chunks.length; result = result.concat(chunks[index++])) {}",
        "let result = [];
            for (const chunk of chunks) {
                result = (result.concat(chunk));
            }",
        "var result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
        "let result = [];
            for (const chunk of chunks) {
                (result) = (result).concat(chunk);
            }",
        "let result = [];
            for (const chunk of chunks) {
                result = result.concat(first, second);
            }",
        "let result = [];
            for (const chunk of chunks) {
                result = result.concat(...chunkGroups);
            }",
        "for (let result = []; condition; result = result.concat(chunk)) {}",
        "for (let result = []; condition;) {
                result = result.concat(chunk);
            }",
    ];

    Tester::new(NoArrayConcatInLoop::NAME, NoArrayConcatInLoop::PLUGIN, pass, fail)
        .test_and_snapshot();
}
