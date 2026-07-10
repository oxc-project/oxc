use oxc_ast::{
    AstKind,
    ast::{Expression, ForInStatement, ForOfStatement, VariableDeclarator},
};
use oxc_semantic::NodeId;
use oxc_span::GetSpan;

use super::{BindingInfo, NoUnusedVars, Symbol, count_whitespace_or_commas};
use crate::fixer::{RuleFix, RuleFixer};

impl NoUnusedVars {
    /// Delete a variable declaration or rename it to match `varsIgnorePattern`.
    ///
    /// - Variable declarations will only be deleted if they have 0 references of any kind.
    /// - Renaming is only attempted if this is not the case.
    /// - Fixing is skipped for the following cases:
    ///   * Function expressions and arrow functions declared in the root scope
    ///     (`const x = function () {}`)
    ///   * Variables initialized with an `await` expression, since these often
    ///     have side effects (`const unusedRes = await api.createUser(data)`)
    ///
    /// Only a small set of `varsIgnorePattern` values are supported for
    /// renaming. Feel free to add support for more as needed.
    pub(in super::super) fn rename_or_remove_var_declaration<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        decl: &VariableDeclarator<'a>,
        decl_id: NodeId,
    ) -> RuleFix {
        if decl.init.as_ref().is_some_and(|init| is_skipped_init(symbol, init)) {
            return fixer.noop();
        }

        let parent = symbol.nodes().parent_node(decl_id);
        let (span, declarations) = match parent.kind() {
            AstKind::VariableDeclaration(decl) => (decl.span, &decl.declarations),
            _ => {
                #[cfg(debug_assertions)]
                panic!(
                    "VariableDeclarator nodes should always be direct children of VariableDeclaration nodes"
                );
                #[cfg(not(debug_assertions))]
                return fixer.noop();
            }
        };

        if let AstKind::ForOfStatement(ForOfStatement { span, .. })
        | AstKind::ForInStatement(ForInStatement { span, .. }) =
            symbol.nodes().parent_kind(parent.id())
            && span.contains_inclusive(symbol.span())
        {
            if let Some(new_name) = self.get_unused_var_name(symbol) {
                return symbol.rename(&new_name).dangerously();
            }
            return fixer.noop();
        }

        // `true` even if references aren't considered a usage.
        let has_references = symbol.has_references();

        // we can delete variable declarations that aren't referenced anywhere
        if !has_references {
            // for `let x = 1;` or `const { x } = obj; the whole declaration can
            // be removed, but for `const { x, y } = obj;` or `let x = 1, y = 2`
            // we need to keep the other declarations
            let has_neighbors = declarations.len() > 1;
            debug_assert!(!declarations.is_empty());
            let binding_info = symbol.get_binding_info(&decl.id);

            match binding_info {
                BindingInfo::NotDestructure => {
                    if has_neighbors {
                        // if every other declarator in this statement is also
                        // a plain (non-destructured) unused binding that's
                        // safe to remove, delete the whole declaration
                        // instead of leaving the other dead declarators
                        // behind. e.g. `let a = 1, b = 2;` (both unused)
                        // should become `` rather than `let b = 2;`.
                        // Destructured neighbors are left to
                        // `delete_from_list` below (see #16832).
                        if self.all_other_declarators_removable(symbol, decl, declarations) {
                            return fixer.delete_range(span).dangerously();
                        }
                        return symbol.delete_from_list(fixer, declarations, decl).dangerously();
                    }
                    return fixer.delete_range(span).dangerously();
                }
                BindingInfo::SingleDestructure => {
                    if has_neighbors {
                        return symbol.delete_from_list(fixer, declarations, decl).dangerously();
                    }
                    return fixer.delete_range(span).dangerously();
                }
                BindingInfo::MultiDestructure(mut span, is_object, is_last) => {
                    let source_after = &fixer.source_text()[(span.end as usize)..];
                    // remove trailing commas
                    span = span.expand_right(count_whitespace_or_commas(source_after.chars()));

                    // remove leading commas when removing the last element in
                    // an array
                    // `const [a, b] = [1, 2];` -> `const [a, b] = [1, 2];`
                    //            ^                         ^^^
                    if !is_object && is_last {
                        debug_assert!(span.start > 0);
                        let source_before = &fixer.source_text()[..(span.start as usize)];
                        let chars = source_before.chars().rev();
                        let start_offset = count_whitespace_or_commas(chars);
                        // do not walk past the beginning of the file
                        debug_assert!(start_offset < span.start);
                        span = span.expand_left(start_offset);
                    }

                    return if is_object || is_last {
                        fixer.delete_range(span).dangerously()
                    } else {
                        // infix array elements need a comma to preserve
                        // unpacking order of symbols around them
                        // e.g. `const [a, b, c] = [1, 2, 3];` -> `const [a, , c] = [1, 2, 3];`
                        fixer.replace(span, ",").dangerously()
                    };
                }
                BindingInfo::NotFound => {
                    return fixer.noop();
                }
            }
        }

        // otherwise, try to rename the variable to match the unused variable
        // pattern
        if let Some(new_name) = self.get_unused_var_name(symbol) {
            return symbol.rename(&new_name).dangerously();
        }

        fixer.noop()
    }

    /// Checks if every declarator in `declarations` other than `own` is also
    /// safe to remove, i.e. it has no simple `BindingIdentifier` bindings that
    /// are used, exported, ignored, or otherwise excluded from removal.
    ///
    /// Destructured declarators (e.g. `const { a } = obj`) are treated as not
    /// removable here; they're left for `Symbol::delete_from_list` to handle
    /// declarator-by-declarator, since collapsing them into a whole-statement
    /// deletion needs more care (see #16832's destructure-in-multi-declarator
    /// bug).
    fn all_other_declarators_removable<'a>(
        &self,
        symbol: &Symbol<'_, 'a>,
        own: &VariableDeclarator<'a>,
        declarations: &[VariableDeclarator<'a>],
    ) -> bool {
        declarations.iter().all(|other| {
            if std::ptr::eq(other, own) {
                return true;
            }

            let Some(binding) = other.id.get_binding_identifier() else {
                // destructured or otherwise non-trivial bindings are left to
                // the existing per-declarator removal logic
                return false;
            };

            if other.init.as_ref().is_some_and(|init| is_skipped_init(symbol, init)) {
                return false;
            }

            let other_symbol = symbol.with_symbol_id(binding.symbol_id());
            !other_symbol.has_references()
                && !other_symbol.is_exported_binding()
                && !self.is_allowed_variable_declaration(&other_symbol, other)
                && self.is_ignored(&other_symbol).is_none()
        })
    }
}

fn is_skipped_init<'a>(symbol: &Symbol<'_, 'a>, init: &Expression<'a>) -> bool {
    match init.get_inner_expression() {
        // Do not delete function expressions or arrow functions declared in the
        // root scope
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
            symbol.is_root()
        }
        // Skip await expressions, since these are often effectful (e.g.
        // sending a POST request to an API and then not using the response)
        Expression::AwaitExpression(_) => true,
        _ => false,
    }
}
