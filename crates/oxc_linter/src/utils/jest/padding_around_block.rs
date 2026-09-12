use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, IdentifierReference, Statement, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode,
        parse_general_jest_fn_call,
    },
};

#[derive(Clone, Copy)]
pub enum JestPaddingTarget {
    Hook(&'static str),
    Test,
}

#[derive(Clone, Copy)]
pub struct JestPaddingConfig<'a> {
    targets: &'a [JestPaddingTarget],
}

impl<'a> JestPaddingConfig<'a> {
    pub const fn new(targets: &'a [JestPaddingTarget]) -> Self {
        Self { targets }
    }

    fn matches(self, jest_fn_call: &ParsedGeneralJestFnCall) -> bool {
        self.targets.iter().any(|target| target.matches(jest_fn_call))
    }
}

impl JestPaddingTarget {
    fn matches(self, jest_fn_call: &ParsedGeneralJestFnCall) -> bool {
        match self {
            Self::Hook(name) => {
                jest_fn_call.kind == JestFnKind::General(JestGeneralFnKind::Hook)
                    && jest_fn_call.name == name
            }
            Self::Test => jest_fn_call.kind == JestFnKind::General(JestGeneralFnKind::Test),
        }
    }
}

#[derive(Clone, Copy)]
enum PaddingDirection {
    Before,
    After,
}

struct StatementsAroundNode<'a> {
    prev: Option<&'a Statement<'a>>,
    current: &'a Statement<'a>,
    next: Option<&'a Statement<'a>>,
}

impl<'a> StatementsAroundNode<'a> {
    fn into_parts(
        self,
    ) -> (Option<&'a Statement<'a>>, &'a Statement<'a>, Option<&'a Statement<'a>>) {
        (self.prev, self.current, self.next)
    }
}

fn padding_around_jest_block_diagnostic(
    span: Span,
    name: &str,
    direction: PaddingDirection,
) -> OxcDiagnostic {
    let direction = match direction {
        PaddingDirection::Before => "before",
        PaddingDirection::After => "after",
    };
    OxcDiagnostic::warn(format!("Missing padding {direction} {name} block"))
        .with_help(format!("Make sure there is an empty new line {direction} the {name} block"))
        .with_label(span)
}

pub fn report_missing_padding_around_jest_block<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
    config: JestPaddingConfig<'_>,
) {
    let Some(statements) = get_statements_around_node(node, ctx) else {
        return;
    };
    let (prev_statement, current_statement, next_statement) = statements.into_parts();

    if let Some(prev_statement) = prev_statement {
        report_missing_padding_between_statements(
            prev_statement.span(),
            node.span(),
            Span::new(node.span().start, node.span().start),
            ctx,
            name,
            PaddingDirection::Before,
        );
    }

    if let Some(next_statement) = next_statement
        && !is_padding_target_statement(next_statement, ctx, config)
    {
        report_missing_padding_between_statements(
            current_statement.span(),
            next_statement.span(),
            Span::new(node.span().end, node.span().end),
            ctx,
            name,
            PaddingDirection::After,
        );
    }
}

fn report_missing_padding_between_statements(
    prev_statement_span: Span,
    next_statement_span: Span,
    report_span: Span,
    ctx: &LintContext,
    name: &str,
    direction: PaddingDirection,
) {
    let comments_range = ctx.comments_range(prev_statement_span.end..next_statement_span.start);
    let mut span_between_start = prev_statement_span.end;
    let mut span_between_end = next_statement_span.start;
    let mut next_attached_start = next_statement_span.start;
    for comment in comments_range.rev() {
        let comment_span = comment.span;
        let space_after = ctx.source_range(Span::new(comment_span.end, next_attached_start));
        if space_after.matches('\n').count() > 1 {
            break;
        }
        let space_before = ctx.source_range(Span::new(prev_statement_span.end, comment_span.start));
        if space_before.matches('\n').count() == 0 {
            span_between_start = comment_span.end;
            break;
        }
        span_between_end = comment_span.start;
        next_attached_start = comment_span.start;
    }

    let span_between = Span::new(span_between_start, span_between_end);
    let content = ctx.source_range(span_between);
    if content.matches('\n').count() < 2 {
        ctx.diagnostic_with_fix(
            padding_around_jest_block_diagnostic(report_span, name, direction),
            |fixer| {
                let whitespace_after_last_line =
                    content.rfind('\n').map_or("", |index| content.split_at(index + 1).1);
                fixer.replace(span_between, format!("\n\n{whitespace_after_last_line}"))
            },
        );
    }
}

