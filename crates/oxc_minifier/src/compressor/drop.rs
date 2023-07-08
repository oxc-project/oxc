//! Expression and statement removal

use oxc_hir::hir::{Expression, Statement};

use super::Compressor;

impl<'a> Compressor<'a> {
    pub(super) fn should_drop<'b>(&mut self, stmt: &'b Statement<'a>) -> bool {
        self.drop_debugger(stmt) || self.drop_console(stmt)
    }

    /// Drop `drop_debugger` statement.
    /// Enabled by `compress.drop_debugger`
    fn drop_debugger<'b>(&mut self, stmt: &'b Statement<'a>) -> bool {
        matches!(stmt, Statement::DebuggerStatement(_)) && self.options.drop_debugger
    }

    /// Drop `console.*` expressions.
    /// Enabled by `compress.drop_console
    fn drop_console<'b>(&mut self, stmt: &'b Statement<'a>) -> bool {
        if !self.options.drop_console {
            return false;
        }
        let Statement::ExpressionStatement(expr) = stmt else { return false };
        let Expression::CallExpression(call_expr) = &expr.expression else { return false };
        let Expression::MemberExpression(member_expr) = &call_expr.callee else { return false };
        let obj = member_expr.object();
        let Some(ident) = obj.get_identifier_reference() else { return false };
        ident.name == "console"
    }
}
