//! Declare symbol for `BindingIdentifier`s

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{symbol::SymbolFlags, SemanticBuilder};

pub trait Binder {
    fn bind(&self, _builder: &mut SemanticBuilder) {}
}

impl<'a> Binder for Class<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        if let Some(ident) = self.id.as_ref()
            && self.r#type == ClassType::ClassDeclaration && !self.modifiers.contains(ModifierKind::Declare) {
            builder.declare_symbol(
                &ident.name,
                ident.span,
                builder.scope.current_scope_id,
                SymbolFlags::Class ,
                SymbolFlags::ClassExcludes,
            );
        }
    }
}
