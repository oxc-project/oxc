//! Declare symbol for `BindingIdentifier`s

use oxc_allocator::{GetAddress, UnstableAddress};
use oxc_ast::{AstKind, ast::*};
use oxc_ecmascript::{BoundNames, IsSimpleParameterList};
use oxc_span::GetSpan;
use oxc_syntax::{node::NodeId, scope::ScopeFlags, symbol::SymbolFlags};

use crate::{SemanticBuilder, checker::is_function_part_of_if_statement};

pub trait Binder<'a> {
    fn bind(&self, builder: &mut SemanticBuilder<'a>);
}

impl<'a> Binder<'a> for VariableDeclarator<'a> {
    fn bind(&self, builder: &mut SemanticBuilder<'a>) {
        let is_declare = matches!(builder
            .nodes
            .parent_kind(builder.current_node_id), AstKind::VariableDeclaration(decl) if decl.declare);

        let (mut includes, excludes) = match self.kind {
            VariableDeclarationKind::Const
            | VariableDeclarationKind::Using
            | VariableDeclarationKind::AwaitUsing => (
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

        if is_declare {
            includes |= SymbolFlags::Ambient;
        }

        if self.kind.is_lexical() {
            self.id.bound_names(&mut |ident| {
                let symbol_id = builder.declare_symbol(ident.span, &ident.name, includes, excludes);
                ident.symbol_id.set(Some(symbol_id));
            });
        } else {
            // ------------------ var hosting ------------------
            let mut target_scope_id = builder.current_scope_id;
            let mut var_scope_ids = vec![];

            // Collect all scopes where variable hoisting can occur
            for scope_id in builder.scoping.scope_ancestors(target_scope_id) {
                let flags = builder.scoping.scope_flags(scope_id);
                if flags.is_var() {
                    target_scope_id = scope_id;
                    break;
                }
                var_scope_ids.push(scope_id);
            }

            self.id.bound_names(&mut |ident| {
                let span = ident.span;
                let name = ident.name;
                let mut declared_symbol_id = None;

                for &scope_id in &var_scope_ids {
                    if let Some(symbol_id) =
                        builder.check_redeclaration(scope_id, span, &name, excludes, true)
                    {
                        builder.add_redeclare_variable(symbol_id, includes, span);
                        declared_symbol_id = Some(symbol_id);

                        // Hoist current symbol to target scope when it is not already declared
                        // in the target scope.
                        if !builder.scoping.scope_has_binding(target_scope_id, &name) {
                            // remove current scope binding and add to target scope
                            // avoid same symbols appear in multi-scopes
                            builder.scoping.remove_binding(scope_id, &name);
                            builder.scoping.add_binding(target_scope_id, &name, symbol_id);
                            builder.scoping.symbol_scope_ids[symbol_id] = target_scope_id;
                        }
                        break;
                    }
                }

                // If a variable is already declared in the hoisted scopes,
                // we don't need to create another symbol with the same name
                // to make sure they point to the same symbol.
                let symbol_id = declared_symbol_id.unwrap_or_else(|| {
                    builder.declare_symbol_on_scope(
                        span,
                        &name,
                        target_scope_id,
                        includes,
                        excludes,
                    )
                });
                ident.symbol_id.set(Some(symbol_id));

                // Finally, add the variable to all hoisted scopes
                // to support redeclaration checks when declaring variables with the same name later.
                for &scope_id in &var_scope_ids {
                    builder.hoisting_variables.entry(scope_id).or_default().insert(name, symbol_id);
                }
            });
        }

        // Save `@__NO_SIDE_EFFECTS__` for function initializers.
        if let BindingPatternKind::BindingIdentifier(id) = &self.id.kind
            && let Some(symbol_id) = id.symbol_id.get()
            && let Some(init) = &self.init
            && match init {
                Expression::FunctionExpression(func) => func.pure,
                Expression::ArrowFunctionExpression(func) => func.pure,
                _ => false,
            }
        {
            builder.scoping.no_side_effects.insert(symbol_id);
        }
    }
}

impl<'a> Binder<'a> for Class<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let includes = if self.declare {
            SymbolFlags::Class | SymbolFlags::Ambient
        } else {
            SymbolFlags::Class
        };
        let Some(ident) = &self.id else { return };
        let symbol_id =
            builder.declare_symbol(ident.span, &ident.name, includes, SymbolFlags::ClassExcludes);
        ident.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> Binder<'a> for Function<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let includes = if self.declare {
            SymbolFlags::Function | SymbolFlags::Ambient
        } else {
            SymbolFlags::Function
        };

        if let Some(ident) = &self.id {
            let excludes = if builder.source_type.is_typescript() {
                SymbolFlags::FunctionExcludes
            } else if is_function_part_of_if_statement(self, builder) {
                SymbolFlags::empty()
            } else {
                // `var x; function x() {}` is valid in non-strict mode, but `TypeScript`
                // doesn't care about non-strict mode, so we need to exclude this,
                // and further check in checker.
                SymbolFlags::FunctionExcludes - SymbolFlags::FunctionScopedVariable
            };

            let symbol_id = builder.declare_symbol(ident.span, &ident.name, includes, excludes);
            ident.symbol_id.set(Some(symbol_id));

            // Save `@__NO_SIDE_EFFECTS__`
            if self.pure {
                builder.scoping.no_side_effects.insert(symbol_id);
            }
        }

        // Bind scope flags: GetAccessor | SetAccessor
        if let AstKind::ObjectProperty(prop) = builder.nodes.parent_kind(builder.current_node_id) {
            // Do not bind scope flags when function is inside of the object property key:
            //
            // { set [function() {}](val) {} }
            //        ^^^^^^^^^^^^^
            if prop.key.span() == self.span {
                return;
            }
            let flags = builder.scoping.scope_flags_mut(builder.current_scope_id);
            match prop.kind {
                PropertyKind::Get => *flags |= ScopeFlags::GetAccessor,
                PropertyKind::Set => *flags |= ScopeFlags::SetAccessor,
                PropertyKind::Init => {}
            }
        }
    }
}

