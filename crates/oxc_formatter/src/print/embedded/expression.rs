//! [`JsRoot::Expression`]: an expression in its host's delimiters (Vue `{{ a }}`, `:a="b"`).

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::GetSpan;
use oxc_syntax::{
    identifier::{is_identifier_name, is_white_space},
    keyword::is_reserved_keyword,
    line_terminator::is_line_terminator,
};

use crate::{
    ExpressionLayout, ExpressionRoot, JsRoot,
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        JsFormatContext, JsFormatter, TailwindContextEntry, comments::gap_segments, prelude::*,
    },
    utils::expression::ExpressionLeftSide,
    write,
};

use super::shape_error;

/// The code is the one expression of `program`, parsed in `(` and `)`.
pub(super) fn check(
    program: &Program<'_>,
    source: Range<usize>,
    root: ExpressionRoot,
    inlines_code: bool,
) -> Result<(), OxcDiagnostic> {
    // An expression that ends inside the source closes the wrapper's paren (`a) + (b`)
    let [Statement::ExpressionStatement(statement)] = program.body.as_slice() else {
        return Err(shape_error());
    };
    let span = statement.expression.span();
    if !program.directives.is_empty()
        || (span.start as usize) < source.start
        || span.end as usize > source.end
    {
        return Err(shape_error());
    }
    // Inlined as written, a top-level `a, b` reads as more than one value (Vue's `toDisplayString(a, b)` shows `a`):
    // the parens it prints in would change it, so it is left as written.
    // In the source's parens or the host's, it reads as one value, as printed
    if inlines_code
        && !root.in_parens
        && matches!(statement.expression, Expression::SequenceExpression(_))
        && skip_trivia(program, &source, source.start) == span.start as usize
    {
        return Err(OxcDiagnostic::error(
            "The code is a sequence expression; it is preserved as-is",
        ));
    }
    // Read as statements, it does not compile; the parens it prints in would make it
    if root.reads_as_statements
        && starts_declaration_as_statements(program, &statement.expression, source)
    {
        return Err(OxcDiagnostic::error(
            "The code starts a declaration or a block where the host reads statements; it is preserved as-is",
        ));
    }
    Ok(())
}

/// The host reads the code as statements, holding a `;` ([`ExpressionRoot::reads_as_statements`]),
/// and its leftmost node starts a declaration or a block there (`function`, `class`, `{`, `let [`),
/// which the expression is not (`function () { a(); }`).
fn starts_declaration_as_statements(
    program: &Program<'_>,
    expression: &Expression<'_>,
    source: Range<usize>,
) -> bool {
    if !program.source_text[source.clone()].contains(';') {
        return false;
    }
    let leftmost = ExpressionLeftSide::leftmost(expression);
    // In parens in the source (`(function () {})()`), it starts no statement
    if skip_trivia(program, &source, source.start) != leftmost.span().start as usize {
        return false;
    }
    match leftmost {
        Expression::FunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::ObjectExpression(_) => true,
        Expression::AssignmentExpression(assignment) => {
            matches!(assignment.left, AssignmentTarget::ObjectAssignmentTarget(_))
        }
        Expression::Identifier(identifier) if identifier.name == "let" => program.source_text
            [skip_trivia(program, &source, identifier.span.end as usize)..]
            .starts_with('['),
        _ => false,
    }
}

/// The position past the whitespace and comments at `position` in `source`;
/// a node there with a later start is in the source's parens (the AST keeps none).
#[expect(clippy::cast_possible_truncation)] // Offsets fit in `u32`, source length does
fn skip_trivia(program: &Program<'_>, source: &Range<usize>, position: usize) -> usize {
    gap_segments(&program.comments, position as u32, source.end as u32)
        .find_map(|(from, to)| {
            let gap = &program.source_text[from as usize..to as usize];
            let rest = gap.trim_start_matches(|c| is_white_space(c) || is_line_terminator(c));
            (!rest.is_empty()).then(|| to as usize - rest.len())
        })
        .unwrap_or(source.end)
}

/// [`super::FormatRoot`] for an expression of one name (`a`), which prints as written whatever the layout, so needs no parse.
/// No string, comment or sequence, so none of what the parse path checks or sets applies.
///
/// Only a name: about half of a Vue template's values, printed by no rule of another node.
/// Other shapes are few, or print by such a rule, which this would have to keep in step:
/// - A member (`a.b`): whether one breaks is for `print/member_expression.rs` to decide
/// - A string: its quotes and escapes are normalized, and its classes may be sorted
/// - A number (`0`) or `!a`: a percent each
pub struct FormatName<'a> {
    pub name: &'a str,
    layout: ExpressionLayout,
}

impl<'a> FormatName<'a> {
    /// `code`, trimmed, when `root` is an expression and the code a name, as the parser reads one.
    ///
    /// - A reserved word is a name only as a value (`true`, `false`, `null`, `this`)
    /// - Any other reserved word (`if`, and contextual ones like `let`) is left to the parse, which decides
    pub fn new(allocator: &'a Allocator, code: &str, root: JsRoot) -> Option<Self> {
        let JsRoot::Expression(root) = root else { return None };
        // Non-ASCII whitespace (NBSP, U+2028) stays, fails `is_identifier_name` and goes to the parse
        let code = code.trim_ascii();
        let is_name = is_identifier_name(code)
            && (!is_reserved_keyword(code) || matches!(code, "true" | "false" | "null" | "this"));
        is_name.then(|| Self { name: allocator.alloc_str(code), layout: root.layout })
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatName<'a> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        // A name never hugs, and has no comment
        write_in_layout(&text_without_whitespace(self.name), self.layout, false, false, f);
    }
}

