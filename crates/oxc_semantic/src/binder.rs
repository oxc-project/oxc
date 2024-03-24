//! Declare symbol for `BindingIdentifier`s

use std::borrow::Cow;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::{
    syntax_directed_operations::{BoundNames, IsSimpleParameterList},
    AstKind,
};
use oxc_span::SourceType;

use crate::{scope::ScopeFlags, symbol::SymbolFlags, SemanticBuilder};

pub trait Binder {
    fn bind(&self, _builder: &mut SemanticBuilder) {}
}

impl<'a> Binder for VariableDeclarator<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.current_scope_id;
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

        if self.kind.is_lexical() {
            self.id.bound_names(&mut |ident| {
                let symbol_id = builder.declare_symbol(ident.span, &ident.name, includes, excludes);
                ident.symbol_id.set(Some(symbol_id));
            });
            return;
        }

        // Logic for scope hoisting `var`

        let mut var_scope_ids = vec![];
        if !builder.current_scope_flags().is_var() {
            for scope_id in builder.scope.ancestors(current_scope_id).skip(1) {
                var_scope_ids.push(scope_id);
                if builder.scope.get_flags(scope_id).is_var() {
                    break;
                }
            }
        }

        self.id.bound_names(&mut |ident| {
            let span = ident.span;
            let name = &ident.name;

            for scope_id in &var_scope_ids {
                if let Some(symbol_id) =
                    builder.check_redeclaration(*scope_id, span, name, excludes, true)
                {
                    ident.symbol_id.set(Some(symbol_id));
                    builder.add_redeclare_variable(symbol_id, ident.span);
                    return;
                }
            }

            let symbol_id =
                builder.declare_symbol_on_scope(span, name, current_scope_id, includes, excludes);
            ident.symbol_id.set(Some(symbol_id));
            for scope_id in &var_scope_ids {
                builder.scope.add_binding(*scope_id, name.to_compact_str(), symbol_id);
            }
        });
    }
}

impl<'a> Binder for Class<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let Some(ident) = &self.id else { return };
        if !self.modifiers.contains(ModifierKind::Declare) {
            let symbol_id = builder.declare_symbol(
                ident.span,
                &ident.name,
                SymbolFlags::Class,
                SymbolFlags::ClassExcludes,
            );
            ident.symbol_id.set(Some(symbol_id));
        }
    }
}

// It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries,
// unless the source text matched by this production is not strict mode code
// and the duplicate entries are only bound by FunctionDeclarations.
// https://tc39.es/ecma262/#sec-block-level-function-declarations-web-legacy-compatibility-semantics
fn function_as_var(flags: ScopeFlags, source_type: SourceType) -> bool {
    flags.is_function() || (source_type.is_script() && flags.is_top())
}

