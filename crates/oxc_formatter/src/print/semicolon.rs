use oxc_ast::ast::{Comment, Declaration, ExportDefaultDeclarationKind, Expression, Statement};
use oxc_formatter_core::{Buffer, Format};
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        JsFormatContext, JsFormatter, JsFormatterExt as _,
        trivia::{FormatTrailingComments, format_leading_comments},
    },
    options::Semicolons,
    utils::{
        format_node_without_trailing_comments::format_content_without_comments_after,
        statement_span, suppressed::write_suppressed_content, typecast::cast_target_end,
    },
    write,
};

use super::{
    function::function_content_end, import_declaration::module_source_end,
    write_leading_comments_with_asi_guard,
};

pub struct OptionalSemicolon;

impl<'a> Format<'a, JsFormatContext<'a>> for OptionalSemicolon {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match f.options().semicolons {
            Semicolons::Always => write!(f, ";"),
            Semicolons::AsNeeded => (),
        }
    }
}

/// The Prettier >= 3.9 rule shared by every semicolon-terminated context:
/// same-line trailing comments between the content end and the source `;`
/// are printed behind the semicolon (`foo = 1 /* c */;` -> `foo = 1; /* c */`).
///
/// Returns the comments to print after the semicolon,
/// or `None` when the rule does not apply and the content prints as-is:
/// - no comment between the content and the end of the node (the common case),
/// - no actual `;` and no dropped `)` in the source (ASI):
///   there is nothing to move the comments across;
///   a closing paren in the range counts because those parens are dropped
///   (or re-printed on the statement's own terms),
///   so the comment would trail the whole statement on the next format anyway (prettier#19930),
/// - a trailing suppression comment (`content /* oxfmt-ignore */;`)
///   must stay visible to the content so it preserves its original text.
///
/// `Some` may hold an empty slice (e.g. own-line comments only); the caller
/// still hides the comments from the content and a later pass prints them.
pub fn trailing_comments_to_move_behind_semicolon<'a>(
    f: &JsFormatter<'_, 'a>,
    content_end: u32,
    node_end: u32,
) -> Option<&'a [Comment]> {
    let comments = f.context().comments();
    if !comments.has_comment_in_range(content_end, node_end)
        || !comments.has_semicolon_or_closing_paren_in_range(content_end, node_end)
    {
        return None;
    }
    let trailing_comments = comments.end_of_line_comments_after(content_end);
    if trailing_comments.iter().any(|comment| comments.is_suppression_comment(comment)) {
        return None;
    }
    Some(trailing_comments)
}

/// Whether a trailing comment sitting right before the closing source paren of
/// the rightmost sub-expression stays inside the re-printed parentheses,
/// instead of moving behind the semicolon.
///
/// Mirrors Prettier's `handleParenthesizedExpressionTrailingComment` (prettier#19263):
/// a parenthesized sequence/assignment keeps the comment when it is a variable declarator initializer,
/// a `return` argument, an arrow expression body, or an assignment's right-hand side.
/// Positions where the parentheses survive in the output. (Notably not `throw` and not a plain expression statement.)
/// The sequence/assignment itself prints the comment before its closing paren.
///
/// `gated` is `true` when `expr` itself sits in one of those positions.
///
/// This is the statement side of the table: it promises not to hide/move the comment,
/// [`write_trailing_comments_inside_parens`] (the expression side) actually prints it.
/// [`arrow_body_keeps_trailing_comment_inside_parens`] encodes the arrow-body slice of this table (plus JSX)
/// for the chain-leaf walk. Change all three together: if they drift, the comment silently lands elsewhere
/// (no assert catches it), so pin every position change in a fixture.
pub fn keeps_trailing_comment_inside_parens(expr: &Expression<'_>, gated: bool) -> bool {
    match expr {
        Expression::SequenceExpression(_) => gated,
        Expression::AssignmentExpression(assignment) => {
            gated
                || match &assignment.right {
                    Expression::SequenceExpression(_) => true,
                    right => keeps_trailing_comment_inside_parens(right, false),
                }
        }
        Expression::ArrowFunctionExpression(arrow) => arrow
            .get_expression()
            .is_some_and(|body| keeps_trailing_comment_inside_parens(body, true)),
        _ => false,
    }
}

