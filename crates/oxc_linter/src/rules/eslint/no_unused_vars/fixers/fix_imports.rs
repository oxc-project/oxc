use oxc_ast::ast::ImportDeclaration;
use oxc_span::GetSpan;

use super::{count_whitespace_or_commas, NoUnusedVars, Symbol};
use crate::fixer::{RuleFix, RuleFixer};

impl NoUnusedVars {
    #[allow(clippy::unused_self)]
    pub(in super::super) fn remove_unused_import_declaration<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        import: &ImportDeclaration<'a>,
    ) -> RuleFix<'a> {
        let specifiers = import
                .specifiers
                .as_ref()
                .expect("Found an unused variable in an ImportDeclaration with no specifiers. This should be impossible.");

        debug_assert!(
            !specifiers.is_empty(),
            "Found an unused variable in an ImportDeclaration with no specifiers. This should be impossible."
        );

        if specifiers.len() == 1 {
            return fixer.delete(import).dangerously();
        }
        let span = specifiers
            .iter()
            .find(|specifier| symbol == specifier)
            .map_or_else(|| symbol.span(), GetSpan::span);
        let text_after = fixer.source_text()[(span.end as usize)..].chars();
        let span = span.expand_right(count_whitespace_or_commas(text_after));

        fixer.delete_range(span).dangerously()
    }
}
