use lazy_regex::Regex;

use oxc_ast::AstKind;
use oxc_semantic::{Reference, ScopeId, Scoping};
use oxc_span::GetSpan;
use oxc_str::{CompactStr, Ident};

use crate::rules::eslint::no_unused_vars::options::IgnorePattern;

use super::{NoUnusedVars, Symbol};

impl NoUnusedVars {
    pub(super) fn get_unused_arg_name(&self, symbol: &Symbol<'_, '_>) -> Option<CompactStr> {
        Self::get_unused_name(symbol, &self.args_ignore_pattern)
    }

    pub(super) fn get_unused_var_name(&self, symbol: &Symbol<'_, '_>) -> Option<CompactStr> {
        Self::get_unused_name(symbol, &self.vars_ignore_pattern)
    }

    /// Build a replacement name that satisfies the configured ignore pattern
    /// and does not collide with another binding in the same scope.
    ///
    /// This currently supports the default ignore pattern and explicit `^_`
    /// patterns:
    ///
    /// ```js
    /// function foo(unused, _unused) {}
    /// // `unused` becomes `_unused0`, not `_unused`.
    /// ```
    ///
    /// More complex ignore patterns are intentionally skipped until the fixer
    /// can reliably synthesize a matching identifier.
    fn get_unused_name(
        symbol: &Symbol<'_, '_>,
        ignore_pattern: &IgnorePattern<Regex>,
    ) -> Option<CompactStr> {
        let scopes = symbol.scoping();
        let scope_id = symbol.scope_id();
        let ignored_name: String = match ignore_pattern.as_ref() {
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
        let mut i = 0;
        let mut new_name = ignored_name.clone();
        while scopes.scope_has_binding(scope_id, Ident::from(new_name.as_str()))
            || Self::would_capture_existing_reference(symbol, &new_name)
        {
            new_name = format!("{ignored_name}{i}");
            i += 1;
        }

        Some(new_name.into())
    }

    /// Returns `true` when renaming `symbol` to `new_name` would change which
    /// binding an existing reference resolves to.
    ///
    /// For example, renaming the parameter `unused` to `_unused` would capture
    /// the outer `_unused` reference in the function body:
    ///
    /// ```js
    /// const _unused = 1;
    /// function foo(unused) {
    ///     return _unused;
    /// }
    /// ```
    ///
    /// But a binding declared inside the renamed symbol's scope is already
    /// closer than the renamed binding, so it is not captured:
    ///
    /// ```js
    /// function foo(unused) {
    ///     {
    ///         let _unused = 1;
    ///         console.log(_unused);
    ///     }
    /// }
    /// ```
    fn would_capture_existing_reference(symbol: &Symbol<'_, '_>, new_name: &str) -> bool {
        let scoping = symbol.scoping();
        let renamed_scope_id = symbol.scope_id();

        for symbol_id in scoping.symbol_ids() {
            if symbol_id == symbol.id() || scoping.symbol_name(symbol_id) != new_name {
                continue;
            }

            let existing_scope_id = scoping.symbol_scope_id(symbol_id);
            if Self::is_scope_descendant_of(scoping, existing_scope_id, renamed_scope_id) {
                continue;
            }

            if scoping.get_resolved_references(symbol_id).any(|reference| {
                Self::is_scope_descendant_of(scoping, reference.scope_id(), renamed_scope_id)
                    && Self::reference_can_be_captured(symbol, reference)
            }) {
                return true;
            }
        }

        scoping.root_unresolved_references().get(new_name).is_some_and(|reference_ids| {
            reference_ids.iter().any(|&reference_id| {
                let reference = scoping.get_reference(reference_id);
                Self::is_scope_descendant_of(scoping, reference.scope_id(), renamed_scope_id)
                    && Self::reference_can_be_captured(symbol, reference)
            })
        })
    }

    /// Returns whether `reference` is in a namespace that could be rebound by
    /// renaming a value parameter to the same name.
    ///
    /// Pure type references cannot be captured by a value binding:
    ///
    /// ```ts
    /// type _unused = number;
    /// function foo(unused: _unused) {}
    /// ```
    ///
    /// Value-space references in type syntax, such as `typeof _unused`, can be
    /// captured and still need to block the plain `_unused` rename.
    fn reference_can_be_captured(symbol: &Symbol<'_, '_>, reference: &Reference) -> bool {
        let flags = reference.flags();
        if flags.is_value() || flags.is_value_as_type() {
            return true;
        }

        if !flags.is_type_only() {
            return false;
        }

        let nodes = symbol.nodes();
        let reference_span = nodes.kind(reference.node_id()).span();
        nodes.ancestors(reference.node_id()).any(|node| match node.kind() {
            AstKind::TSTypeQuery(_) => true,
            AstKind::TSPropertySignature(signature) => {
                signature.computed && signature.key.span().contains_inclusive(reference_span)
            }
            _ => false,
        })
    }

    fn is_scope_descendant_of(scoping: &Scoping, scope_id: ScopeId, ancestor_id: ScopeId) -> bool {
        scoping.scope_ancestors(scope_id).any(|scope_id| scope_id == ancestor_id)
    }
}