impl<'a> Binder<'a> for BindingRestElement<'a> {
    // Binds the FormalParameters's rest of a function or method.
    fn bind(&self, builder: &mut SemanticBuilder) {
        let parent_kind = builder.nodes.parent_kind(builder.current_node_id);
        let AstKind::FormalParameters(_) = parent_kind else {
            return;
        };

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
        let parent_kind = builder.nodes.parent_kind(builder.current_node_id);
        let AstKind::FormalParameters(parameters) = parent_kind else { unreachable!() };

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
        || matches!(builder.nodes.parent_kind(builder.current_node_id), AstKind::ImportDeclaration(decl) if decl.import_kind.is_type(),
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
        let includes = if self.declare {
            SymbolFlags::TypeAlias | SymbolFlags::Ambient
        } else {
            SymbolFlags::TypeAlias
        };
        let symbol_id = builder.declare_symbol(
            self.id.span,
            &self.id.name,
            includes,
            SymbolFlags::TypeAliasExcludes,
        );
        self.id.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> Binder<'a> for TSInterfaceDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let includes = if self.declare {
            SymbolFlags::Interface | SymbolFlags::Ambient
        } else {
            SymbolFlags::Interface
        };
        let symbol_id = builder.declare_symbol(
            self.id.span,
            &self.id.name,
            includes,
            SymbolFlags::InterfaceExcludes,
        );
        self.id.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> Binder<'a> for TSEnumDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let is_const = self.r#const;
        let includes = if self.declare { SymbolFlags::Ambient } else { SymbolFlags::empty() };
        let (includes, excludes) = if is_const {
            (SymbolFlags::ConstEnum | includes, SymbolFlags::ConstEnumExcludes)
        } else {
            (SymbolFlags::RegularEnum | includes, SymbolFlags::RegularEnumExcludes)
        };
        let symbol_id = builder.declare_symbol(self.id.span, &self.id.name, includes, excludes);
        self.id.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> Binder<'a> for TSEnumMember<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        builder.declare_symbol(
            self.span,
            self.id.static_name().as_str(),
            SymbolFlags::EnumMember,
            SymbolFlags::EnumMemberExcludes,
        );
    }
}

impl<'a> Binder<'a> for TSModuleDeclaration<'a> {
    fn bind(&self, builder: &mut SemanticBuilder<'a>) {
        let TSModuleDeclarationName::Identifier(id) = &self.id else { return };
        let instantiated =
            get_module_instance_state(builder, self, builder.current_node_id).is_instantiated();
        let (mut includes, excludes) = if instantiated {
            (SymbolFlags::ValueModule, SymbolFlags::ValueModuleExcludes)
        } else {
            (SymbolFlags::NamespaceModule, SymbolFlags::NamespaceModuleExcludes)
        };

        if self.declare {
            includes |= SymbolFlags::Ambient;
        }
        let symbol_id = builder.declare_symbol(id.span, &id.name, includes, excludes);

        id.set_symbol_id(symbol_id);
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ModuleInstanceState {
    NonInstantiated,
    Instantiated,
    ConstEnumOnly,
}

impl ModuleInstanceState {
    fn is_instantiated(self) -> bool {
        self != ModuleInstanceState::NonInstantiated
    }
}

/// Determines if a module is instantiated or not.
///
/// Based on `https://github.com/microsoft/TypeScript/blob/15392346d05045742e653eab5c87538ff2a3c863/src/compiler/binder.ts#L342-L474`
fn get_module_instance_state<'a>(
    builder: &mut SemanticBuilder<'a>,
    decl: &TSModuleDeclaration<'a>,
    current_node_id: NodeId,
) -> ModuleInstanceState {
    get_module_instance_state_impl(builder, decl, current_node_id, &mut Vec::new())
}

