use oxc_allocator::{Allocator, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{visit::walk::*, Visit};
use oxc_span::Atom;
use oxc_syntax::scope::ScopeFlags;
use rustc_hash::FxHashSet;

/// Declaration scope.
#[derive(Debug)]
struct Scope<'a> {
    type_bindings: FxHashSet<Atom<'a>>,
    value_bindings: FxHashSet<Atom<'a>>,
    type_references: FxHashSet<Atom<'a>>,
    value_references: FxHashSet<Atom<'a>>,
    flags: ScopeFlags,
}

impl<'a> Scope<'a> {
    fn new(flags: ScopeFlags) -> Self {
        Self {
            value_bindings: FxHashSet::default(),
            type_bindings: FxHashSet::default(),
            type_references: FxHashSet::default(),
            value_references: FxHashSet::default(),
            flags,
        }
    }
}

/// Linear tree of declaration scopes.
pub struct ScopeTree<'a> {
    levels: Vec<'a, Scope<'a>>,
}

impl<'a> ScopeTree<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        let levels = ast.vec1(Scope::new(ScopeFlags::Top));
        Self { levels }
    }

    pub fn is_ts_module_block_flag(&self) -> bool {
        let scope = self.levels.last().unwrap();
        scope.flags.contains(ScopeFlags::TsModuleBlock)
    }

    pub fn has_reference(&self, name: &str) -> bool {
        // XXX(lucab): this should probably unwrap?
        let Some(scope) = self.levels.last() else { return false };
        scope.value_references.contains(name) || scope.type_references.contains(name)
    }

    pub fn references_len(&self) -> usize {
        let scope = self.levels.last().unwrap();
        scope.value_references.len() + scope.type_references.len()
    }

    fn add_value_binding(&mut self, ident: Atom<'a>) {
        let scope = self.levels.last_mut().unwrap();
        scope.value_bindings.insert(ident);
    }

    fn add_type_binding(&mut self, ident: Atom<'a>) {
        let scope = self.levels.last_mut().unwrap();
        scope.type_bindings.insert(ident);
    }

    fn add_value_reference(&mut self, ident: Atom<'a>) {
        let scope = self.levels.last_mut().unwrap();
        scope.value_references.insert(ident);
    }

    fn add_type_reference(&mut self, ident: Atom<'a>) {
        let scope = self.levels.last_mut().unwrap();
        scope.type_references.insert(ident);
    }

    /// Resolve references in the current scope, and propagate unresolved ones.
    fn resolve_references(&mut self) {
        debug_assert!(self.levels.len() >= 2);

        // Remove the current scope.
        let mut current_scope = self.levels.pop().unwrap();

        // Resolve references in the current scope.
        let current_value_bindings = current_scope.value_bindings;
        let current_value_references = current_scope.value_references;
        let val_diff = current_value_references.difference(&current_value_bindings).cloned();
        current_scope.type_references.extend(val_diff);
        let current_type_bindings = current_scope.type_bindings;
        let current_type_references = current_scope.type_references;
        let type_diff = current_type_references.difference(&current_type_bindings).cloned();

        // Merge unresolved references to the parent scope.
        self.levels.last_mut().unwrap().type_references.extend(type_diff);
    }
}

