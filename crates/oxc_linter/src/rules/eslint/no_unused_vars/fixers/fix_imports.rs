use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier};
use oxc_span::{GetSpan, Span};

use super::{NoUnusedVars, Symbol, count_whitespace_or_commas};
use crate::fixer::{RuleFix, RuleFixer};

impl NoUnusedVars {
    #[expect(clippy::unused_self)]
    pub(in super::super) fn remove_unused_import_declaration<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        import: &ImportDeclaration<'a>,
    ) -> RuleFix {
        let specifiers = import
            .specifiers
            .as_ref()
            .expect("Found an unused variable in an ImportDeclaration with no specifiers. This should be impossible.");

        debug_assert!(
            !specifiers.is_empty(),
            "Found an unused variable in an ImportDeclaration with no specifiers. This should be impossible."
        );

        let Some((position, specifier)) = specifiers.iter().enumerate().find(|(_, s)| symbol == *s)
        else {
            debug_assert!(
                false,
                "Could not find matching specifier for symbol in ImportDeclaration."
            );
            return fixer.noop();
        };

        // `import foo, { bar, baz } from 'module';` where all specifiers are unused
        if Self::all_specifiers_unused(symbol, specifiers) {
            return Self::delete_import_declaration(fixer, import);
        }

        // Count named imports (excludes default and namespace imports)
        let named_import_count = specifiers
            .iter()
            .filter(|s| matches!(s, ImportDeclarationSpecifier::ImportSpecifier(_)))
            .count();

        match specifier {
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                Self::remove_namespace_import(fixer, specifiers, specifier)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                Self::remove_default_import(fixer, specifiers, specifier)
            }
            ImportDeclarationSpecifier::ImportSpecifier(_) => {
                if named_import_count == 1 {
                    return Self::remove_named_imports_block(fixer, specifiers);
                }

                Self::remove_specifier_from_list(fixer, specifiers, position, specifier)
            }
        }
    }

    /// Check if all specifiers in the import are unused (have no references)
    fn all_specifiers_unused(
        symbol: &Symbol<'_, '_>,
        specifiers: &[ImportDeclarationSpecifier<'_>],
    ) -> bool {
        let scoping = symbol.scoping();
        specifiers.iter().all(|specifier| scoping.symbol_is_unused(specifier.local().symbol_id()))
    }

    /// Delete the entire import declaration, preserving blank lines appropriately
    #[expect(clippy::cast_possible_truncation)]
    fn delete_import_declaration<'a>(
        fixer: RuleFixer<'_, 'a>,
        import: &ImportDeclaration<'a>,
    ) -> RuleFix {
        let mut span = import.span;
        let source = fixer.source_text();

        let before_import = &source[..(span.start as usize)];
        let leading_whitespace: u32 = before_import
            .chars()
            .rev()
            .take_while(|c| c.is_whitespace() && *c != '\n' && *c != '\r')
            .map(|c| c.len_utf8() as u32)
            .sum();
        span.start -= leading_whitespace;

        let after_import = &source[(span.end as usize)..];
        let trailing_newlines: u32 = after_import
            .chars()
            .take_while(|c| *c == '\n' || *c == '\r')
            .take(1)
            .map(|c| c.len_utf8() as u32)
            .sum();
        span.end += trailing_newlines;

        fixer.delete_range(span).dangerously()
    }

    /// Remove the entire `{ ... }` block when all named imports are removed
    /// Transforms: `import Default, { Unused } from 'module'` -> `import Default from 'module'`
    fn remove_named_imports_block<'a>(
        fixer: RuleFixer<'_, 'a>,
        specifiers: &[ImportDeclarationSpecifier<'a>],
    ) -> RuleFix {
        let default_specifier = specifiers
            .iter()
            .find(|s| matches!(s, ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));

        let Some(default_spec) = default_specifier else {
            debug_assert!(false, "Expected a default import when removing named imports block.");

            return fixer.noop();
        };

        let last_named = specifiers
            .iter()
            .rev()
            .find(|s| matches!(s, ImportDeclarationSpecifier::ImportSpecifier(_)));

        let Some(last_named) = last_named else {
            return fixer.noop();
        };

        let comma_offset = fixer.find_next_token_from(default_spec.span().end, ",").unwrap_or(0);
        let delete_start = default_spec.span().end + comma_offset;

        let brace_offset =
            fixer.find_next_token_from(last_named.span().end, "}").map_or(0, |i| i + 1);
        let delete_end = last_named.span().end + brace_offset;

        let span = Span::new(delete_start, delete_end);
        fixer.delete_range(span).dangerously()
    }

    /// Remove the default import when there are other imports remaining
    /// Transforms: `import Unused, { Used } from 'module'` -> `import { Used } from 'module'`
    /// Also handles: `import Unused, * as foo from 'module'` -> `import * as foo from 'module'`
    fn remove_default_import<'a>(
        fixer: RuleFixer<'_, 'a>,
        specifiers: &[ImportDeclarationSpecifier<'a>],
        specifier: &ImportDeclarationSpecifier<'a>,
    ) -> RuleFix {
        let default_span = specifier.span();

        let next = specifiers.get(1);

        let Some(next_spec) = next else {
            debug_assert!(
                false,
                "Expected another specifier after default import when removing default import."
            );
            return fixer.noop();
        };

        let next_specifier_start = match next_spec {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                let next_token =
                    fixer.find_next_token_within(default_span.end, specifier.span.start, "{");

                let Some(next_token) = next_token else {
                    debug_assert!(
                        false,
                        "Could not find expected token after default import when removing default import."
                    );
                    return fixer.noop();
                };

                default_span.end + next_token
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => specifier.span.start,
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                debug_assert!(
                    false,
                    "Expected named or namespace import after default import when removing default import."
                );
                return fixer.noop();
            }
        };

        let delete_span = Span::new(default_span.start, next_specifier_start);
        fixer.delete_range(delete_span).dangerously()
    }

    /// Remove the namespace import when there's a default import remaining
    /// Transforms: `import a, * as Unused from 'module'` -> `import a from 'module'`
    fn remove_namespace_import<'a>(
        fixer: RuleFixer<'_, 'a>,
        specifiers: &[ImportDeclarationSpecifier<'a>],
        specifier: &ImportDeclarationSpecifier<'a>,
    ) -> RuleFix {
        let default_specifier = specifiers
            .iter()
            .find(|s| matches!(s, ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));

        let Some(default_spec) = default_specifier else {
            debug_assert!(false, "Expected a default import when removing namespace import.");
            return fixer.noop();
        };

        // `import a, * as foo from 'module'` -> `import a from 'module'`
        let delete_start = default_spec.span().end;
        let delete_end = specifier.span().end;

        let span = Span::new(delete_start, delete_end);
        fixer.delete_range(span).dangerously()
    }

    /// Remove a specifier from a list, handling comma placement properly
    #[expect(clippy::cast_possible_truncation)]
    fn remove_specifier_from_list<'a>(
        fixer: RuleFixer<'_, 'a>,
        specifiers: &[ImportDeclarationSpecifier<'a>],
        position: usize,
        specifier: &ImportDeclarationSpecifier<'a>,
    ) -> RuleFix {
        let source = fixer.source_text();
        let span = specifier.span();

        let named_imports: Vec<_> = specifiers
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, ImportDeclarationSpecifier::ImportSpecifier(_)))
            .collect();

        let named_position = named_imports.iter().position(|(idx, _)| *idx == position);

        let Some(named_pos) = named_position else {
            debug_assert!(false, "Expected to find named import position.");
            return fixer.noop();
        };

        let is_last_named = named_pos == named_imports.len() - 1;
        let is_first_named = named_pos == 0;

        if is_last_named && !is_first_named {
            let prev_specifier = named_imports.get(named_pos - 1).map(|(_, s)| s);
            if let Some(prev) = prev_specifier {
                let delete_start = prev.span().end;
                let delete_end = span.end;

                let text_after = &source[(span.end as usize)..];
                let has_trailing_comma = text_after.trim_start().starts_with(',');

                if has_trailing_comma {
                    let trailing_space: u32 = text_after
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .map(|c| c.len_utf8() as u32)
                        .sum();
                    let delete_span = Span::new(delete_start, delete_end + trailing_space);
                    return fixer.delete_range(delete_span).dangerously();
                }

                let delete_span = Span::new(delete_start, delete_end);
                return fixer.delete_range(delete_span).dangerously();
            }
        }

        let text_after = &source[(span.end as usize)..];
        let trailing = count_whitespace_or_commas(text_after.chars());
        fixer.delete_range(span.expand_right(trailing)).dangerously()
    }
}
