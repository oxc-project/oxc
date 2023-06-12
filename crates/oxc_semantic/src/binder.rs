//! Declare symbol for `BindingIdentifier`s

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::{syntax_directed_operations::BoundNames, AstKind};
use oxc_span::SourceType;

use crate::{
    scope::{Scope, ScopeFlags, ScopeId},
    symbol::SymbolFlags,
    SemanticBuilder,
};

pub trait Binder {
    fn bind(&self, _builder: &mut SemanticBuilder) {}
}

impl<'a> Binder for VariableDeclarator<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.scope.current_scope_id;
        let (includes, excludes) = match self.kind {
            VariableDeclarationKind::Const => (
                SymbolFlags::BlockScopedVariable | SymbolFlags::ConstVariable,
                SymbolFlags::BlockScopedVariableExcludes,
            ),
            VariableDeclarationKind::Let => {
                (SymbolFlags::BlockScopedVariable, SymbolFlags::BlockScopedVariableExcludes)
            }
            VariableDeclarationKind::Var => {
                (SymbolFlags::FunctionScopedVariable, SymbolFlags::FunctionScopedVariableExcludes)
            }
        };
        self.id.bound_names(&mut |ident| {
            let symbol_id = builder.declare_symbol(
                &ident.name,
                ident.span,
                current_scope_id,
                includes,
                excludes,
            );
            if self.kind == VariableDeclarationKind::Var
                && !builder.scope.current_scope().flags.intersects(ScopeFlags::Var)
            {
                let mut scope_ids = vec![];
                for scope_id in current_scope_id.ancestors(&builder.scope.scopes).skip(1) {
                    let scope = builder.scope.scopes[scope_id].get();
                    if scope.flags.intersects(ScopeFlags::Var) {
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
        });
    }
}

impl<'a> Binder for Class<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        if let Some(ident) = &self.id && !self.modifiers.contains(ModifierKind::Declare) {
            builder.declare_symbol(
                &ident.name,
                ident.span,
                builder.scope.current_scope_id,
                SymbolFlags::Class,
                SymbolFlags::ClassExcludes,
            );
        }
    }
}

// It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries,
// unless the source text matched by this production is not strict mode code
// and the duplicate entries are only bound by FunctionDeclarations.
// https://tc39.es/ecma262/#sec-block-level-function-declarations-web-legacy-compatibility-semantics
fn function_as_var(scope: &Scope, source_type: SourceType) -> bool {
    scope.flags.intersects(ScopeFlags::Function)
        || (source_type.is_script() && scope.flags.intersects(ScopeFlags::Top))
}

impl<'a> Binder for Function<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        if let Some(ident) = &self.id {
            let current_scope_id = builder.scope.current_scope_id;
            let scope = builder.scope.current_scope();
            if !scope.strict_mode && matches!(builder.parent_kind(), AstKind::IfStatement(_)) {
                // Do not declare in if single statements,
                // if (false) function f() {} else function g() { }
            } else if self.r#type == FunctionType::FunctionDeclaration {
                // The visitor is already inside the function scope,
                // retrieve the parent scope for the function id to bind to.
                let parent_scope_id =
                    builder.scope.scopes[*current_scope_id].parent().unwrap().into();
                let parent_scope: &Scope = &builder.scope.scopes[parent_scope_id];

                let (includes, excludes) =
                    if (parent_scope.strict_mode || self.r#async || self.generator)
                        && !function_as_var(parent_scope, builder.source_type)
                    {
                        (SymbolFlags::BlockScopedVariable, SymbolFlags::BlockScopedVariableExcludes)
                    } else {
                        (
                            SymbolFlags::FunctionScopedVariable,
                            SymbolFlags::FunctionScopedVariableExcludes,
                        )
                    };

                builder.declare_symbol(
                    &ident.name,
                    ident.span,
                    parent_scope_id,
                    includes,
                    excludes,
                );
            }
        }
    }
}

impl<'a> Binder for FormalParameters<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let includes = SymbolFlags::FunctionScopedVariable;
        let excludes = SymbolFlags::FunctionScopedVariableExcludes;
        let is_signature = self.kind == FormalParameterKind::Signature;
        self.bound_names(&mut |ident| {
            if !is_signature {
                builder.declare_symbol(
                    &ident.name,
                    ident.span,
                    builder.scope.current_scope_id,
                    includes,
                    excludes,
                );
            }
        });
    }
}

impl<'a> Binder for CatchClause<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.scope.current_scope_id;
        if let Some(param) = &self.param {
            // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
            // It is a Syntax Error if any element of the BoundNames of CatchParameter also occurs in the VarDeclaredNames of Block
            // unless CatchParameter is CatchParameter : BindingIdentifier
            if let BindingPatternKind::BindingIdentifier(ident) = &param.kind {
                let includes = SymbolFlags::FunctionScopedVariable | SymbolFlags::CatchVariable;
                builder.declare_shadow_symbol(&ident.name, ident.span, current_scope_id, includes);
            } else {
                param.bound_names(&mut |ident| {
                    builder.declare_symbol(
                        &ident.name,
                        ident.span,
                        current_scope_id,
                        SymbolFlags::BlockScopedVariable | SymbolFlags::CatchVariable,
                        SymbolFlags::BlockScopedVariableExcludes,
                    );
                });
            }
        }
    }
}

impl<'a> Binder for ModuleDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        self.bound_names(&mut |ident| {
            builder.declare_symbol(
                &ident.name,
                ident.span,
                builder.scope.current_scope_id,
                SymbolFlags::empty(),
                SymbolFlags::empty(),
            );
        });
    }
}