/// The opposite outcome of [`keeps_trailing_comment_inside_parens`]:
/// where that table says the parentheses do NOT survive, the semicolon directly follows the content,
/// so trailing comments move behind it even from inside the dropped parentheses.
/// Those parentheses are excluded from the chain's rightmost leaf's span but included in the chain's,
/// so the leaf's end bounds the content (prettier#19893):
///
/// ```js
/// assigned = (a = c /* c */);
/// // ->
/// assigned = a = c; /* c */
/// ```
///
/// Bounds the content at every semicolon-terminated content site
/// (expression statements, export defaults, variable declarations, class property values, return arguments).
/// The chain also passes through arrow expression bodies (prettier#19930 family);
/// bodies matching [`arrow_body_keeps_trailing_comment_inside_parens`] stop the walk.
///
/// A JSDoc cast target leaf ends past its cast parens (`cast_target_end`), like the keeps above;
/// not via `end_including_source_parens`: the walk has no `node_end` bound,
/// and the cast's matching `)` is counted, not the last one in a window.
pub fn assignment_chain_leaf_end(expr: &Expression<'_>, f: &JsFormatter<'_, '_>) -> u32 {
    let mut leaf = expr;
    loop {
        match leaf {
            Expression::AssignmentExpression(assignment) => leaf = &assignment.right,
            Expression::ArrowFunctionExpression(arrow) => match arrow.get_expression() {
                Some(body) if !arrow_body_keeps_trailing_comment_inside_parens(body) => {
                    leaf = body;
                }
                _ => break,
            },
            _ => break,
        }
    }
    cast_target_end(leaf.span(), f).unwrap_or(leaf.span().end)
}

/// Content end for a semicolon-terminated expression site, pairing the two functions above:
/// past the closing source paren where the sub-expression keeps the comment inside, the chain leaf otherwise.
///
/// `paren_scan_start` bounds the paren scan on the keeps side
/// (usually the expression end; a declaration passes its declarations end).
pub fn semicolon_terminated_expression_content_end(
    f: &JsFormatter<'_, '_>,
    expr: &Expression<'_>,
    paren_scan_start: u32,
    node_end: u32,
    gated: bool,
) -> u32 {
    if keeps_trailing_comment_inside_parens(expr, gated) {
        f.context().comments().end_including_source_parens(paren_scan_start, node_end)
    } else {
        assignment_chain_leaf_end(expr, f)
    }
}

/// Whether an arrow expression body's printer re-adds the parentheses
/// AND keeps a trailing comment inside them, stopping the [`assignment_chain_leaf_end`] walk:
///
/// - sequence/assignment: the [`keeps_trailing_comment_inside_parens`] table
///   (`write_trailing_comments_inside_parens` prints the comment inside)
/// - JSX: the multiline form re-adds the parens with the `)` on its own line,
///   so moving the comment would cross it and a line boundary;
///   the flat form drops them (group-fit, unknowable here), the pinned known limitation
///
/// NOT conditional: its parens are formatter-owned (`should_add_parens`) and
/// often absent (object leftmost, broken groups), and its `;` stays on the content's line,
/// so its comments move uniformly.
fn arrow_body_keeps_trailing_comment_inside_parens(body: &Expression<'_>) -> bool {
    matches!(
        body,
        Expression::AssignmentExpression(_)
            | Expression::SequenceExpression(_)
            | Expression::JSXElement(_)
            | Expression::JSXFragment(_)
    )
}

/// The printing half of [`keeps_trailing_comment_inside_parens`]:
/// sequence/assignment call this at the end of their `write` to print the comments
/// sitting right before their closing source paren inside the parentheses.
///
/// `is_sequence` distinguishes the one asymmetric position:
/// only a sequence keeps its parens as an assignment's right-hand side.
pub fn write_trailing_comments_inside_parens<'a>(
    f: &mut JsFormatter<'_, 'a>,
    parent: &AstNodes<'a>,
    node_end: u32,
    is_sequence: bool,
) {
    let parens_survive = match parent {
        AstNodes::ArrowFunctionExpression(_)
        | AstNodes::VariableDeclarator(_)
        | AstNodes::ReturnStatement(_) => true,
        AstNodes::AssignmentExpression(_) => is_sequence,
        _ => false,
    };
    if parens_survive {
        write_comments_before_closing_paren(f, node_end);
    }
}

/// Prints the comments sitting right before the closing source paren after `end`,
/// inside the parentheses, for a node that re-prints them.
pub fn write_comments_before_closing_paren(f: &mut JsFormatter<'_, '_>, end: u32) {
    if let Some(comments) = f.context().comments().comments_before_closing_paren(end) {
        write!(f, FormatTrailingComments::Comments(comments));
    }
}

