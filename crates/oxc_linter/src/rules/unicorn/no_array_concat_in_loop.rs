use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentTarget, BindingPattern, CallExpression, Expression,
        VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::AssignmentOperator;

use crate::{
    AstNode,
    ast_util::{
        get_declaration_of_variable, get_symbol_id_of_variable, iter_outer_expressions,
        variable_declaration_kind,
    },
    context::LintContext,
    rule::Rule,
};

fn no_array_concat_in_loop_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `Array#concat()` to accumulate an array in a loop.")
        .with_help("Use `Array#push()` or collect the chunks and flatten them after the loop.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayConcatInLoop;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows accumulating an array with `Array#concat()` inside a loop.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#concat()` creates a new array and copies the existing accumulator on
    /// every iteration. As the accumulator grows, this can turn a linear operation
    /// into one with quadratic time and allocation costs.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let result = [];
    /// for (const chunk of chunks) {
    ///     result = result.concat(chunk);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const result = [];
    /// for (const chunk of chunks) {
    ///     result.push(...chunk);
    /// }
    /// ```
    NoArrayConcatInLoop,
    unicorn,
    perf,
    version = "next",
    short_description = "Disallows array accumulation with `Array#concat()` in loops.",
);

impl Rule for NoArrayConcatInLoop {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assignment) = node.kind() else {
            return;
        };

        if let Some(span) = array_concat_in_loop_property_span(assignment, node, ctx) {
            ctx.diagnostic(no_array_concat_in_loop_diagnostic(span));
        }
    }
}

fn array_concat_in_loop_property_span<'a>(
    assignment: &AssignmentExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<Span> {
    if assignment.operator != AssignmentOperator::Assign {
        return None;
    }

    let AssignmentTarget::AssignmentTargetIdentifier(left) = &assignment.left else {
        return None;
    };
    let Expression::CallExpression(call) = assignment.right.get_inner_expression() else {
        return None;
    };
    if call.optional || call.arguments.is_empty() {
        return None;
    }

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    if member.optional || member.property.name != "concat" {
        return None;
    }
    let Expression::Identifier(receiver) = member.object.get_inner_expression() else {
        return None;
    };

    let Some(symbol_id) = get_symbol_id_of_variable(left, ctx.semantic()) else {
        return None;
    };
    if get_symbol_id_of_variable(receiver, ctx.semantic()) != Some(symbol_id)
        || !ctx.scoping().symbol_redeclarations(symbol_id).is_empty()
        || (ctx.source_type().is_script()
            && ctx.scoping().symbol_scope_id(symbol_id) == ctx.scoping().root_scope_id())
    {
        return None;
    }

    let Some(declaration) = get_declaration_of_variable(left, ctx.semantic()) else {
        return None;
    };
    let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
        return None;
    };
    let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
        return None;
    };
    if binding.symbol_id() != symbol_id
        || !matches!(
            variable_declaration_kind(declarator, ctx),
            VariableDeclarationKind::Let | VariableDeclarationKind::Var
        )
    {
        return None;
    }

    let Some(Expression::ArrayExpression(initializer)) =
        declarator.init.as_ref().map(Expression::get_inner_expression)
    else {
        return None;
    };
    if !initializer.elements.is_empty() {
        return None;
    }

    let Some(loop_body_span) = nearest_loop_body_span(node, ctx) else {
        return None;
    };
    if loop_body_span.contains_inclusive(declarator.span) {
        return None;
    }

    Some(member.property.span)
}

pub(super) fn is_array_concat_in_loop_call<'a>(
    node: &AstNode<'a>,
    call: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let Some(AstKind::AssignmentExpression(assignment)) =
        iter_outer_expressions(ctx.nodes(), node.id()).next()
    else {
        return false;
    };
    let Expression::CallExpression(right_call) = assignment.right.get_inner_expression() else {
        return false;
    };
    if right_call.span != call.span {
        return false;
    }

    array_concat_in_loop_property_span(assignment, node, ctx).is_some()
}

fn nearest_loop_body_span(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<Span> {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        let kind = ancestor.kind();
        if kind.is_function_like() {
            return None;
        }

        let body_span = match kind {
            AstKind::ForStatement(statement) => Some(statement.body.span()),
            AstKind::ForInStatement(statement) => Some(statement.body.span()),
            AstKind::ForOfStatement(statement) => Some(statement.body.span()),
            AstKind::WhileStatement(statement) => Some(statement.body.span()),
            AstKind::DoWhileStatement(statement) => Some(statement.body.span()),
            _ => None,
        };
        if body_span.is_some() {
            return body_span;
        }
    }

    None
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
        "var result = [];
            var result;
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
        "let result = [];
            for (const chunk of chunks) {
                result += result.concat(chunk);
            }",
    ];

    let fail = vec![
        "let result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
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
        .change_rule_path_extension("mjs")
        .test_and_snapshot();
}

#[test]
fn test_script_scope() {
    use crate::tester::Tester;

    let pass = vec![
        "let result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
        "var result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }",
    ];

    let fail = vec![
        "function collect(chunks) {
            let result = [];
            for (const chunk of chunks) {
                result = result.concat(chunk);
            }
        }",
    ];

    Tester::new(NoArrayConcatInLoop::NAME, NoArrayConcatInLoop::PLUGIN, pass, fail)
        .change_rule_path_extension("js")
        .with_snapshot_suffix("script")
        .test_and_snapshot();
}

#[test]
fn test_typescript() {
    use crate::tester::Tester;

    let pass = vec![
        "export {};
            let result = ['initial'] as string[];
            for (const chunk of chunks) {
                result = (result as string[]).concat(chunk);
            }",
    ];

    let fail = vec![
        "export {};
            let result = [] as string[];
            for (const chunk of chunks) {
                result = (result as string[]).concat(chunk);
            }",
        "export {};
            let result = [] satisfies string[];
            for (const chunk of chunks) {
                result = result!.concat(chunk);
            }",
        "export {};
            let result = <string[]>[];
            for (const chunk of chunks) {
                result = (<string[]>result).concat(chunk);
            }",
    ];

    Tester::new(NoArrayConcatInLoop::NAME, NoArrayConcatInLoop::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("typescript")
        .test_and_snapshot();
}
