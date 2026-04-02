use oxc_span::CompactStr;
use oxc_syntax::symbol::SymbolId;
use oxc_types::{
    LiteralType, ObjectFlags, PropertyInfo, StructuredType, StructuredTypeKind, TypeData,
    TypeFlags, TypeId, sort_properties,
};
use smallvec::SmallVec;

use crate::checker::CheckMode;
use crate::Checker;

impl Checker<'_> {
    /// Get the type of a global identifier in value/expression position.
    ///
    /// Prefers the value-side type (from `declare var` / `declare function` in
    /// lib.d.ts), falls back to type-side (interface), then `any`.
    pub(crate) fn get_type_of_global_identifier(&self, name: &str) -> TypeId {
        match name {
            "undefined" => self.undefined_type,
            _ => {
                // Prefer value-side (declare var, declare function)
                if let Some(t) = self.host.get_global_value_type(name) {
                    return t;
                }
                // Fallback to type-side (interface) — better than any
                self.get_global_type(name)
            }
        }
    }

    /// Get the type of a symbol, with caching and cycle detection.
    ///
    /// On first call, resolves the symbol's type from its declaration and caches
    /// the result. Subsequent calls return the cached type. If the symbol is
    /// already being resolved (circular reference), returns `any_type`.
    ///
    /// Mirrors tsgo's `getTypeOfSymbol` with `valueSymbolLinks.resolvedType`
    /// caching and `pushTypeResolution`/`popTypeResolution` cycle detection.
    pub fn get_type_of_symbol(&mut self, symbol_id: SymbolId) -> TypeId {
        // Check cache (IndexVec: O(1) array indexing)
        if let Some(cached) = self.symbol_type_cache[symbol_id] {
            return cached;
        }

        // Cycle detection: if this symbol is already being resolved, break cycle
        if !self.resolving_symbols.insert(symbol_id) {
            return self.any_type;
        }

        // Resolve, remove from resolution set, cache
        let result = self.resolve_symbol_type(symbol_id);
        self.resolving_symbols.remove(&symbol_id);
        self.symbol_type_cache[symbol_id] = Some(result);
        result
    }

    /// Resolve the type of a symbol from its declaration.
    ///
    /// This is the uncached inner logic — callers should use `get_type_of_symbol`.
    fn resolve_symbol_type(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;

        // Import binding — resolve value-side via host (cross-file)
        let symbol_flags = self.semantic().scoping().symbol_flags(symbol_id);
        if symbol_flags.is_import() {
            return self.resolve_import_as_value(symbol_id);
        }

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        match node.kind() {
            AstKind::VariableDeclarator(decl) => {
                // Check if this is a destructuring pattern
                if !matches!(decl.id, oxc_ast::ast::BindingPattern::BindingIdentifier(_)) {
                    // Destructuring: get overall type, then extract this symbol's portion
                    let overall_type = if let Some(annotation) = &decl.type_annotation {
                        self.get_type_from_type_node(&annotation.type_annotation)
                    } else if let Some(init) = &decl.init {
                        self.get_type_of_expression(init, None, CheckMode::TYPE_ONLY)
                    } else {
                        // May be in a for-of/for-in loop
                        self.get_type_from_for_loop_context(node_id)
                    };
                    let resolved = self
                        .resolve_destructured_binding_type(&decl.id, overall_type, symbol_id)
                        .unwrap_or(self.any_type);
                    // Widen literal types for let/var destructured bindings
                    return if decl.kind != oxc_ast::ast::VariableDeclarationKind::Const {
                        self.get_widened_literal_type(resolved)
                    } else {
                        resolved
                    };
                }
                if let Some(annotation) = &decl.type_annotation {
                    self.get_type_from_type_node(&annotation.type_annotation)
                } else if let Some(init) = &decl.init {
                    let inferred = self.get_type_of_expression(init, None, CheckMode::TYPE_ONLY);
                    // Widen literal types for non-const declarations
                    if decl.kind != oxc_ast::ast::VariableDeclarationKind::Const {
                        self.get_widened_literal_type(inferred)
                    } else {
                        inferred
                    }
                } else {
                    // No annotation and no initializer — check for for-of/for-in context
                    self.get_type_from_for_loop_context(node_id)
                }
            }
            AstKind::FormalParameter(param) => {
                if let Some(annotation) = &param.type_annotation {
                    self.get_type_from_type_node(&annotation.type_annotation)
                } else {
                    self.any_type
                }
            }
            AstKind::Class(class) => {
                // The value type of a class is the constructor — an anonymous
                // object type with the class symbol attached. Displays as
                // "typeof C". Includes static members as properties and a
                // construct signature that returns the instance type.
                let mut properties = Vec::new();
                let mut construct_signatures = Vec::new();

                for element in &class.body.body {
                    use oxc_ast::ast::ClassElement;
                    match element {
                        ClassElement::PropertyDefinition(prop) => {
                            if !prop.r#static {
                                continue; // instance props go on the instance type
                            }
                            if let Some(name) = prop.key.static_name() {
                                let prop_type = if let Some(ann) = &prop.type_annotation {
                                    self.get_type_from_type_node(&ann.type_annotation)
                                } else if let Some(init) = &prop.value {
                                    self.get_type_of_expression(init, None, CheckMode::TYPE_ONLY)
                                } else {
                                    self.any_type
                                };
                                properties.push(PropertyInfo {
                                    name: CompactStr::new(&name),
                                    type_id: prop_type,
                                    optional: prop.optional,
                                    readonly: prop.readonly,
                                    decl_order: 0,
                                });
                            }
                        }
                        ClassElement::MethodDefinition(method) => {
                            if method.r#static {
                                // Static method → property on the constructor type
                                if let Some(name) = method.key.static_name() {
                                    let sig = self.build_signature_from_function(&method.value);
                                    let method_type = self.create_function_type(sig);
                                    properties.push(PropertyInfo::new(
                                        CompactStr::new(&name),
                                        method_type,
                                    ));
                                }
                            } else if method.kind == oxc_ast::ast::MethodDefinitionKind::Constructor
                            {
                                // Constructor → construct signature returning the instance type
                                let instance_type = self.get_declared_type_of_symbol(symbol_id);
                                let mut sig = self.build_signature_from_function(&method.value);
                                sig.return_type = instance_type;
                                construct_signatures.push(sig);
                            }
                        }
                        _ => {}
                    }
                }

                // If no explicit constructor, add a default construct signature
                if construct_signatures.is_empty() {
                    let instance_type = self.get_declared_type_of_symbol(symbol_id);
                    construct_signatures.push(oxc_types::Signature {
                        flags: oxc_types::SignatureFlags::None,
                        parameters: Vec::new(),
                        return_type: instance_type,
                        type_parameters: SmallVec::new(),
                        min_argument_count: 0,
                    });
                }

                sort_properties(&mut properties);
                self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Anonymous,
                    TypeData::Structured(Box::new(StructuredType {
                        properties,
                        string_index_type: None,
                        number_index_type: None,
                        call_signatures: Vec::new(),
                        construct_signatures,
                        kind: StructuredTypeKind::Anonymous { target: None },
                    })),
                    Some((self.file_idx, symbol_id)),
                )
            }
            AstKind::TSModuleDeclaration(decl) => {
                // The value type of a namespace is a structured object with
                // properties for each exported value declaration.
                // Similar to enum value types above.
                self.get_namespace_value_type(decl, symbol_id)
            }
            AstKind::TSEnumDeclaration(decl) => {
                // The value type of an enum is the namespace object with member
                // properties. Displays as "typeof E". For binding identifiers,
                // the conformance runner uses get_declared_type_of_symbol which
                // returns the enum union type displayed as "E".
                let mut properties = Vec::new();
                let mut auto_value: f64 = 0.0;
                for member in &decl.body.members {
                    let name = member.id.static_name();
                    let member_type = self.compute_enum_member_value(member, &mut auto_value);
                    properties.push(PropertyInfo::new(CompactStr::new(&name), member_type));
                }
                sort_properties(&mut properties);
                self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Anonymous,
                    TypeData::Structured(Box::new(StructuredType {
                        properties,
                        string_index_type: None,
                        number_index_type: None,
                        call_signatures: Vec::new(),
                        construct_signatures: Vec::new(),
                        kind: StructuredTypeKind::Anonymous { target: None },
                    })),
                    Some((self.file_idx, symbol_id)),
                )
            }
            AstKind::TSEnumMember(member) => {
                // Individual enum member: resolve its literal type
                if let Some(init) = &member.initializer {
                    self.get_type_of_expression(init, None, CheckMode::TYPE_ONLY)
                } else {
                    // Auto-incremented numeric member — walk the parent enum body
                    // to compute the auto-incremented value.
                    let target_node_id = member.node_id.get();
                    let parent_id = self.semantic().nodes().parent_id(node_id);
                    let parent = self.semantic().nodes().get_node(parent_id);
                    if let oxc_ast::AstKind::TSEnumBody(body) = parent.kind() {
                        let mut auto_value: f64 = 0.0;
                        for m in &body.members {
                            let t = self.compute_enum_member_value(m, &mut auto_value);
                            if m.node_id.get() == target_node_id {
                                return t;
                            }
                        }
                    }
                    self.any_type
                }
            }
            AstKind::Function(func) => {
                let sig = self.build_signature_from_function(func);
                self.create_function_type(sig)
            }
            _ => {
                // For merged symbols (e.g., interface + declare var), the primary
                // declaration may be the type-side node. Search redeclarations
                // for a value-side declaration.
                self.resolve_value_from_redeclarations(symbol_id)
            }
        }
    }

    /// Search a symbol's redeclarations for a value-side declaration node.
    ///
    /// When a symbol has merged declarations (e.g., `interface RegExp` +
    /// `declare var RegExp: RegExpConstructor`), `symbol_declaration()` returns
    /// the first declaration which may be the type-side node. This method
    /// searches all redeclarations for one with value flags and resolves its type.
    fn resolve_value_from_redeclarations(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;
        use oxc_syntax::symbol::SymbolFlags;

        // Extract only (NodeId, flags) to release the borrow on semantic
        // before entering mutable type resolution below.
        let value_decl_node = self
            .semantic()
            .scoping()
            .symbol_redeclarations(symbol_id)
            .iter()
            .find(|r| r.flags.intersects(SymbolFlags::Variable | SymbolFlags::Function))
            .map(|r| r.declaration);

        let Some(node_id) = value_decl_node else {
            return self.any_type;
        };

        let node = self.semantic().nodes().get_node(node_id);
        match node.kind() {
            AstKind::VariableDeclarator(decl) => {
                if let Some(annotation) = &decl.type_annotation {
                    return self.get_type_from_type_node(&annotation.type_annotation);
                }
                if let Some(init) = &decl.init {
                    return self.get_type_of_expression(init, None, CheckMode::TYPE_ONLY);
                }
                self.any_type
            }
            AstKind::Function(func) => {
                let sig = self.build_signature_from_function(func);
                self.create_function_type(sig)
            }
            _ => self.any_type,
        }
    }

    /// Compute the type of an enum member, updating the auto-increment counter.
    pub(crate) fn compute_enum_member_value(
        &mut self,
        member: &oxc_ast::ast::TSEnumMember<'_>,
        auto_value: &mut f64,
    ) -> TypeId {
        if let Some(init) = &member.initializer {
            let t = self.get_type_of_expression(init, None, CheckMode::TYPE_ONLY);
            if let TypeData::Literal(LiteralType::Number(n)) = self.type_arena.get_data(t) {
                *auto_value = *n + 1.0;
            }
            t
        } else {
            let t = self.get_or_create_number_literal_type(*auto_value);
            *auto_value += 1.0;
            t
        }
    }

    /// Build the value type of a namespace/module declaration.
    ///
    /// Iterates through the body's statements, collecting exported value
    /// declarations (variables, functions, classes, enums, nested namespaces)
    /// as properties of a structured object type.
    fn get_namespace_value_type(
        &mut self,
        decl: &oxc_ast::ast::TSModuleDeclaration<'_>,
        symbol_id: SymbolId,
    ) -> TypeId {
        use oxc_ast::ast::{Statement, TSModuleDeclarationBody};

        let mut properties = Vec::new();

        let Some(body) = &decl.body else {
            // Ambient module with no body (e.g., `declare module "foo";`)
            return self.type_arena.new_type(
                TypeFlags::Object,
                ObjectFlags::Anonymous,
                TypeData::Structured(Box::new(StructuredType {
                    properties,
                    string_index_type: None,
                    number_index_type: None,
                    call_signatures: Vec::new(),
                    construct_signatures: Vec::new(),
                    kind: StructuredTypeKind::Anonymous { target: None },
                })),
                Some((self.file_idx, symbol_id)),
            );
        };

        match body {
            TSModuleDeclarationBody::TSModuleBlock(block) => {
                for stmt in &block.body {
                    // In a namespace body, exported declarations appear as
                    // ExportNamedDeclaration wrapping a Declaration.
                    if let Statement::ExportNamedDeclaration(export) = stmt {
                        if let Some(inner_decl) = &export.declaration {
                            self.collect_namespace_properties_from_declaration(
                                inner_decl,
                                &mut properties,
                            );
                        }
                    }
                }
            }
            TSModuleDeclarationBody::TSModuleDeclaration(inner) => {
                // Dotted name: `namespace A.B { ... }` — the inner module
                // becomes a single property on the outer namespace.
                let name = inner.id.name();
                let inner_type = self.get_namespace_value_type(inner, symbol_id);
                properties.push(PropertyInfo::new(CompactStr::new(&name), inner_type));
            }
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous,
            TypeData::Structured(Box::new(StructuredType {
                properties,
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            })),
            Some((self.file_idx, symbol_id)),
        )
    }

    /// Collect property entries from a declaration inside a namespace body.
    fn collect_namespace_properties_from_declaration(
        &mut self,
        decl: &oxc_ast::ast::Declaration<'_>,
        properties: &mut Vec<PropertyInfo>,
    ) {
        use oxc_ast::ast::Declaration;

        match decl {
            Declaration::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id {
                        if let Some(sid) = id.symbol_id.get() {
                            let prop_type = self.get_type_of_symbol(sid);
                            properties
                                .push(PropertyInfo::new(CompactStr::new(&id.name), prop_type));
                        }
                    }
                }
            }
            Declaration::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    if let Some(sid) = id.symbol_id.get() {
                        let func_type = self.get_type_of_symbol(sid);
                        properties.push(PropertyInfo::new(CompactStr::new(&id.name), func_type));
                    }
                }
            }
            Declaration::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    if let Some(sid) = id.symbol_id.get() {
                        let class_type = self.get_type_of_symbol(sid);
                        properties.push(PropertyInfo::new(CompactStr::new(&id.name), class_type));
                    }
                }
            }
            Declaration::TSEnumDeclaration(enum_decl) => {
                if let Some(sid) = enum_decl.id.symbol_id.get() {
                    let enum_type = self.get_type_of_symbol(sid);
                    properties
                        .push(PropertyInfo::new(CompactStr::new(&enum_decl.id.name), enum_type));
                }
            }
            Declaration::TSModuleDeclaration(inner_ns) => {
                let name = inner_ns.id.name();
                if let oxc_ast::ast::TSModuleDeclarationName::Identifier(id) = &inner_ns.id {
                    if let Some(sid) = id.symbol_id.get() {
                        let ns_type = self.get_type_of_symbol(sid);
                        properties.push(PropertyInfo::new(CompactStr::new(&name), ns_type));
                    }
                }
            }
            // Type-only declarations (interfaces, type aliases) don't contribute
            // to the namespace value type.
            _ => {}
        }
    }

    /// Check if a VariableDeclarator is inside a for-of or for-in statement.
    /// If so, return the iterated element type; otherwise return `any_type`.
    pub(crate) fn get_type_from_for_loop_context(
        &mut self,
        declarator_node_id: oxc_syntax::node::NodeId,
    ) -> TypeId {
        use oxc_ast::AstKind;
        // Walk up: VariableDeclarator → VariableDeclaration → ForOfStatement/ForInStatement
        // Capture the loop node ID without holding a mutable borrow.
        let loop_node_id = 'search: {
            for ancestor in self.semantic().nodes().ancestors(declarator_node_id) {
                match ancestor.kind() {
                    AstKind::ForOfStatement(_) | AstKind::ForInStatement(_) => {
                        break 'search Some(ancestor.id());
                    }
                    AstKind::VariableDeclarator(_) | AstKind::VariableDeclaration(_) => continue,
                    _ => break,
                }
            }
            None
        };
        let Some(loop_node_id) = loop_node_id else {
            return self.any_type;
        };
        let node = self.semantic().nodes().get_node(loop_node_id);
        match node.kind() {
            AstKind::ForOfStatement(for_of) => {
                let iterable_type = self.get_type_of_expression(&for_of.right, None, CheckMode::TYPE_ONLY);
                self.get_iterated_type_of_iterable(iterable_type)
            }
            AstKind::ForInStatement(_) => self.string_type,
            _ => unreachable!(),
        }
    }

    /// Get the iterated element type from an iterable type.
    /// For Array<T>, returns T. For tuples, returns union of element types.
    pub(crate) fn get_iterated_type_of_iterable(&mut self, type_id: TypeId) -> TypeId {
        // Check Array<T> → T
        if let TypeData::TypeReference(tr) = self.type_arena.get_data(type_id) {
            if tr.target == Some(self.array_type) && !tr.resolved_type_arguments.is_empty() {
                return tr.resolved_type_arguments[0];
            }
            return self.any_type;
        }
        // Collect tuple element types (releases the type_arena borrow before union creation)
        let tuple_element_types = if let TypeData::Tuple(tuple) = self.type_arena.get_data(type_id)
        {
            Some(tuple.element_infos.iter().map(|e| e.element_type).collect::<Vec<_>>())
        } else {
            None
        };
        if let Some(types) = tuple_element_types {
            if types.is_empty() {
                return self.never_type;
            }
            return self.get_or_create_union_type(types);
        }
        self.any_type
    }

    /// Walk a destructuring pattern to find the binding matching `target_symbol`,
    /// extracting the corresponding property/element type from `init_type`.
    pub(crate) fn resolve_destructured_binding_type(
        &mut self,
        pattern: &oxc_ast::ast::BindingPattern<'_>,
        init_type: TypeId,
        target_symbol: SymbolId,
    ) -> Option<TypeId> {
        use oxc_ast::ast::BindingPattern;
        match pattern {
            BindingPattern::BindingIdentifier(ident) => {
                if ident.symbol_id.get() == Some(target_symbol) { Some(init_type) } else { None }
            }
            BindingPattern::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    if let Some(name) = prop.key.static_name() {
                        // TODO: should emit an error or return `undefined` when property doesn't exist
                        let prop_type =
                            self.get_property_of_type(init_type, &name).unwrap_or(self.any_type);
                        if let Some(result) = self.resolve_destructured_binding_type(
                            &prop.value,
                            prop_type,
                            target_symbol,
                        ) {
                            return Some(result);
                        }
                    }
                }
                if let Some(rest) = &obj.rest {
                    if let Some(result) = self.resolve_destructured_binding_type(
                        &rest.argument,
                        self.any_type,
                        target_symbol,
                    ) {
                        return Some(result);
                    }
                }
                None
            }
            BindingPattern::ArrayPattern(arr) => {
                for (i, element) in arr.elements.iter().enumerate() {
                    if let Some(element) = element {
                        let elem_type = self.get_element_type_at_index(init_type, i);
                        if let Some(result) = self.resolve_destructured_binding_type(
                            element,
                            elem_type,
                            target_symbol,
                        ) {
                            return Some(result);
                        }
                    }
                }
                if let Some(rest) = &arr.rest {
                    if let Some(result) = self.resolve_destructured_binding_type(
                        &rest.argument,
                        self.any_type,
                        target_symbol,
                    ) {
                        return Some(result);
                    }
                }
                None
            }
            BindingPattern::AssignmentPattern(assign) => {
                // Default value doesn't change the declared type
                self.resolve_destructured_binding_type(&assign.left, init_type, target_symbol)
            }
        }
    }

    /// Get the element type at a specific index from an array or tuple type.
    fn get_element_type_at_index(&self, type_id: TypeId, index: usize) -> TypeId {
        match self.type_arena.get_data(type_id) {
            TypeData::Tuple(tuple) => {
                if index < tuple.element_infos.len() {
                    tuple.element_infos[index].element_type
                } else {
                    self.any_type
                }
            }
            TypeData::TypeReference(tr) => {
                // Array<T> → element type is T
                if let Some(target) = tr.target {
                    if target == self.array_type && !tr.resolved_type_arguments.is_empty() {
                        return tr.resolved_type_arguments[0];
                    }
                }
                self.any_type
            }
            _ => self.any_type,
        }
    }

    /// Resolve an import binding to its `ExportedBinding` via the host.
    ///
    /// Walks from the import specifier's declaration node up to its parent
    /// ImportDeclaration to extract the module specifier and import name,
    /// then calls `host.resolve_import()`.
    fn resolve_import_binding(
        &self,
        symbol_id: SymbolId,
    ) -> Option<oxc_checker_host::ExportedBinding> {
        use oxc_ast::AstKind;

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        // Extract the imported name from the specifier
        let export_name = match node.kind() {
            AstKind::ImportSpecifier(spec) => spec.imported.name().to_string(),
            AstKind::ImportDefaultSpecifier(_) => "default".to_string(),
            _ => return None,
        };

        // Walk up to the ImportDeclaration to get the module specifier
        let parent_id = self.semantic().nodes().parent_id(node_id);
        let parent = self.semantic().nodes().get_node(parent_id);
        let AstKind::ImportDeclaration(import_decl) = parent.kind() else {
            return None;
        };

        let module_specifier = import_decl.source.value.as_str();
        self.host.resolve_import(&self.file_path, module_specifier, &export_name)
    }

    /// Resolve an import binding's value-side type.
    /// Called from `resolve_symbol_type` (value context).
    pub(crate) fn resolve_import_as_value(&mut self, symbol_id: SymbolId) -> TypeId {
        self.resolve_import_binding(symbol_id).and_then(|b| b.value_type).unwrap_or(self.any_type)
    }

    /// Resolve an import binding's type-side type.
    /// Called from `resolve_declared_type` (type context).
    pub(crate) fn resolve_import_as_type(&mut self, symbol_id: SymbolId) -> TypeId {
        self.resolve_import_binding(symbol_id).and_then(|b| b.type_type).unwrap_or(self.any_type)
    }
}