/// Formats `content` followed by an `OptionalSemicolon`,
/// printing the content's trailing comments after the semicolon like Prettier:
/// `foo = 1 /* c */;` -> `foo = 1; /* c */`
///
/// In these statement-terminator contexts the semicolon directly follows the content in the output
/// (source parentheses like `(a = c /* c */);` are not re-printed around the end),
/// so the comments move even from inside them.
/// Return/throw arguments differ: their parentheses survive in the output,
/// so comments inside them stay there (see `ReturnAndThrowStatement`).
///
/// Own-line comments before the semicolon are left for the next node's leading-comments pass,
/// also like Prettier.
pub struct FormatContentWithSemicolon<'b, T> {
    content: &'b T,
    /// End position of the content; comments between it and the semicolon
    /// move behind the semicolon.
    content_end: u32,
    /// End position of the whole node, after the semicolon if it exists.
    node_end: u32,
}

impl<'b, T> FormatContentWithSemicolon<'b, T> {
    pub fn new(content: &'b T, content_end: u32, node_end: u32) -> Self {
        Self { content, content_end, node_end }
    }
}

impl<'a, T> Format<'a, JsFormatContext<'a>> for FormatContentWithSemicolon<'_, T>
where
    T: Format<'a, JsFormatContext<'a>>,
{
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let Some(trailing_comments) =
            trailing_comments_to_move_behind_semicolon(f, self.content_end, self.node_end)
        else {
            write!(f, [self.content, OptionalSemicolon]);
            return;
        };

        // Hide the trailing comments while formatting the content.
        // So they are not printed before the semicolon.
        format_content_without_comments_after(self.content, self.content_end, f);

        write!(f, [OptionalSemicolon, FormatTrailingComments::Comments(trailing_comments)]);
    }
}

/// Prints a statement when it is suppressed (`oxfmt-ignore` / `prettier-ignore`, leading or trailing comment)
/// and returns whether it was: its leading comments, then the source text up to its content end,
/// then the formatter's terminator per `semi` like every other statement (the token-class table in AGENTS.md).
/// A statement without a terminator of its own prints its whole span.
/// Any site printing a statement outside the generated `Statement` fmt (an `if` consequent before `else`)
/// must call this first, or the generic verbatim path prints the source `;` regardless of `semi`.
/// Same-line comments between the content end and a later-line source `;` stay on the content's line,
/// own-line ones are left for the next node's leading pass;
/// the caller (the generated `Statement` fmt) prints the trailing comments.
pub fn write_suppressed_statement<'a>(
    stmt: &AstNode<'a, Statement<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    let span = statement_span(stmt.as_ref());
    if !f.comments().is_node_suppressed(span, || suppressed_statement_content_end(stmt.as_ref(), f))
    {
        return false;
    }
    match stmt.as_ast_nodes() {
        // The `semi: false` ASI guard goes before a leading type cast comment, verbatim or not:
        // Prettier's `printIgnored` puts it after and breaks the cast (DIVERGENCES.md#suppressed-cast-comment-asi-guard)
        AstNodes::ExpressionStatement(stmt) => write_leading_comments_with_asi_guard(stmt, true, f),
        _ => format_leading_comments(span).fmt(f),
    }
    if write_suppressed_content(span, suppressed_statement_content_end(stmt.as_ref(), f), f) {
        OptionalSemicolon.fmt(f);
    }
    true
}

#[expect(clippy::cast_possible_truncation)]
const RETURN_KEYWORD_LEN: u32 = "return".len() as u32;
#[expect(clippy::cast_possible_truncation)]
const DEBUGGER_KEYWORD_LEN: u32 = "debugger".len() as u32;
#[expect(clippy::cast_possible_truncation)]
const BREAK_KEYWORD_LEN: u32 = "break".len() as u32;
#[expect(clippy::cast_possible_truncation)]
const CONTINUE_KEYWORD_LEN: u32 = "continue".len() as u32;