/// Writes `expression`, the root of `program`, in the layout `root` names.
pub(super) fn write_expression<'a>(
    program: &AstNode<'_, 'a, Program<'a>>,
    expression: &'a Expression<'a>,
    root: ExpressionRoot,
    ends_in_line_comment: bool,
    f: &mut JsFormatter<'_, 'a>,
) {
    f.context_mut().set_expression_root(root);
    // Under the `Program`, not the `ExpressionStatement`: no statement rule applies to it
    let expression = AstNode::new(expression, AstNodes::Program(program));
    let tailwind = f
        .options()
        .sort_tailwindcss
        .as_ref()
        .filter(|_| root.tailwind_classes)
        .map(|options| TailwindContextEntry::new(options.preserve_whitespace));
    if let Some(context) = tailwind {
        f.context_mut().push_tailwind_context(context);
    }
    let hugs = match expression.as_ref() {
        Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => true,
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
            matches!(root.layout, ExpressionLayout::Hug { literals: true })
        }
        _ => false,
    };
    write_in_layout(&expression, root.layout, hugs, ends_in_line_comment, f);
    if tailwind.is_some() {
        f.context_mut().pop_tailwind_context();
    }
}

/// Writes `expression` in `layout`.
/// `hugs`: it hugs under `ExpressionLayout::Hug`; `ends_in_line_comment`: on its own lines unless `Bare`.
fn write_in_layout<'a>(
    expression: &impl Format<'a, JsFormatContext<'a>>,
    layout: ExpressionLayout,
    hugs: bool,
    ends_in_line_comment: bool,
    f: &mut JsFormatter<'_, 'a>,
) {
    match layout {
        ExpressionLayout::Bare => write!(f, expression),
        _ if ends_in_line_comment => write!(f, soft_block_indent(expression)),
        ExpressionLayout::Group => write!(f, group(expression)),
        ExpressionLayout::Hug { .. } if hugs => write!(f, group(expression)),
        ExpressionLayout::Hug { .. } => write!(f, soft_block_indent(expression)),
    }
}

/// The object at `span`, in an embedded expression its host ends at the first `}}` ([`ExpressionRoot::ends_at_double_brace`]),
/// prints right before another `}`: it prints in parens.
/// A template literal's substitution closes right after it (`${({a: 1})}`), whatever `bracketSpacing` says;
/// without bracket spacing, an object's does too (`{a: ({b: 1})}`, `{...({a: 1})}`).
///
/// Prettier parens a property's value only (`apps/oxfmt/DIVERGENCES.md#js-in-vue-object-before-double-brace`).
///
/// NOTE: Only an object is covered, the contract's known gap (`crates/oxc_formatter/AGENTS.md` "js-in-xxx").
pub fn object_prints_double_brace(
    span: Span,
    parent: &AstNodes<'_, '_>,
    f: &JsFormatter<'_, '_>,
) -> bool {
    f.context().expression_root().is_some_and(|root| root.ends_at_double_brace)
        && is_followed_by_right_bracket(span, parent, f.options().bracket_spacing.value())
}

/// The node at `span` prints right before a `}`: a template literal's substitution's,
/// or an object's without `bracket_spacing`.
fn is_followed_by_right_bracket(
    span: Span,
    parent: &AstNodes<'_, '_>,
    bracket_spacing: bool,
) -> bool {
    // `node` at `span` is the last of the object `parent`, whose `}` prints right after it without bracket spacing
    let ends_object = |parent: &AstNodes<'_, '_>, span: Span| {
        !bracket_spacing
            && matches!(parent, AstNodes::ObjectExpression(object)
                if object.properties.last().is_some_and(|last| last.span() == span))
    };
    let ends_parent = match parent {
        // One of its expressions, which its substitution's `}` closes
        AstNodes::TemplateLiteral(_) => return true,
        AstNodes::ObjectProperty(property) if property.value.span() == span => {
            return ends_object(property.parent(), property.span);
        }
        // Another argument (a conditional, a logical, an assignment) prints in parens there
        AstNodes::SpreadElement(spread)
            if matches!(
                spread.argument,
                Expression::ObjectExpression(_) | Expression::UnaryExpression(_)
            ) =>
        {
            return ends_object(spread.parent(), spread.span);
        }
        AstNodes::BinaryExpression(binary) => binary.right.span() == span,
        AstNodes::LogicalExpression(logical) => logical.right.span() == span,
        AstNodes::ConditionalExpression(conditional) => conditional.alternate.span() == span,
        AstNodes::UnaryExpression(_) => true,
        _ => false,
    };
    ends_parent && is_followed_by_right_bracket(parent.span(), parent.parent(), bracket_spacing)
}
