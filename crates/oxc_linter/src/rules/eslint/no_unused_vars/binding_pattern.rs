use oxc_ast::ast::*;
use oxc_semantic::{Semantic, SymbolId};

use crate::ModuleRecord;

use super::{symbol::Symbol, NoUnusedVars};

#[derive(Clone, Copy)]
pub(super) struct BindingContext<'s, 'a> {
    pub options: &'s NoUnusedVars,
    pub semantic: &'s Semantic<'a>,
    pub module_record: &'s ModuleRecord,
}

impl<'s, 'a> BindingContext<'s, 'a> {
    #[inline]
    pub fn symbol(&self, module_record: &'s ModuleRecord, symbol_id: SymbolId) -> Symbol<'s, 'a> {
        Symbol::new(self.semantic, module_record, symbol_id)
    }

    #[inline]
    pub fn has_usages(&self, symbol_id: SymbolId, module_record: &'s ModuleRecord) -> bool {
        self.symbol(module_record, symbol_id).has_usages(self.options)
    }
}

pub(super) trait HasAnyUsedBinding<'a> {
    /// Returns `true` if this node contains a binding that is used or ignored.
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool;
}

impl<'a> HasAnyUsedBinding<'a> for BindingPattern<'a> {
    #[inline]
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        self.kind.has_any_used_binding(ctx)
    }
}
impl<'a> HasAnyUsedBinding<'a> for BindingPatternKind<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        match self {
            Self::BindingIdentifier(id) => id.has_any_used_binding(ctx),
            Self::AssignmentPattern(id) => id.left.has_any_used_binding(ctx),
            Self::ObjectPattern(id) => id.has_any_used_binding(ctx),
            Self::ArrayPattern(id) => id.has_any_used_binding(ctx),
        }
    }
}

impl<'a> HasAnyUsedBinding<'a> for BindingIdentifier<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        ctx.has_usages(self.symbol_id(), ctx.module_record)
    }
}
impl<'a> HasAnyUsedBinding<'a> for ObjectPattern<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        if ctx.options.ignore_rest_siblings && self.rest.is_some() {
            return true;
        }
        self.properties.iter().any(|p| p.value.has_any_used_binding(ctx))
            || self.rest.as_ref().is_some_and(|rest| rest.argument.has_any_used_binding(ctx))
    }
}
impl<'a> HasAnyUsedBinding<'a> for ArrayPattern<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        self.elements.iter().flatten().any(|el| {
            // if the destructured element is ignored, it is considered used
            el.get_identifier_name()
                .is_some_and(|name| ctx.options.is_ignored_array_destructured(&name))
                || el.has_any_used_binding(ctx)
        }) || self.rest.as_ref().is_some_and(|rest| rest.argument.has_any_used_binding(ctx))
    }
}
