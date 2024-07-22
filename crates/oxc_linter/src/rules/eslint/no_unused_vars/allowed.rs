//! This module checks if an unused variable is allowed. Note that this does not
//! consider variables ignored by name pattern, but by where they are declared.
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;

use super::binding_pattern::CheckBinding;
use super::{options::ArgsOption, NoUnusedVars, Symbol};

impl NoUnusedVars {
    pub(super) fn is_allowed_argument<'a>(
        &self,
        semantic: &Semantic<'a>,
        symbol: &Symbol<'_, 'a>,
        param: &FormalParameter<'a>,
    ) -> bool {
        // early short-circuit when no argument checking should be performed
        if self.args.is_none() {
            return true;
        }

        // find FormalParameters. Should be the next parent of param, but this
        // is safer.
        let Some((params, params_id)) = symbol
            .iter_parents()
            .filter_map(|p| {
                if let AstKind::FormalParameters(params) = p.kind() {
                    Some((params, p.id()))
                } else {
                    None
                }
            })
            .next()
        else {
            debug_assert!(false, "FormalParameter should always have a parent FormalParameters");
            return false;
        };

        // arguments inside setters are allowed
        // (1 to skip self, then the next should be a function or method) = 2
        let maybe_method_or_fn =
            semantic.nodes().iter_parents(params_id).nth(2).map(|node| node.kind());
        if matches!(
            maybe_method_or_fn,
            Some(
                AstKind::MethodDefinition(MethodDefinition { kind: MethodDefinitionKind::Set, .. })
                    | AstKind::ObjectProperty(ObjectProperty { kind: PropertyKind::Set, .. })
            )
        ) {
            return true;
        }

        // Parameters are always checked. Must be done after above checks,
        // because in those cases a parameter is required
        if self.args.is_none() {
            return false;
        }

        debug_assert_eq!(self.args, ArgsOption::AfterUsed);

        // from eslint rule documentation:
        // after-used - unused positional arguments that occur before the last
        // used argument will not be checked, but all named arguments and all
        // positional arguments after the last used argument will be checked.

        // check if this is a positional argument - unused non-positional
        // arguments are never allowed
        if param.pattern.kind.is_destructuring_pattern() {
            return false;
        }

        // find the index of the parameter in the parameters list. We want to
        // check all parameters after this one for usages.
        let position =
            params.items.iter().enumerate().find(|(_, p)| p.span == param.span).map(|(i, _)| i);
        debug_assert!(
            position.is_some(),
            "could not find FormalParameter in a FormalParameters node that is its parent."
        );
        let Some(position) = position else {
            return false;
        };

        // This is the last parameter, so need to check for usages on following parameters
        if position == params.items.len() - 1 {
            return false;
        }

        params.items.iter().skip(position + 1).any(|p| {
            let Some(id) = p.pattern.get_binding_identifier() else {
                return false;
            };
            let Some(symbol_id) = id.symbol_id.get() else {
                return false;
            };
            let symbol = Symbol::new(semantic, symbol_id);
            symbol.has_usages()
        })
    }
}
