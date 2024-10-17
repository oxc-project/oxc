use oxc_ast::{
    ast::{Expression, VariableDeclarator},
    AstKind,
};
use oxc_semantic::{AstNode, NodeId};
use oxc_span::CompactStr;

use super::{count_whitespace_or_commas, BindingInfo, NoUnusedVars, Symbol};
use crate::{
    fixer::{RuleFix, RuleFixer},
    rules::eslint::no_unused_vars::options::IgnorePattern,
};

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
    #[allow(clippy::cast_possible_truncation)]
    pub(in super::super) fn rename_or_remove_var_declaration<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        decl: &VariableDeclarator<'a>,
        decl_id: NodeId,
    ) -> RuleFix<'a> {
        if decl.init.as_ref().is_some_and(|init| is_skipped_init(symbol, init)) {
            return fixer.noop();
        }

        let Some(parent) = symbol.nodes().parent_node(decl_id).map(AstNode::kind) else {
            #[cfg(debug_assertions)]
            panic!("VariableDeclarator nodes should always have a parent node");
            #[cfg(not(debug_assertions))]
            return fixer.noop();
        };
        let (span, declarations) = match parent {
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

        // `true` even if references aren't considered a usage.
        let has_references = symbol.has_references();

        // we can delete variable declarations that aren't referenced anywhere
        if !has_references {
            // for `let x = 1;` or `const { x } = obj; the whole declaration can
            // be removed, but for `const { x, y } = obj;` or `let x = 1, y = 2`
            // we need to keep the other declarations
            let has_neighbors = declarations.len() > 1;
            debug_assert!(!declarations.is_empty());
            let binding_info = symbol.get_binding_info(&decl.id.kind);

            match binding_info {
                BindingInfo::SingleDestructure | BindingInfo::NotDestructure => {
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

    fn get_unused_var_name(&self, symbol: &Symbol<'_, '_>) -> Option<CompactStr> {
        let ignored_name: String = match self.vars_ignore_pattern.as_ref() {
            // TODO: support more patterns
            IgnorePattern::Default => {
                format!("_{}", symbol.name())
            }
            IgnorePattern::Some(re) if re.as_str() == "^_" => {
                format!("_{}", symbol.name())
            }
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
