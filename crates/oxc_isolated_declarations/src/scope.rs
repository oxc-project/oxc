use oxc_allocator::{Allocator, Vec};
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::AstBuilder;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{visit::walk::*, Visit};
use oxc_span::Atom;
use oxc_syntax::scope::ScopeFlags;
use rustc_hash::FxHashSet;

pub struct ScopeTree<'a> {
    type_bindings: Vec<'a, FxHashSet<Atom<'a>>>,
    value_bindings: Vec<'a, FxHashSet<Atom<'a>>>,
    type_references: Vec<'a, FxHashSet<Atom<'a>>>,
    value_references: Vec<'a, FxHashSet<Atom<'a>>>,
    flags: Vec<'a, ScopeFlags>,
}

impl<'a> ScopeTree<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        let mut scope = Self {
            type_bindings: ast.new_vec(),
            value_bindings: ast.new_vec(),
            type_references: ast.new_vec(),
            value_references: ast.new_vec(),
            flags: ast.new_vec(),
        };
        scope.enter_scope(ScopeFlags::Top);
        scope
    }

    pub fn is_ts_module_block_flag(&self) -> bool {
        self.flags.last().unwrap().contains(ScopeFlags::TsModuleBlock)
    }

    pub fn has_reference(&self, name: &Atom<'a>) -> bool {
        self.value_references.last().is_some_and(|rs| rs.contains(name))
            || self.type_references.last().is_some_and(|rs| rs.contains(name))
    }

    pub fn references_len(&self) -> usize {
        self.value_references.last().unwrap().len() + self.type_references.last().unwrap().len()
    }

    fn add_value_binding(&mut self, ident: &Atom<'a>) {
        self.value_bindings.last_mut().unwrap().insert(ident.clone());
    }

    fn add_type_binding(&mut self, ident: &Atom<'a>) {
        self.type_bindings.last_mut().unwrap().insert(ident.clone());
    }

    fn add_value_reference(&mut self, ident: &Atom<'a>) {
        self.value_references.last_mut().unwrap().insert(ident.clone());
    }

    fn add_type_reference(&mut self, ident: &Atom<'a>) {
        self.type_references.last_mut().unwrap().insert(ident.clone());
    }

    /// resolve references in the current scope
    /// and merge unresolved references to the parent scope
    /// and remove the current scope
    fn resolve_references(&mut self) {
        let current_value_bindings = self.value_bindings.pop().unwrap_or_default();
        let current_value_references = self.value_references.pop().unwrap_or_default();
        self.type_references
            .last_mut()
            .unwrap()
            .extend(current_value_references.difference(&current_value_bindings).cloned());

        let current_type_bindings = self.type_bindings.pop().unwrap_or_default();
        let current_type_references = self.type_references.pop().unwrap_or_default();
        self.type_references
            .last_mut()
            .unwrap()
            .extend(current_type_references.difference(&current_type_bindings).cloned());
    }
}

impl<'a> Visit<'a> for ScopeTree<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags) {
        self.flags.push(flags);
        self.value_bindings.push(FxHashSet::default());
        self.type_bindings.push(FxHashSet::default());
        self.type_references.push(FxHashSet::default());
        self.value_references.push(FxHashSet::default());
    }

    fn leave_scope(&mut self) {
        self.resolve_references();
        self.flags.pop();
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.add_value_reference(&ident.name);
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.kind {
            self.add_value_binding(&ident.name);
        }
        walk_binding_pattern(self, pattern);
    }

    fn visit_ts_type_name(&mut self, name: &TSTypeName<'a>) {
        if let TSTypeName::IdentifierReference(ident) = name {
            self.add_type_reference(&ident.name);
        } else {
            walk_ts_type_name(self, name);
        }
    }

    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        if let Some(type_name) = ty.expr_name.as_ts_type_name() {
            let ident = TSTypeName::get_first_name(type_name);
            self.add_value_reference(&ident.name);
        } else {
            walk_ts_type_query(self, ty);
        }
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        for specifier in &decl.specifiers {
            if let ModuleExportName::Identifier(ident) = &specifier.local {
                self.add_type_reference(&ident.name);
                self.add_value_reference(&ident.name);
            }
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::Identifier(ident) = &decl.declaration {
            self.add_type_reference(&ident.name);
            self.add_value_reference(&ident.name);
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
                    self.add_value_binding(&id.name);
                }
            }
            Declaration::ClassDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.add_value_binding(&id.name);
                }
            }
            Declaration::TSTypeAliasDeclaration(decl) => {
                self.add_type_binding(&decl.id.name);
            }
            Declaration::TSInterfaceDeclaration(decl) => {
                self.add_type_binding(&decl.id.name);
            }
            Declaration::TSEnumDeclaration(decl) => {
                self.add_value_binding(&decl.id.name);
                self.add_type_binding(&decl.id.name);
            }
            Declaration::TSModuleDeclaration(decl) => {
                if let TSModuleDeclarationName::Identifier(ident) = &decl.id {
                    self.add_value_binding(&ident.name);
                    self.add_type_binding(&ident.name);
                }
            }
            Declaration::TSImportEqualsDeclaration(decl) => {
                self.add_value_binding(&decl.id.name);
            }
        }
        walk_declaration(self, declaration);
    }

    // ==================== TSTypeParameter ====================

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
        walk_ts_type_parameter_declaration(self, decl);
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

    fn visit_arrow_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        walk_arrow_expression(self, expr);
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

    /// `type D<T> = { [K in keyof T]: K };`
    ///             ^^^^^^^^^^^^^^^^^^^^
    ///                `K` is a type parameter
    /// We need to add `K` to the scope
    fn visit_ts_mapped_type(&mut self, ty: &TSMappedType<'a>) {
        // copy from walk_ts_mapped_type
        self.enter_scope(ScopeFlags::empty());
        self.add_type_binding(&ty.type_parameter.name.name);
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
        self.add_type_binding(&ty.type_parameter.name.name);
    }
}
