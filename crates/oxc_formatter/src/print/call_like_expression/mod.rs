mod arguments;

use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    formatter::prelude::*,
    utils::{
        call_expression::{callee_opener, is_test_call_expression},
        member_chain::MemberChain,
        statement_body::FormatBeforeOpener,
        tailwindcss::tailwind_context_for_callee,
    },
    write,
};
use arguments::{is_simple_module_import, is_verbatim_multiline_template_sole_arg};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'_, 'a, CallExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let callee = self.callee();
        let type_arguments = self.type_arguments();
        let arguments = self.arguments();

        // For nested non-Tailwind calls inside a Tailwind context, hide the context
        // to prevent sorting strings and templates inside the nested call's arguments.
        // (e.g., `classNames("a", x.includes("\n") ? "b" : "c")` - don't sort "\n")
        // A Tailwind call pushes its own context, around its arguments only
        let hides_tailwind_ctx = f.context().tailwind_context().is_some()
            && tailwind_context_for_callee(&self.callee, f).is_none();
        if hides_tailwind_ctx {
            f.context_mut().hide_tailwind_context();
        }

        // NOTE: Read before the arguments push the Tailwind context,
        // so a Tailwind call with a multiline template it collapses stays out of `MemberChain`,
        // which would break the chain on the template's line breaks in the source
        if callee.is_member_expression()
            && !is_verbatim_multiline_template_sole_arg(arguments.as_slice(), Some(self), f)
            && !is_simple_module_import(&self.arguments(), f.comments())
            && !is_test_call_expression(self, f.comments())
        {
            MemberChain::from_call_expression(self, f).fmt(f);
        } else {
            let format_inner = format_with(|f| {
                write!(
                    f,
                    [
                        FormatBeforeOpener(&callee, callee_opener(self)),
                        self.optional.then_some("?."),
                        type_arguments
                    ]
                );
                write!(f, arguments);
            });
            if matches!(callee.as_ref(), Expression::CallExpression(_)) {
                write!(f, [group(&format_inner)]);
            } else {
                write!(f, [format_inner]);
            }
        }

        if hides_tailwind_ctx {
            f.context_mut().pop_tailwind_context();
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'_, 'a, NewExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, ["new", space()]);
        if let Some(type_arguments) = self.type_arguments() {
            write!(f, [FormatBeforeOpener(&self.callee(), b'<'), type_arguments]);
        } else {
            write!(f, [self.callee()]);
        }
        write!(f, self.arguments());
    }
}

impl<'a> FormatWrite<'a> for AstNode<'_, 'a, ImportExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, ["import"]);
        if let Some(phase) = &self.phase() {
            write!(f, [".", phase.as_str()]);
        }

        // Use the same logic as CallExpression arguments formatting
        let arguments = self.arguments(f.allocator());
        write!(f, arguments);
    }
}