impl<'a> Binder for Function<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.current_scope_id;
        if let Some(ident) = &self.id {
            if !builder.current_scope_flags().is_strict_mode()
                && matches!(
                    builder.nodes.parent_kind(builder.current_node_id),
                    Some(AstKind::IfStatement(_))
                )
            {
                // Do not declare in if single statements,
                // if (false) function f() {} else function g() { }
            } else if self.r#type == FunctionType::FunctionDeclaration {
                // The visitor is already inside the function scope,
                // retrieve the parent scope for the function id to bind to.
                let parent_scope_id = builder.scope.get_parent_id(current_scope_id).unwrap();
                let parent_flags = builder.scope.get_flags(parent_scope_id);

                let (includes, excludes) =
                    if (parent_flags.is_strict_mode() || self.r#async || self.generator)
                        && !function_as_var(parent_flags, builder.source_type)
                    {
                        (
                            SymbolFlags::Function | SymbolFlags::BlockScopedVariable,
                            SymbolFlags::BlockScopedVariableExcludes,
                        )
                    } else {
                        (
                            SymbolFlags::FunctionScopedVariable,
                            SymbolFlags::FunctionScopedVariableExcludes,
                        )
                    };

                let symbol_id = builder.declare_symbol_on_scope(
                    ident.span,
                    &ident.name,
                    parent_scope_id,
                    includes,
                    excludes,
                );
                ident.symbol_id.set(Some(symbol_id));
            } else if self.r#type == FunctionType::FunctionExpression {
                // https://tc39.es/ecma262/#sec-runtime-semantics-instantiateordinaryfunctionexpression
                // 5. Perform ! funcEnv.CreateImmutableBinding(name, false).
                let symbol_id = builder.declare_symbol(
                    ident.span,
                    &ident.name,
                    SymbolFlags::empty(),
                    SymbolFlags::empty(),
                );
                ident.symbol_id.set(Some(symbol_id));
            }
        }

        // bind scope flags: Constructor | GetAccessor | SetAccessor
        debug_assert!(builder.current_scope_flags().contains(ScopeFlags::Function));
        if let Some(kind) = builder.nodes.parent_kind(builder.current_node_id) {
            match kind {
                AstKind::MethodDefinition(def) => {
                    let flag = builder.scope.get_flags_mut(current_scope_id);
                    *flag |= match def.kind {
                        MethodDefinitionKind::Constructor => ScopeFlags::Constructor,
                        MethodDefinitionKind::Get => ScopeFlags::GetAccessor,
                        MethodDefinitionKind::Set => ScopeFlags::SetAccessor,
                        MethodDefinitionKind::Method => ScopeFlags::empty(),
                    };
                }
                AstKind::ObjectProperty(prop) => {
                    let flag = builder.scope.get_flags_mut(current_scope_id);
                    *flag |= match prop.kind {
                        PropertyKind::Get => ScopeFlags::GetAccessor,
                        PropertyKind::Set => ScopeFlags::SetAccessor,
                        PropertyKind::Init => ScopeFlags::empty(),
                    };
                }
                _ => {}
            }
        }
    }
}

impl<'a> Binder for BindingRestElement<'a> {
    // Binds the FormalParameters's rest of a function or method.
    fn bind(&self, builder: &mut SemanticBuilder) {
        let parent_kind = builder.nodes.parent_kind(builder.current_node_id).unwrap();
        let AstKind::FormalParameters(parameters) = parent_kind else {
            return;
        };

        if parameters.kind.is_signature() {
            return;
        }

        let includes = SymbolFlags::FunctionScopedVariable;
        let excludes =
            SymbolFlags::FunctionScopedVariable | SymbolFlags::FunctionScopedVariableExcludes;
        self.bound_names(&mut |ident| {
            let symbol_id = builder.declare_symbol(ident.span, &ident.name, includes, excludes);
            ident.symbol_id.set(Some(symbol_id));
        });
    }
}

impl<'a> Binder for FormalParameter<'a> {
    // Binds the FormalParameter of a function or method.
    fn bind(&self, builder: &mut SemanticBuilder) {
        let parent_kind = builder.nodes.parent_kind(builder.current_node_id).unwrap();
        let AstKind::FormalParameters(parameters) = parent_kind else { unreachable!() };

        if parameters.kind.is_signature() {
            return;
        }

        let includes = SymbolFlags::FunctionScopedVariable;

        let is_not_allowed_duplicate_parameters = matches!(
                parameters.kind,
                // ArrowFormalParameters: UniqueFormalParameters
                FormalParameterKind::ArrowFormalParameters |
                // UniqueFormalParameters : FormalParameters
                // * It is a Syntax Error if BoundNames of FormalParameters contains any duplicate elements.
                FormalParameterKind::UniqueFormalParameters
            ) ||
            // Multiple occurrences of the same BindingIdentifier in a FormalParameterList is only allowed for functions which have simple parameter lists and which are not defined in strict mode code.
            builder.strict_mode() ||
            // FormalParameters : FormalParameterList
            // * It is a Syntax Error if IsSimpleParameterList of FormalParameterList is false and BoundNames of FormalParameterList contains any duplicate elements.
            !parameters.is_simple_parameter_list();

        let excludes = if is_not_allowed_duplicate_parameters {
            SymbolFlags::FunctionScopedVariable | SymbolFlags::FunctionScopedVariableExcludes
        } else {
            SymbolFlags::FunctionScopedVariableExcludes
        };

        self.bound_names(&mut |ident| {
            let symbol_id = builder.declare_symbol(ident.span, &ident.name, includes, excludes);
            ident.symbol_id.set(Some(symbol_id));
        });
    }
}