/// Where a suppressed statement's content ends and the formatter's terminator takes over:
/// the last content token (extended over dropped source parens), the keyword for a bare `return`/`break`/`continue`/`debugger`.
/// `None` for a statement without a terminator of its own (a block, a declaration with a body, ...);
/// a statement ending in a body answers for its rightmost body.
/// Mirrors the `;`-printing sites of the reprint (`FormatContentWithSemicolon` / `OptionalSemicolon`): keep them in step.
/// Same terminator, NOT the same content end, and not to be unified:
/// the reprint's is the comment-move boundary (chain leaf, cast target),
/// this one is the token end past the source parens the reprint drops (`p = (q = 1)`).
pub fn suppressed_statement_content_end(
    stmt: &Statement<'_>,
    f: &JsFormatter<'_, '_>,
) -> Option<u32> {
    let span = stmt.span();
    let with_parens =
        |content_end: u32| f.comments().end_including_source_parens(content_end, span.end);
    match stmt {
        Statement::ExpressionStatement(s) => Some(with_parens(s.expression.span().end)),
        Statement::ReturnStatement(s) => {
            Some(s.argument.as_ref().map_or(span.start + RETURN_KEYWORD_LEN, |argument| {
                with_parens(argument.span().end)
            }))
        }
        Statement::ThrowStatement(s) => Some(with_parens(s.argument.span().end)),
        Statement::DoWhileStatement(s) => Some(with_parens(s.test.span().end)),
        Statement::DebuggerStatement(_) => Some(span.start + DEBUGGER_KEYWORD_LEN),
        Statement::BreakStatement(s) => {
            Some(s.label.as_ref().map_or(span.start + BREAK_KEYWORD_LEN, |label| label.span.end))
        }
        Statement::ContinueStatement(s) => {
            Some(s.label.as_ref().map_or(span.start + CONTINUE_KEYWORD_LEN, |label| label.span.end))
        }
        Statement::ImportDeclaration(s) => {
            Some(module_source_end(&s.source, s.with_clause.as_deref()))
        }
        Statement::ExportAllDeclaration(s) => {
            Some(module_source_end(&s.source, s.with_clause.as_deref()))
        }
        Statement::ExportFromDeclaration(s) => {
            Some(module_source_end(&s.source, s.with_clause.as_deref()))
        }
        Statement::ExportNamedDeclaration(s) => {
            // `export { a }`: the `}` after the last specifier (or the `{`)
            let from = s.specifiers.last().map_or(span.start, |specifier| specifier.span.end);
            Some(f.comments().position_after_character(from, b'}'))
        }
        Statement::ExportDefaultDeclaration(s) => match &s.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                function_content_end(function, f)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(_)
            | ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => None,
            expression => Some(with_parens(expression.span().end)),
        },
        Statement::ExportDeclaration(s) => declaration_content_end(&s.declaration, f),
        Statement::TSExportAssignment(s) => Some(with_parens(s.expression.span().end)),
        Statement::TSNamespaceExportDeclaration(s) => Some(s.id.span.end),
        Statement::IfStatement(s) => {
            suppressed_statement_content_end(s.alternate.as_ref().unwrap_or(&s.consequent), f)
        }
        Statement::WhileStatement(s) => suppressed_statement_content_end(&s.body, f),
        Statement::WithStatement(s) => suppressed_statement_content_end(&s.body, f),
        Statement::ForStatement(s) => suppressed_statement_content_end(&s.body, f),
        Statement::ForInStatement(s) => suppressed_statement_content_end(&s.body, f),
        Statement::ForOfStatement(s) => suppressed_statement_content_end(&s.body, f),
        Statement::LabeledStatement(s) => suppressed_statement_content_end(&s.body, f),
        _ => stmt.as_declaration().and_then(|declaration| declaration_content_end(declaration, f)),
    }
}

/// [`suppressed_statement_content_end`] for a declaration (also behind `export`):
/// the last declarator, a type alias's type, a bodyless (`declare`) function's signature, a bodyless `declare module "m"`.
fn declaration_content_end(declaration: &Declaration<'_>, f: &JsFormatter<'_, '_>) -> Option<u32> {
    match declaration {
        Declaration::VariableDeclaration(s) => {
            // `VariableDeclaration` always has at least one declarator
            let declarations_end = s.declarations.last().unwrap().span.end;
            Some(f.comments().end_including_source_parens(declarations_end, s.span.end))
        }
        Declaration::FunctionDeclaration(function) => function_content_end(function, f),
        Declaration::TSTypeAliasDeclaration(s) => Some(s.type_annotation.span().end),
        Declaration::TSImportEqualsDeclaration(s) => Some(s.module_reference.span().end),
        Declaration::TSExternalModuleDeclaration(s) if s.body.is_none() => Some(s.id.span.end),
        _ => None,
    }
}
