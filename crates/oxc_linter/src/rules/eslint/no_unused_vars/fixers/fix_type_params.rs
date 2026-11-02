use oxc_ast::AstKind;
use oxc_semantic::NodeId;

use super::{NoUnusedVars, Symbol};
use crate::fixer::{RuleFix, RuleFixer};

impl NoUnusedVars {
    /// Rename an unused type parameter to match `varsIgnorePattern`.
    ///
    /// ```ts
    /// type Foo<T> = {};
    /// function foo<T>() {}
    /// ```
    ///
    /// becomes:
    ///
    /// ```ts
    /// type Foo<_T> = {};
    /// function foo<_T>() {}
    /// ```
    ///
    /// Type parameters of interfaces and classes are left alone. Those
    /// declarations can merge with other declarations of the same name (in
    /// this file, in a global or module augmentation, or in a lib file), and
    /// TypeScript requires every merged declaration to use identical type
    /// parameter names.
    pub(in super::super) fn rename_unused_type_parameter<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        declaration_id: NodeId,
    ) -> RuleFix {
        if Self::type_parameter_owner_can_merge(symbol, declaration_id) {
            return fixer.noop();
        }

        if let Some(new_name) = self.get_unused_var_name(symbol) {
            return symbol.rename_with_fixer(fixer, &new_name).dangerously();
        }

        fixer.noop()
    }

    fn type_parameter_owner_can_merge(symbol: &Symbol<'_, '_>, declaration_id: NodeId) -> bool {
        let nodes = symbol.nodes();
        let owner_id = match nodes.parent_kind(declaration_id) {
            AstKind::TSTypeParameterDeclaration(_) => nodes.parent_id(declaration_id),
            _ => declaration_id,
        };
        matches!(
            nodes.parent_kind(owner_id),
            AstKind::TSInterfaceDeclaration(_) | AstKind::Class(_)
        )
    }
}