fn is_padding_target_statement<'a>(
    statement: &Statement<'a>,
    ctx: &LintContext<'a>,
    config: JestPaddingConfig<'_>,
) -> bool {
    let Statement::ExpressionStatement(expr_stmt) = statement else {
        return false;
    };
    let Expression::CallExpression(call_expr) = &expr_stmt.expression else {
        return false;
    };
    let AstKind::CallExpression(call_expr) = ctx.nodes().get_node(call_expr.node_id()).kind()
    else {
        return false;
    };
    let Some(possible_jest_node) = get_possible_jest_node_for_call_expression(call_expr, ctx)
    else {
        return false;
    };
    let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, &possible_jest_node, ctx) else {
        return false;
    };

    config.matches(&jest_fn_call)
}

fn get_possible_jest_node_for_call_expression<'a, 'c>(
    call_expr: &'a CallExpression<'a>,
    ctx: &'c LintContext<'a>,
) -> Option<PossibleJestNode<'a, 'c>> {
    let ident = resolve_first_ident(&call_expr.callee)?;
    let reference_id = ident.reference_id.get()?;
    let reference = ctx.scoping().get_reference(reference_id);
    let original = if let Some(symbol_id) = reference.symbol_id() {
        if !ctx.scoping().symbol_flags(symbol_id).is_import() {
            return None;
        }
        let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
        let AstKind::ImportDeclaration(import_decl) = ctx.nodes().parent_kind(declaration_id)
        else {
            return None;
        };
        if !matches!(
            import_decl.source.value.as_str(),
            "@jest/globals" | "vitest" | "vite-plus/test" | "@effect/vitest"
        ) {
            return None;
        }
        let name = ctx.scoping().symbol_name(symbol_id);
        super::find_original_name(import_decl, name)
    } else {
        if !super::JEST_METHOD_NAMES.contains(&ident.name.as_str()) {
            return None;
        }
        None
    };

    Some(PossibleJestNode { node: ctx.nodes().get_node(call_expr.node_id()), original })
}

fn resolve_first_ident<'a>(expr: &'a Expression<'a>) -> Option<&'a IdentifierReference<'a>> {
    match expr {
        Expression::Identifier(ident) => Some(ident),
        match_member_expression!(Expression) => {
            resolve_first_ident(expr.to_member_expression().object())
        }
        Expression::CallExpression(call_expr) => resolve_first_ident(&call_expr.callee),
        Expression::TaggedTemplateExpression(tagged_expr) => resolve_first_ident(&tagged_expr.tag),
        _ => None,
    }
}

fn get_statements_around_node<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<StatementsAroundNode<'a>> {
    let scope_node = ctx.nodes().get_node(ctx.scoping().get_node_id(node.scope_id()));
    match scope_node.kind() {
        AstKind::Program(program) => {
            get_statements_around_node_from_statements(node, program.body.as_slice())
        }
        AstKind::ArrowFunctionExpression(arrow_func_expr) => {
            let body = arrow_func_expr.get_function_body()?;
            get_statements_around_node_from_statements(node, body.statements.as_slice())
        }
        AstKind::Function(function) => {
            let body = function.body.as_ref()?;
            get_statements_around_node_from_statements(node, body.statements.as_slice())
        }
        _ => None,
    }
}

fn get_statements_around_node_from_statements<'a>(
    node: &AstNode<'a>,
    statements: &'a [Statement<'a>],
) -> Option<StatementsAroundNode<'a>> {
    let index = statements.iter().position(|statement| {
        let statement_span = statement.span();
        statement_span.start <= node.span().start && node.span().end <= statement_span.end
    })?;

    Some(StatementsAroundNode {
        prev: index.checked_sub(1).map(|index| &statements[index]),
        current: &statements[index],
        next: statements.get(index + 1),
    })
}
