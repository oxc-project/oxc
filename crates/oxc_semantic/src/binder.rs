//! Declare symbol for `BindingIdentifier`s

use std::{borrow::Cow, ptr};

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_span::{GetSpan, SourceType};
use oxc_syntax_operations::{BoundNames, IsSimpleParameterList};

use crate::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
    SemanticBuilder,
};

pub(crate) trait Binder<'a> {
    #[allow(unused_variables)]
    fn bind(&self, builder: &mut SemanticBuilder<'a>) {}
}

impl<'a> Binder<'a> for VariableDeclarator<'a> {
    fn bind(&self, builder: &mut SemanticBuilder<'a>) {
        let (includes, excludes) = match self.kind {
            VariableDeclarationKind::Const => (
                SymbolFlags::BlockScopedVariable | SymbolFlags::ConstVariable,
                SymbolFlags::BlockScopedVariableExcludes,
            ),
            VariableDeclarationKind::Let => {
                (SymbolFlags::BlockScopedVariable, SymbolFlags::BlockScopedVariableExcludes)
            }
            VariableDeclarationKind::Var
            | VariableDeclarationKind::Using
            | VariableDeclarationKind::AwaitUsing => {
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

        // ------------------ var hosting ------------------
        let mut target_scope_id = builder.current_scope_id;
        let mut var_scope_ids = vec![];

        // Collect all scopes where variable hoisting can occur
        for scope_id in builder.scope.ancestors(target_scope_id) {
            let flags = builder.scope.get_flags(scope_id);
            if flags.is_var() {
                target_scope_id = scope_id;
                break;
            }
            var_scope_ids.push(scope_id);
        }

        self.id.bound_names(&mut |ident| {
            let span = ident.span;
            let name = &ident.name;
            let mut declared_symbol_id = None;

            for &scope_id in &var_scope_ids {
                if let Some(symbol_id) =
                    builder.check_redeclaration(scope_id, span, name, excludes, true)
                {
                    builder.add_redeclare_variable(symbol_id, span);
                    declared_symbol_id = Some(symbol_id);

                    let name = name.to_compact_str();
                    // remove current scope binding and add to target scope
                    // avoid same symbols appear in multi-scopes
                    builder.scope.remove_binding(scope_id, &name);
                    builder.scope.add_binding(target_scope_id, name, symbol_id);
                    builder.symbols.scope_ids[symbol_id] = target_scope_id;
                    break;
                }
            }

            // If a variable is already declared in the hoisted scopes,
            // we don't need to create another symbol with the same name
            // to make sure they point to the same symbol.
            let symbol_id = declared_symbol_id.unwrap_or_else(|| {
                builder.declare_symbol_on_scope(span, name, target_scope_id, includes, excludes)
            });
            ident.symbol_id.set(Some(symbol_id));

            // Finally, add the variable to all hoisted scopes
            // to support redeclaration checks when declaring variables with the same name later.
            for &scope_id in &var_scope_ids {
                builder
                    .hoisting_variables
                    .entry(scope_id)
                    .or_default()
                    .insert(name.clone(), symbol_id);
            }
        });
    }
}

impl<'a> Binder<'a> for Class<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        if !self.declare {
            let Some(ident) = &self.id else { return };
            let symbol_id = builder.declare_symbol(
                ident.span,
                &ident.name,
                SymbolFlags::Class,
                if self.is_declaration() {
                    SymbolFlags::ClassExcludes
                } else {
                    SymbolFlags::empty()
                },
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

/// Check for Annex B `if (foo) function a() {} else function b() {}`
fn is_function_part_of_if_statement(function: &Function, builder: &SemanticBuilder) -> bool {
    if builder.current_scope_flags().is_strict_mode() {
        return false;
    }
    let Some(AstKind::IfStatement(stmt)) = builder.nodes.parent_kind(builder.current_node_id)
    else {
        return false;
    };
    if let Statement::FunctionDeclaration(func) = &stmt.consequent {
        if ptr::eq(func.as_ref(), function) {
            return true;
        }
    }
    if let Some(Statement::FunctionDeclaration(func)) = &stmt.alternate {
        if ptr::eq(func.as_ref(), function) {
            return true;
        }
    }
    false
}

impl<'a> Binder<'a> for Function<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.current_scope_id;
        let scope_flags = builder.current_scope_flags();
        if let Some(ident) = &self.id {
            if is_function_part_of_if_statement(self, builder) {
                let symbol_id = builder.symbols.create_symbol(
                    ident.span,
                    ident.name.clone().into(),
                    SymbolFlags::Function,
                    ScopeId::new(u32::MAX - 1), // Not bound to any scope.
                    builder.current_node_id,
                );
                ident.symbol_id.set(Some(symbol_id));
            } else if self.r#type == FunctionType::FunctionDeclaration {
                // The visitor is already inside the function scope,
                // retrieve the parent scope for the function id to bind to.

                let (includes, excludes) =
                    if (scope_flags.is_strict_mode() || self.r#async || self.generator)
                        && !function_as_var(scope_flags, builder.source_type)
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

                let symbol_id = builder.declare_symbol(ident.span, &ident.name, includes, excludes);
                ident.symbol_id.set(Some(symbol_id));
            } else if self.r#type == FunctionType::FunctionExpression {
                // https://tc39.es/ecma262/#sec-runtime-semantics-instantiateordinaryfunctionexpression
                // 5. Perform ! funcEnv.CreateImmutableBinding(name, false).
                let symbol_id = builder.declare_symbol(
                    ident.span,
                    &ident.name,
                    SymbolFlags::Function,
                    SymbolFlags::empty(),
                );
                ident.symbol_id.set(Some(symbol_id));
            }
        }

        // Bind scope flags: GetAccessor | SetAccessor
        if let Some(AstKind::ObjectProperty(prop)) =
            builder.nodes.parent_kind(builder.current_node_id)
        {
            let flags = builder.scope.get_flags_mut(current_scope_id);
            match prop.kind {
                PropertyKind::Get => *flags |= ScopeFlags::GetAccessor,
                PropertyKind::Set => *flags |= ScopeFlags::SetAccessor,
                PropertyKind::Init => {}
            };
        }
    }
}