impl<'a> Visit<'a> for ScopeTree<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags) {
        let scope = Scope::new(flags);
        self.levels.push(scope);
    }

    fn leave_scope(&mut self) {
        self.resolve_references();
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.add_value_reference(ident.name.clone());
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.kind {
            self.add_value_binding(ident.name.clone());
        }
        walk_binding_pattern(self, pattern);
    }

    fn visit_ts_type_name(&mut self, name: &TSTypeName<'a>) {
        if let TSTypeName::IdentifierReference(ident) = name {
            self.add_type_reference(ident.name.clone());
        } else {
            walk_ts_type_name(self, name);
        }
    }

    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        if let Some(type_name) = ty.expr_name.as_ts_type_name() {
            let ident = TSTypeName::get_first_name(type_name);
            self.add_value_reference(ident.name.clone());
        } else {
            walk_ts_type_query(self, ty);
        }
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        for specifier in &decl.specifiers {
            if let Some(name) = specifier.local.identifier_name() {
                self.add_type_reference(name.clone());
                self.add_value_reference(name);
            }
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::Identifier(ident) = &decl.declaration {
            self.add_type_reference(ident.name.clone());
            self.add_value_reference(ident.name.clone());
        } else {
            walk_export_default_declaration(self, decl);
        }
    }

    fn visit_declaration(&mut self, declaration: &Declaration<'a>) {
        match declaration {
            Declaration::VariableDeclaration(_) | Declaration::UsingDeclaration(_) => {
                // add binding in BindingPattern
            }
            Declaration::FunctionDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.add_value_binding(id.name.clone());
                }
            }
            Declaration::ClassDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.add_value_binding(id.name.clone());
                }
            }
            Declaration::TSTypeAliasDeclaration(decl) => {
                self.add_type_binding(decl.id.name.clone());
            }
            Declaration::TSInterfaceDeclaration(decl) => {
                self.add_type_binding(decl.id.name.clone());
            }
            Declaration::TSEnumDeclaration(decl) => {
                self.add_value_binding(decl.id.name.clone());
                self.add_type_binding(decl.id.name.clone());
            }
            Declaration::TSModuleDeclaration(decl) => {
                if let TSModuleDeclarationName::Identifier(ident) = &decl.id {
                    self.add_value_binding(ident.name.clone());
                    self.add_type_binding(ident.name.clone());
                }
            }
            Declaration::TSImportEqualsDeclaration(decl) => {
                self.add_value_binding(decl.id.name.clone());
            }
        }
        walk_declaration(self, declaration);
    }

    // ==================== TSTypeParameter ====================

    fn visit_ts_type_parameter(&mut self, it: &TSTypeParameter<'a>) {
        self.add_type_binding(it.name.name.clone());
        if let Some(constraint) = &it.constraint {
            self.visit_ts_type(constraint);
        }
        if let Some(default) = &it.default {
            self.visit_ts_type(default);
        }
    }

    /// ```ts
    /// function foo<T>(x: T): T {
    ///             ^^^
    ///             `T` is a type parameter
    ///    return x;
    /// }
    /// ```
    /// We should create a new scope for TSTypeParameterDeclaration
    /// Because the type parameter is can be used in following nodes
    /// until the end of the function. So we leave the scope in the parent node (Function)
    fn visit_ts_type_parameter_declaration(&mut self, decl: &TSTypeParameterDeclaration<'a>) {
        self.enter_scope(ScopeFlags::empty());
        decl.params.iter().for_each(|param| self.visit_ts_type_parameter(param));
        // exit scope in parent AST node
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        walk_class(self, class);
        if class.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: Option<ScopeFlags>) {
        walk_function(self, func, flags);
        if func.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        walk_arrow_function_expression(self, expr);
        if expr.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        walk_ts_type_alias_declaration(self, decl);
        if decl.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        walk_ts_interface_declaration(self, decl);
        if decl.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_ts_call_signature_declaration(&mut self, signature: &TSCallSignatureDeclaration<'a>) {
        walk_ts_call_signature_declaration(self, signature);
        if signature.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_ts_method_signature(&mut self, signature: &TSMethodSignature<'a>) {
        walk_ts_method_signature(self, signature);
        if signature.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_ts_construct_signature_declaration(
        &mut self,
        signature: &TSConstructSignatureDeclaration<'a>,
    ) {
        walk_ts_construct_signature_declaration(self, signature);
        if signature.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    fn visit_ts_function_type(&mut self, signature: &TSFunctionType<'a>) {
        walk_ts_function_type(self, signature);
        if signature.type_parameters.is_some() {
            self.leave_scope();
        }
    }

    /// `type D = { [key in keyof T]: K };`
    ///             ^^^^^^^^^^^^^^^^^^^^
    /// We need to add both `T` and `K` to the scope
    fn visit_ts_mapped_type(&mut self, ty: &TSMappedType<'a>) {
        // copy from walk_ts_mapped_type
        self.enter_scope(ScopeFlags::empty());
        self.visit_ts_type_parameter(&ty.type_parameter);
        if let Some(name) = &ty.name_type {
            self.visit_ts_type(name);
        }
        if let Some(type_annotation) = &ty.type_annotation {
            self.visit_ts_type(type_annotation);
        }
        self.leave_scope();
    }

    /// `export type Flatten<Type> = Type extends Array<infer Item> ? Item : Type;`
    ///                                                ^^^^^^^^^^
    ///                                                  `Item` is a type parameter
    /// We need to add `Item` to the scope
    fn visit_conditional_expression(&mut self, expr: &ConditionalExpression<'a>) {
        self.enter_scope(ScopeFlags::empty());
        walk_conditional_expression(self, expr);
        self.leave_scope();
    }

    fn visit_ts_infer_type(&mut self, ty: &TSInferType<'a>) {
        // copy from walk_ts_infer_type
        self.visit_ts_type_parameter(&ty.type_parameter);
    }
}
