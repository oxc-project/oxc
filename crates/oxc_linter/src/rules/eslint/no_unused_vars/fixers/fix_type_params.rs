use oxc_ast::AstKind;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use super::{NoUnusedVars, Symbol};
use crate::fixer::{RuleFix, RuleFixer};

impl NoUnusedVars {
    /// Remove an unused type parameter.
    ///
    /// ```ts
    /// type Foo<T> = {};
    /// function foo<T, U>(u: U) {}
    /// ```
    ///
    /// becomes:
    ///
    /// ```ts
    /// type Foo = {};
    /// function foo<U>(u: U) {}
    /// ```
    ///
    /// The type parameter list is removed entirely when the last parameter is
    /// removed. No fix is offered for:
    /// - type parameters of interfaces and classes, since those declarations
    ///   can merge with other declarations of the same name, which must all
    ///   declare identical type parameters;
    /// - `infer` type parameters in conditional types, which cannot simply be
    ///   removed.
    ///
    /// This is a dangerous fix: references that pass explicit type arguments
    /// (`foo<string, number>()`) will no longer match the new arity.
    pub(in super::super) fn remove_unused_type_parameter<'a>(
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        declaration_id: NodeId,
    ) -> RuleFix {
        let nodes = symbol.nodes();
        let decl_id = nodes.parent_id(declaration_id);
        let AstKind::TSTypeParameterDeclaration(decl) = nodes.kind(decl_id) else {
            return fixer.noop();
        };
        if matches!(
            nodes.parent_kind(decl_id),
            AstKind::TSInterfaceDeclaration(_) | AstKind::Class(_)
        ) {
            return fixer.noop();
        }

        let params = &decl.params;
        let Some(position) = params.iter().position(|param| symbol == &param.name) else {
            return fixer.noop();
        };

        let fix = if params.len() == 1 {
            // `type Foo<T> = {}` -> `type Foo = {}`
            fixer.delete_range(decl.span)
        } else if let Some(right) = params.get(position + 1) {
            // `<T, U>` -> `<U>`
            fixer.delete_range(Span::new(params[position].span.start, right.span.start))
        } else {
            // `<T, U>` -> `<T>`, also keeping a trailing comma: `<T, U,>` -> `<T,>`
            let left = &params[position - 1];
            fixer.delete_range(Span::new(left.span().end, params[position].span.end))
        };

        fix.with_message(format!("Remove unused type parameter '{}'", symbol.name())).dangerously()
    }
}
