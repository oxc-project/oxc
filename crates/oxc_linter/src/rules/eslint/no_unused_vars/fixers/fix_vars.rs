use oxc_ast::{
    ast::{Expression, VariableDeclarator},
    AstKind,
};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::CompactStr;
use regex::Regex;

use super::{count_whitespace_or_commas, BindingInfo, NoUnusedVars, Symbol};
use crate::fixer::{RuleFix, RuleFixer};

impl NoUnusedVars {
    /// Delete a variable declaration or rename it to match `varsIgnorePattern`.
    ///
    /// Variable declarations will only be deleted if they have 0 references of any kind. Renaming
    /// is only attempted if this is not the case. Only a small set of `varsIgnorePattern` values
    /// are supported for renaming. Feel free to add support for more as needed.
    #[allow(clippy::cast_possible_truncation)]
    pub(in super::super) fn rename_or_remove_var_declaration<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        decl: &VariableDeclarator<'a>,
        decl_id: AstNodeId,
    ) -> RuleFix<'a> {
        if decl.init.as_ref().is_some_and(Expression::is_function) {
            return fixer.noop();
        }

        let Some(AstKind::VariableDeclaration(declaration)) =
            symbol.nodes().parent_node(decl_id).map(AstNode::kind)
        else {
            panic!("VariableDeclarator nodes should always be direct children of VariableDeclaration nodes");
        };

        // `true` even if references aren't considered a usage.
        let has_references = symbol.has_references();

        // we can delete variable declarations that aren't referenced anywhere
        if !has_references {
            // for `let x = 1;` or `const { x } = obj; the whole declaration can
            // be removed, but for `const { x, y } = obj;` or `let x = 1, y = 2`
            // we need to keep the other declarations
            let has_neighbors = declaration.declarations.len() > 1;
            debug_assert!(!declaration.declarations.is_empty());
            let binding_info = symbol.get_binding_info(&decl.id.kind);

            match binding_info {
                BindingInfo::SingleDestructure | BindingInfo::NotDestructure => {
                    if has_neighbors {
                        return symbol
                            .delete_from_list(fixer, &declaration.declarations, decl)
                            .dangerously();
                    }
                    return fixer.delete(declaration).dangerously();
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

    fn get_unused_var_name(&self, symbol: &Symbol<'_, '_>) -> Option<CompactStr> {
        let pat = self.vars_ignore_pattern.as_ref().map(Regex::as_str)?;

        let ignored_name: String = match pat {
            // TODO: support more patterns
            "^_" => format!("_{}", symbol.name()),
            _ => return None,
        };

        // adjust name to avoid conflicts
        let scopes = symbol.scopes();
        let scope_id = symbol.scope_id();
        let mut i = 0;
        let mut new_name = ignored_name.clone();
        while scopes.has_binding(scope_id, &new_name) {
            new_name = format!("{ignored_name}{i}");
            i += 1;
        }

        Some(new_name.into())
    }
}
