mod arguments;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{TailwindContextEntry, prelude::*, trivia::FormatTrailingComments},
    print::arrow_function_expression::is_multiline_template_starting_on_same_line,
    utils::{
        call_expression::is_test_call_expression,
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        member_chain::MemberChain, tailwindcss::is_tailwind_function_call,
    },
    write,
};
use arguments::is_simple_module_import;

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, CallExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let callee = self.callee();
        let type_arguments = self.type_arguments();
        let arguments = self.arguments();
        let optional = self.optional();

        // Check if this is a Tailwind function call (e.g., clsx, cn, tw)
        let is_tailwind_call = f
            .options()
            .sort_tailwindcss
            .as_ref()
            .is_some_and(|opts| is_tailwind_function_call(&self.callee, opts));

        // For nested non-Tailwind calls inside a Tailwind context, disable sorting
        // to prevent sorting strings inside the nested call's arguments.
        // (e.g., `classNames("a", x.includes("\n") ? "b" : "c")` - don't sort "\n")
        let was_disabled =
            if !is_tailwind_call && let Some(ctx) = f.context_mut().tailwind_context_mut() {
                let was = ctx.disabled;
                ctx.disabled = true;
                Some(was)
            } else {
                None
            };

        let is_template_literal_single_arg = arguments.len() == 1
            && arguments.first().unwrap().as_expression().is_some_and(|expr| {
                is_multiline_template_starting_on_same_line(expr, f.source_text())
            });

        if !is_template_literal_single_arg
            && matches!(
                callee.as_ref(),
                Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_)
            )
            && !is_simple_module_import(self.arguments(), f.comments())
            && !is_test_call_expression(self)
        {
            MemberChain::from_call_expression(self, f).fmt(f);
        } else {
            let format_inner = format_with(|f| {
                write_callee_and_type_arguments(
                    callee,
                    type_arguments,
                    optional,
                    self.arguments.first().map(|argument| argument.span().start),
                    self.span.end,
                    f,
                );

                // If this IS a Tailwind function call, push the Tailwind context
                let tailwind_ctx_to_push = if is_tailwind_call {
                    f.options()
                        .sort_tailwindcss
                        .as_ref()
                        .map(|opts| TailwindContextEntry::new(opts.preserve_whitespace))
                } else {
                    None
                };

                // Push Tailwind context before formatting arguments
                if let Some(ctx) = tailwind_ctx_to_push {
                    f.context_mut().push_tailwind_context(ctx);
                }

                write!(f, arguments);

                // Pop Tailwind context after formatting
                if tailwind_ctx_to_push.is_some() {
                    f.context_mut().pop_tailwind_context();
                }
            });
            if matches!(callee.as_ref(), Expression::CallExpression(_)) {
                write!(f, [group(&format_inner)]);
            } else {
                write!(f, [format_inner]);
            }
        }

        // Restore the previous disabled state
        if let Some(was) = was_disabled
            && let Some(ctx) = f.context_mut().tailwind_context_mut()
        {
            ctx.disabled = was;
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, NewExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, ["new", space()]);
        write_callee_and_type_arguments(
            self.callee(),
            self.type_arguments(),
            false,
            self.arguments.first().map(|argument| argument.span().start),
            self.span.end,
            f,
        );
        write!(f, self.arguments());
    }
}

/// Prints a call-like expression's callee and type arguments, each followed by a `line_suffix_boundary()`
/// (Prettier's `printCallee` and `typeArgumentsDoc`):
/// a same-line line comment trailing either flushes right there instead of riding to the end of the statement
/// (`foo // c` + break + `<T>()`, `foo<T> // c` + break + `(1)`),
/// and an own-line comment before an empty `()` keeps its own line (`foo<T>` + `// c` + `()`).
///
/// Comments between the callee and its `(` attach like Prettier's attachment defaults
/// (`handleCallExpressionComments` claims only the ones inside the parens):
/// - with arguments, a same-line line comment trails the callee (the boundary flushes it: `foo // c` + break + `(1)`),
///   everything else leads the first argument (`foo /* c */(1)` -> `foo(/* c */ 1)`, Prettier's fixpoint)
/// - before an empty `()` or an optional `?.`, everything trails the callee
///   (`call /* c */()`, `alert /* c */?.("value")`, `foo` + `// c` + `()`)
///
/// With type arguments the callee's generic trailing pass already lands there
/// (a same-line comment trails, an own-line one leads the type arguments, glued like Prettier: `foo// c` + break + `<T>()`).
fn write_callee_and_type_arguments<'a>(
    callee: &AstNode<'a, Expression<'a>>,
    type_arguments: Option<&AstNode<'a, TSTypeParameterInstantiation<'a>>>,
    optional: bool,
    arguments_start: Option<u32>,
    node_end: u32,
    f: &mut JsFormatter<'_, 'a>,
) {
    let callee_start = callee.span().start;
    // The common case: no pending comment up to the arguments (or the node end), nothing to attach,
    // no `line_suffix` a boundary could flush; the callee's own trailing pass finds nothing either
    if !f.comments().has_comment_before(arguments_start.unwrap_or(node_end)) {
        write!(f, [callee, optional.then_some("?."), type_arguments]);
        return;
    }
    let has_arguments = arguments_start.is_some();
    if type_arguments.is_some() {
        write!(f, callee);
        write_boundary_after(callee_start, f);
        write!(f, [optional.then_some("?."), type_arguments]);
        write_boundary_after(callee_start, f);
        return;
    }

    write!(f, [FormatNodeWithoutTrailingComments(callee)]);
    let callee_end = callee.span().end;
    // The lexical scan for the token is unbounded; a `new X` without parens has no `(` of its own,
    // so the node end fences out everything past it
    let comments_before_token = |f: &JsFormatter<'_, 'a>, token: u8| {
        let comments = f.context().comments().comments_before_character(callee_end, token);
        &comments[..comments.iter().take_while(|comment| comment.span.end <= node_end).count()]
    };
    if optional || !has_arguments {
        let comments = comments_before_token(f, if optional { b'?' } else { b'(' });
        write!(f, FormatTrailingComments::Comments(comments));
    } else {
        let comments = comments_before_token(f, b'(');
        let same_line_count = comments.iter().take_while(|c| !c.preceded_by_newline()).count();
        let same_line = &comments[..same_line_count];
        if same_line.last().is_some_and(|comment| comment.is_line()) {
            write!(f, FormatTrailingComments::Comments(same_line));
        }
    }
    write_boundary_after(callee_start, f);
    write!(f, optional.then_some("?."));
}

/// The `line_suffix_boundary()` of [`write_callee_and_type_arguments`], emitted only when it has
/// something to flush: a comment printed since `start` may be riding a `line_suffix`
/// (a same-line line comment, or an own-line comment rendered trailing).
/// Skipped otherwise to keep the IR lean, calls being the most common expression.
fn write_boundary_after(start: u32, f: &mut JsFormatter<'_, '_>) {
    if f.comments().printed_comments().last().is_some_and(|comment| comment.span.start >= start) {
        write!(f, line_suffix_boundary());
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, ["import"]);
        if let Some(phase) = &self.phase() {
            write!(f, [".", phase.as_str()]);
        }

        // Use the same logic as CallExpression arguments formatting
        write!(f, self.to_arguments());
    }
}
