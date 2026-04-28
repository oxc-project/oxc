mod arguments;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{Formatter, TailwindContextEntry, prelude::*, trivia::FormatTrailingComments},
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
    fn write(&self, f: &mut Formatter<'_, 'a>) {
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
                // Preserve trailing comments of the callee in the following cases:
                // `call /**/()`
                // `call /**/<T>()`
                if self.type_arguments.is_some() {
                    write!(f, [callee]);
                } else {
                    write!(f, [FormatNodeWithoutTrailingComments(callee)]);

                    let character = if self.optional {
                        // For optional calls with arguments, preserve trailing comments
                        // between the `callee` and `?.` operator.
                        // `alert/* comment */?.('value')` → `alert /* comment */?.("value");`
                        Some(b'?')
                    } else if self.arguments.is_empty() {
                        // For empty argument calls, preserve trailing comments between
                        // the `callee` and `()`.
                        // `call/**/()` → `call /**/();`
                        Some(b'(')
                    } else {
                        None
                    };
                    if let Some(character) = character {
                        let callee_trailing_comments = f
                            .context()
                            .comments()
                            .comments_before_character(self.callee.span().end, character);
                        write!(f, FormatTrailingComments::Comments(callee_trailing_comments));
                    }
                }
                write!(f, [optional.then_some("?."), type_arguments]);

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
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["new", space(), self.callee(), self.type_arguments(), self.arguments()]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["import"]);
        if let Some(phase) = &self.phase() {
            write!(f, [".", phase.as_str()]);
        }

        // Use the same logic as CallExpression arguments formatting
        write!(f, self.to_arguments());
    }
}