fn get_module_instance_state_impl<'a, 'b>(
    builder: &mut SemanticBuilder<'a>,
    decl: &'b TSModuleDeclaration<'a>,
    current_node_id: NodeId,
    module_declaration_stmts: &mut Vec<&'b Statement<'a>>,
) -> ModuleInstanceState {
    // `SemanticBuilder` takes an immutable reference to AST, so `unstable_address` produces stable `Address`es
    let address = decl.unstable_address();

    if let Some(state) = builder.module_instance_state_cache.get(&address) {
        return *state;
    }

    let Some(body) = &decl.body else {
        // For modules without a block, we consider them instantiated
        return ModuleInstanceState::Instantiated;
    };

    // A module is uninstantiated if it contains only specific declarations
    let state = match body {
        TSModuleDeclarationBody::TSModuleBlock(block) => {
            module_declaration_stmts.extend(block.body.iter());

            let mut child_state = ModuleInstanceState::NonInstantiated;
            for stmt in &block.body {
                child_state = get_module_instance_state_for_statement(
                    builder,
                    stmt,
                    current_node_id,
                    module_declaration_stmts,
                );
                if child_state.is_instantiated() {
                    break;
                }
            }
            child_state
        }
        TSModuleDeclarationBody::TSModuleDeclaration(module) => {
            get_module_instance_state(builder, module, current_node_id)
        }
    };

    builder.module_instance_state_cache.insert(address, state);
    state
}

fn get_module_instance_state_for_statement<'a, 'b>(
    builder: &mut SemanticBuilder<'a>,
    stmt: &'b Statement<'a>,
    current_node_id: NodeId,
    module_declaration_stmts: &mut Vec<&'b Statement<'a>>,
) -> ModuleInstanceState {
    let address = stmt.address();
    if let Some(state) = builder.module_instance_state_cache.get(&address) {
        return *state;
    }

    let state = match stmt {
            // 1. interface declarations, type alias declarations
            Statement::TSInterfaceDeclaration(_)
            | Statement::TSTypeAliasDeclaration(_)
            // 3. non-exported import declarations
            | Statement::TSImportEqualsDeclaration(_) => {
                ModuleInstanceState::NonInstantiated
            }
            // 2. const enum declarations
            Statement::TSEnumDeclaration(enum_decl) => {
                if enum_decl.r#const {
                    ModuleInstanceState::ConstEnumOnly
                } else {
                    ModuleInstanceState::Instantiated
                }
            }
            Statement::ExportDefaultDeclaration(export_decl)  => {
                if matches!(export_decl.declaration, ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)) {
                    ModuleInstanceState::NonInstantiated
                } else {
                    ModuleInstanceState::Instantiated
                }
            }
            Statement::ExportNamedDeclaration(export_decl) if export_decl.declaration.is_some() => {
                match export_decl.declaration.as_ref().unwrap() {
                    Declaration::TSModuleDeclaration(module_decl) => {
                        get_module_instance_state_impl(builder, module_decl, current_node_id, module_declaration_stmts)
                    }
                    decl => if decl.is_type() {
                        ModuleInstanceState::NonInstantiated
                    } else {
                        ModuleInstanceState::Instantiated
                    }
                }
            }
            // 4. Export alias declarations pointing at uninstantiated modules
            Statement::ExportNamedDeclaration(export_decl) => {
                if export_decl.source.is_none() {
                    let mut export_state = ModuleInstanceState::NonInstantiated;
                    for specifier in &export_decl.specifiers {
                        export_state = get_module_instance_state_for_alias_target(builder, specifier, current_node_id, module_declaration_stmts.as_slice());
                        if export_state.is_instantiated() {
                            break;
                        }
                    }
                    export_state
                } else {
                    ModuleInstanceState::Instantiated
                }
            }
            // 5. other module declarations
            Statement::TSModuleDeclaration(module_decl) => {
                get_module_instance_state_impl(builder, module_decl, current_node_id, module_declaration_stmts)
            }
            // Any other type of statement means the module is instantiated
            _ => ModuleInstanceState::Instantiated,
        };

    builder.module_instance_state_cache.insert(address, state);
    state
}