impl<'a> Binder for CatchClause<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.current_scope_id;
        if let Some(param) = &self.param {
            // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
            // It is a Syntax Error if any element of the BoundNames of CatchParameter also occurs in the VarDeclaredNames of Block
            // unless CatchParameter is CatchParameter : BindingIdentifier
            if let BindingPatternKind::BindingIdentifier(ident) = &param.kind {
                let includes = SymbolFlags::FunctionScopedVariable | SymbolFlags::CatchVariable;
                let symbol_id = builder.declare_shadow_symbol(
                    &ident.name,
                    ident.span,
                    current_scope_id,
                    includes,
                );
                ident.symbol_id.set(Some(symbol_id));
            } else {
                param.bound_names(&mut |ident| {
                    let symbol_id = builder.declare_symbol(
                        ident.span,
                        &ident.name,
                        SymbolFlags::BlockScopedVariable | SymbolFlags::CatchVariable,
                        SymbolFlags::BlockScopedVariableExcludes,
                    );
                    ident.symbol_id.set(Some(symbol_id));
                });
            }
        }
    }
}

fn declare_symbol_for_import_specifier(ident: &BindingIdentifier, builder: &mut SemanticBuilder) {
    let symbol_id = builder.declare_symbol(
        ident.span,
        &ident.name,
        SymbolFlags::ImportBinding,
        SymbolFlags::ImportBindingExcludes,
    );
    ident.symbol_id.set(Some(symbol_id));
}

impl<'a> Binder for ImportSpecifier<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        declare_symbol_for_import_specifier(&self.local, builder);
    }
}

impl<'a> Binder for ImportDefaultSpecifier<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        declare_symbol_for_import_specifier(&self.local, builder);
    }
}

impl<'a> Binder for ImportNamespaceSpecifier<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        declare_symbol_for_import_specifier(&self.local, builder);
    }
}

impl<'a> Binder for TSTypeAliasDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let symbol_id = builder.declare_symbol(
            self.id.span,
            &self.id.name,
            SymbolFlags::TypeAlias,
            SymbolFlags::TypeAliasExcludes,
        );
        self.id.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> Binder for TSInterfaceDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let symbol_id = builder.declare_symbol(
            self.id.span,
            &self.id.name,
            SymbolFlags::Interface,
            SymbolFlags::InterfaceExcludes,
        );
        self.id.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> Binder for TSEnumDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let is_const = self.modifiers.contains(ModifierKind::Const);
        let includes = if is_const { SymbolFlags::ConstEnum } else { SymbolFlags::RegularEnum };
        let excludes = if is_const {
            SymbolFlags::ConstEnumExcludes
        } else {
            SymbolFlags::RegularEnumExcludes
        };
        let symbol_id = builder.declare_symbol(self.id.span, &self.id.name, includes, excludes);
        self.id.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> Binder for TSEnumMember<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        // TODO: Perf
        if matches!(&self.id, TSEnumMemberName::ComputedPropertyName(_)) {
            return;
        }
        let name = match &self.id {
            TSEnumMemberName::Identifier(id) => Cow::Borrowed(id.name.as_str()),
            TSEnumMemberName::StringLiteral(s) => Cow::Borrowed(s.value.as_str()),
            TSEnumMemberName::NumericLiteral(n) => Cow::Owned(n.value.to_string()),
            TSEnumMemberName::ComputedPropertyName(_) => panic!("TODO: implement"),
        };
        builder.declare_symbol(
            self.span,
            &name,
            SymbolFlags::EnumMember,
            SymbolFlags::EnumMemberExcludes,
        );
    }
}

impl<'a> Binder for TSModuleDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        // At declaration time a module has no value declaration it is only when a value declaration
        // is made inside a the scope of a module that the symbol is modified
        let ambient = if self.modifiers.contains(ModifierKind::Declare) {
            SymbolFlags::Ambient
        } else {
            SymbolFlags::None
        };
        builder.declare_symbol(
            self.span,
            self.id.name(),
            SymbolFlags::NameSpaceModule | ambient,
            SymbolFlags::None,
        );
    }
}

impl<'a> Binder for TSTypeParameter<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let symbol_id = builder.declare_symbol(
            self.name.span,
            &self.name.name,
            SymbolFlags::TypeParameter,
            SymbolFlags::TypeParameterExcludes,
        );
        self.name.symbol_id.set(Some(symbol_id));
    }
}
