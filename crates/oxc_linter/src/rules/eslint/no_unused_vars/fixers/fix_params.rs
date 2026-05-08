use oxc_ast::ast::FormalParameter;

use super::{NoUnusedVars, Symbol};
use crate::fixer::{RuleFix, RuleFixer};

impl NoUnusedVars {
    /// Rename an unused function parameter to match `argsIgnorePattern`.
    ///
    /// Only simple identifier parameters are handled in this phase. Default
    /// parameters and destructuring parameters are left for follow-up work.
    pub(in super::super) fn rename_unused_function_parameter<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        param: &FormalParameter<'a>,
    ) -> RuleFix {
        if param.initializer.is_some()
            || param.has_modifier()
            || param.pattern.is_destructuring_pattern()
            || param.pattern.get_binding_identifier().is_none()
        {
            return fixer.noop();
        }

        if let Some(new_name) = self.get_unused_arg_name(symbol) {
            return symbol.rename_with_fixer(fixer, &new_name).dangerously();
        }

        fixer.noop()
    }
}
