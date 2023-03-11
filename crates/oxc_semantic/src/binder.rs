//! Declare symbol for `BindingIdentifier`s

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::syntax_directed_operations::BoundNames;

use crate::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
    SemanticBuilder,
};

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

impl<'a> Binder for VariableDeclarator<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.scope.current_scope_id;
        let (includes, excludes) = match self.kind {
            VariableDeclarationKind::Const | VariableDeclarationKind::Let => {
                (SymbolFlags::BlockScopedVariable, SymbolFlags::BlockScopedVariableExcludes)
            }
            VariableDeclarationKind::Var => {
                (SymbolFlags::FunctionScopedVariable, SymbolFlags::FunctionScopedVariableExcludes)
            }
        };
        for ident in self.id.bound_names() {
            let symbol_id = builder.declare_symbol(
                &ident.name,
                ident.span,
                current_scope_id,
                includes,
                excludes,
            );
            if self.kind == VariableDeclarationKind::Var
                && !builder.scope.current_scope().flags.intersects(ScopeFlags::VAR)
            {
                let mut scope_ids = vec![];
                for scope_id in current_scope_id.ancestors(&builder.scope.scopes).skip(1) {
                    let scope = builder.scope.scopes[scope_id].get();
                    if scope.flags.intersects(ScopeFlags::VAR) {
                        scope_ids.push(ScopeId::from(scope_id));
                        break;
                    }
                    scope_ids.push(ScopeId::from(scope_id));
                }
                for scope_id in scope_ids {
                    if builder
                        .check_redeclaration(scope_id, &ident.name, ident.span, excludes)
                        .is_none()
                    {
                        builder.scope.scopes[scope_id]
                            .variables
                            .insert(ident.name.clone(), symbol_id);
                    }
                }
            }
        }
    }
}

impl<'a> Binder for FormalParameters<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let includes = SymbolFlags::FunctionScopedVariable;
        let excludes = SymbolFlags::FunctionScopedVariableExcludes;
        let is_signature = self.kind == FormalParameterKind::Signature;
        for ident in self.bound_names() {
            if !is_signature {
                builder.declare_symbol(
                    &ident.name,
                    ident.span,
                    builder.scope.current_scope_id,
                    includes,
                    excludes,
                );
            }
        }
    }
}