impl<'a> Binder<'a> for BindingRestElement<'a> {
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

impl<'a> Binder<'a> for FormalParameter<'a> {
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

impl<'a> Binder<'a> for CatchParameter<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let current_scope_id = builder.current_scope_id;
        // https://tc39.es/ecma262/#sec-variablestatements-in-catch-blocks
        // It is a Syntax Error if any element of the BoundNames of CatchParameter also occurs in the VarDeclaredNames of Block
        // unless CatchParameter is CatchParameter : BindingIdentifier
        if let BindingPatternKind::BindingIdentifier(ident) = &self.pattern.kind {
            let includes = SymbolFlags::FunctionScopedVariable | SymbolFlags::CatchVariable;
            let symbol_id =
                builder.declare_shadow_symbol(&ident.name, ident.span, current_scope_id, includes);
            ident.symbol_id.set(Some(symbol_id));
        } else {
            self.pattern.bound_names(&mut |ident| {
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

fn declare_symbol_for_import_specifier(
    ident: &BindingIdentifier,
    is_type: bool,
    builder: &mut SemanticBuilder,
) {
    let includes = if is_type
        || builder.nodes.parent_kind(builder.current_node_id).is_some_and(
            |decl| matches!(decl, AstKind::ImportDeclaration(decl) if decl.import_kind.is_type()),
        ) {
        SymbolFlags::TypeImport
    } else {
        SymbolFlags::Import
    };

    let symbol_id = builder.declare_symbol(
        ident.span,
        &ident.name,
        includes,
        SymbolFlags::ImportBindingExcludes,
    );
    ident.symbol_id.set(Some(symbol_id));
}

impl<'a> Binder<'a> for ImportSpecifier<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        declare_symbol_for_import_specifier(&self.local, self.import_kind.is_type(), builder);
    }
}

impl<'a> Binder<'a> for ImportDefaultSpecifier<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        declare_symbol_for_import_specifier(&self.local, false, builder);
    }
}

impl<'a> Binder<'a> for ImportNamespaceSpecifier<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        declare_symbol_for_import_specifier(&self.local, false, builder);
    }
}

impl<'a> Binder<'a> for TSImportEqualsDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        declare_symbol_for_import_specifier(&self.id, false, builder);
    }
}

impl<'a> Binder<'a> for TSTypeAliasDeclaration<'a> {
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

impl<'a> Binder<'a> for TSInterfaceDeclaration<'a> {
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

impl<'a> Binder<'a> for TSEnumDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let is_const = self.r#const;
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

impl<'a> Binder<'a> for TSEnumMember<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        // TODO: Perf
        if self.id.is_expression() {
            return;
        }
        let name = match &self.id {
            TSEnumMemberName::StaticIdentifier(id) => Cow::Borrowed(id.name.as_str()),
            TSEnumMemberName::StaticStringLiteral(s) => Cow::Borrowed(s.value.as_str()),
            TSEnumMemberName::StaticTemplateLiteral(s) => Cow::Borrowed(
                s.quasi().expect("Template enum members must have no substitutions.").as_str(),
            ),
            TSEnumMemberName::StaticNumericLiteral(n) => Cow::Owned(n.value.to_string()),
            match_expression!(TSEnumMemberName) => panic!("TODO: implement"),
        };
        builder.declare_symbol(
            self.span,
            &name,
            SymbolFlags::EnumMember,
            SymbolFlags::EnumMemberExcludes,
        );
    }
}

impl<'a> Binder<'a> for TSModuleDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        // do not bind `global` for `declare global { ... }`
        if matches!(self.kind, TSModuleDeclarationKind::Global) {
            return;
        }

        // At declaration time a module has no value declaration it is only when a value declaration
        // is made inside a the scope of a module that the symbol is modified
        let ambient = if self.declare { SymbolFlags::Ambient } else { SymbolFlags::None };
        let symbol_id = builder.declare_symbol(
            self.id.span(),
            self.id.name().as_str(),
            SymbolFlags::NameSpaceModule | ambient,
            SymbolFlags::None,
        );

        // do not bind `global` for `declare global { ... }`
        if !self.kind.is_global() {
            if let TSModuleDeclarationName::Identifier(id) = &self.id {
                id.symbol_id.set(Some(symbol_id));
            }
        }
    }
}

impl<'a> Binder<'a> for TSTypeParameter<'a> {
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