// `module_declaration_stmts` is stored statements that are collected from the all ModuleBlocks.
// The reason we need to collect and pass in this method is that we need to check export specifiers
// whether they refer to a declaration that declared in the module block or not. And we can't use
// `self.nodes.node(node_id)` to get the nested module block's statements since the child ModuleBlock
// AstNode hasn't created yet.
fn get_module_instance_state_for_alias_target<'a>(
    builder: &mut SemanticBuilder<'a>,
    specifier: &ExportSpecifier<'a>,
    mut current_node_id: NodeId,
    module_declaration_stmts: &[&Statement<'a>],
) -> ModuleInstanceState {
    let ModuleExportName::IdentifierReference(local) = &specifier.local else {
        return ModuleInstanceState::Instantiated;
    };

    let name = local.name;
    let mut current_block_stmts = module_declaration_stmts.to_vec();
    loop {
        let mut found = false;
        for stmt in &current_block_stmts {
            match stmt {
                Statement::VariableDeclaration(decl) => {
                    decl.bound_names(&mut |id| {
                        if id.name == name {
                            found = true;
                        }
                    });
                }
                match_declaration!(Statement) => {
                    if stmt.to_declaration().id().is_some_and(|id| id.name == name) {
                        found = true;
                    }
                }
                Statement::ExportNamedDeclaration(decl) => match decl.declaration.as_ref() {
                    Some(Declaration::VariableDeclaration(decl)) => {
                        decl.bound_names(&mut |id| {
                            if id.name == name {
                                found = true;
                            }
                        });
                    }
                    Some(decl) => {
                        if decl.id().is_some_and(|id| id.name == name) {
                            found = true;
                        }
                    }
                    None => {
                        continue;
                    }
                },
                Statement::ExportDefaultDeclaration(decl) => match &decl.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(decl) => {
                        if decl.id.as_ref().is_some_and(|id| id.name == name) {
                            found = true;
                        }
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(decl) => {
                        if decl.id.as_ref().is_some_and(|id| id.name == name) {
                            found = true;
                        }
                    }
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(decl) => {
                        if decl.id.name == name {
                            found = true;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            if found {
                if matches!(stmt, Statement::TSImportEqualsDeclaration(_)) {
                    // Treat re-exports of import aliases as instantiated,
                    // since they're ambiguous. This is consistent with
                    // `export import x = mod.x` being treated as instantiated:
                    //   import x = mod.x;
                    //   export { x };
                    return ModuleInstanceState::Instantiated;
                }
                return get_module_instance_state_for_statement(
                    builder,
                    stmt,
                    current_node_id,
                    &mut Vec::default(), // No need to check export specifier
                );
            }
        }

        let Some(node) = builder.nodes.ancestors(current_node_id).find(|node| {
            matches!(
                node.kind(),
                AstKind::Program(_) | AstKind::TSModuleBlock(_) | AstKind::BlockStatement(_)
            )
        }) else {
            break;
        };

        current_node_id = node.id();
        current_block_stmts.clear();
        // Didn't find the declaration whose name matches export specifier
        // in the current block, so we need to check the parent block.
        current_block_stmts.extend(match node.kind() {
            AstKind::Program(program) => program.body.iter(),
            AstKind::TSModuleBlock(block) => block.body.iter(),
            AstKind::BlockStatement(block) => block.body.iter(),
            _ => unreachable!(),
        });
    }

    // Not found in any of the statements
    ModuleInstanceState::Instantiated
}

impl<'a> Binder<'a> for TSTypeParameter<'a> {
    fn bind(&self, builder: &mut SemanticBuilder) {
        let scope_id = if matches!(
            builder.nodes.parent_kind(builder.current_node_id),
            AstKind::TSInferType(_)
        ) {
            builder
                .scoping
                .scope_ancestors(builder.current_scope_id)
                .find(|scope_id| builder.scoping.scope_flags(*scope_id).is_ts_conditional())
        } else {
            None
        };

        let symbol_id = builder.declare_symbol_on_scope(
            self.name.span,
            &self.name.name,
            scope_id.unwrap_or(builder.current_scope_id),
            SymbolFlags::TypeParameter,
            SymbolFlags::TypeParameterExcludes,
        );
        self.name.symbol_id.set(Some(symbol_id));
    }
}
