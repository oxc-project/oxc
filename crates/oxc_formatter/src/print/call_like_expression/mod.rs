mod arguments;

use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    formatter::{TailwindContextEntry, prelude::*},
    utils::{
        call_expression::{callee_opener, is_test_call_expression},
        member_chain::MemberChain,
        statement_body::FormatBeforeOpener,
        tailwindcss::is_tailwind_function_call,
    },
    write,
};
use arguments::{is_simple_module_import, is_verbatim_multiline_template_sole_arg};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, CallExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let callee = self.callee();
        let type_arguments = self.type_arguments();
        let arguments = self.arguments();

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

        if matches!(
            callee.as_ref(),
            Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_)
        ) && !is_verbatim_multiline_template_sole_arg(arguments.as_slice(), Some(self), f)
            && !is_simple_module_import(self.arguments(), f.comments())
            && !is_test_call_expression(self, f.comments())
        {
            MemberChain::from_call_expression(self, f).fmt(f);
        } else {
            let format_inner = format_with(|f| {
                write!(
                    f,
                    [
                        FormatBeforeOpener(callee, callee_opener(self)),
                        self.optional.then_some("?."),
                        type_arguments
                    ]
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
        if let Some(type_arguments) = self.type_arguments() {
            write!(f, [FormatBeforeOpener(self.callee(), b'<'), type_arguments]);
        } else {
            write!(f, [self.callee()]);
        }
        write!(f, self.arguments());
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
