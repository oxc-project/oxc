use oxc_span::CompactStr;
use oxc_syntax::node::NodeId;
use oxc_syntax::symbol::SymbolId;
use oxc_types::{
    LiteralType, ObjectFlags, PropertyInfo, Signature, StructuredType, StructuredTypeKind,
    TypeData, TypeFlags, TypeId, TypeParameterType, sort_properties,
};
use smallvec::SmallVec;

use crate::Checker;
use crate::checker::CheckMode;

impl Checker<'_> {
    /// Get the declared type of a type-namespace symbol (type alias, interface,
    /// class, enum). Uses caching and cycle detection, mirroring tsgo's
    /// `getDeclaredTypeOfSymbol`.
    pub fn get_declared_type_of_symbol(&mut self, symbol_id: SymbolId) -> TypeId {
        // Check cache (IndexVec: O(1) array indexing)
        if let Some(cached) = self.caches.declared_type_cache[symbol_id] {
            return cached;
        }

        // Cycle detection
        if !self.resolving_symbols.insert(symbol_id) {
            return self.any_type;
        }

        let result = self.resolve_declared_type(symbol_id);
        self.resolving_symbols.remove(&symbol_id);
        self.caches.declared_type_cache[symbol_id] = Some(result);
        result
    }

    /// Resolve the declared type from a type-namespace declaration.
    fn resolve_declared_type(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;

        // Import binding — resolve type-side via host (cross-file)
        let symbol_flags = self.semantic().scoping().symbol_flags(symbol_id);
        if symbol_flags.is_import() {
            return self.resolve_import_as_type(symbol_id);
        }

        // Interface — may have multiple declarations that need merging.
        // Dispatched by flag rather than AST kind so that merged interface+var
        // symbols (where the primary declaration might be the var) are handled
        // correctly.
        if symbol_flags.is_interface() {
            return self.get_type_of_merged_interface(symbol_id);
        }

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        match node.kind() {
            AstKind::TSTypeAliasDeclaration(decl) => {
                // Resolve outer type parameters (e.g., T in `type Partial<T> = ...`)
                // so they exist in declared_type_cache for the body to reference.
                self.get_type_parameters_from_declaration(decl.type_parameters.as_deref());

                // Resolve the body — type parameter references within resolve
                // via declared_type_cache. Instantiation with concrete type args
                // happens later in maybe_instantiate_type_alias (type aliases are
                // transparent — no TypeReference wrapper).
                let body_type = self.get_type_from_type_node(&decl.type_annotation);

                // Attach the alias symbol so `type_to_string` can display the alias name.
                // Only attach when the body has no intrinsic name (no symbol) and no
                // existing alias. If the body already has a name (e.g., an interface
                // reference or another type alias), keep it — that name takes priority.
                if self.type_arena.get_symbol(body_type).is_none()
                    && self.type_arena.get_alias_symbol(body_type).is_none()
                {
                    self.type_arena.clone_type_with_alias(body_type, (self.file_idx, symbol_id))
                } else {
                    body_type
                }
            }
            AstKind::Class(decl) => self.get_type_of_class_declaration(decl, symbol_id),
            AstKind::TSEnumDeclaration(decl) => self.get_type_of_enum_declaration(decl, symbol_id),
            _ => self.any_type,
        }
    }

    /// Build an interface type from one or more same-file interface declarations.
    ///
    /// When a symbol has multiple interface declarations (e.g., `interface Foo { x: string }`
    /// + `interface Foo { y: number }`), this method merges all declarations into a single
    /// StructuredType with the combined properties, call signatures, index signatures,
    /// and base types.
    ///
    /// Type parameters from the first declaration are canonical. Subsequent declarations'
    /// type parameter SymbolIds are aliased to the first declaration's TypeParameter TypeIds
    /// so that body member types resolve correctly.
    ///
    /// Call signature ordering follows TypeScript semantics: later declarations' signatures
    /// come first in overload resolution.
    fn get_type_of_merged_interface(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;

        // Collect all interface declaration NodeIds into owned storage,
        // releasing the borrow on symbol_redeclarations.
        let redeclarations = self.semantic().scoping().symbol_redeclarations(symbol_id);
        let interface_node_ids: SmallVec<[NodeId; 4]> = if redeclarations.is_empty() {
            // No redeclarations — single declaration (common fast path)
            SmallVec::from_elem(self.semantic().scoping().symbol_declaration(symbol_id), 1)
        } else {
            // Filter to interface-only (skip `declare var` in merged interface+var symbols)
            redeclarations
                .iter()
                .filter(|r| r.flags.is_interface())
                .map(|r| r.declaration)
                .collect()
        };

        // Phase 1: Process type parameters from the first declaration.
        // These become the canonical TypeParameter TypeIds for the merged type.
        let first_node = self.semantic().nodes().get_node(interface_node_ids[0]);
        let first_decl = match first_node.kind() {
            AstKind::TSInterfaceDeclaration(decl) => decl,
            _ => return self.any_type,
        };
        let type_parameters =
            self.get_type_parameters_from_declaration(first_decl.type_parameters.as_deref());

        // Phase 2: Collect members from all declarations.
        let is_merged = interface_node_ids.len() > 1;
        let mut properties = Vec::new();
        // Track call signatures per-declaration only when merging (need reverse ordering).
        // For single declarations, collect directly into a flat vec.
        let mut per_decl_call_sigs: Vec<Vec<Signature>> = if is_merged {
            Vec::with_capacity(interface_node_ids.len())
        } else {
            Vec::new()
        };
        let mut flat_call_sigs: Vec<Signature> = Vec::new();
        let construct_signatures: Vec<Signature> = Vec::new();
        let mut string_index_type: Option<TypeId> = None;
        let mut number_index_type: Option<TypeId> = None;
        let mut resolved_base_types: SmallVec<[TypeId; 4]> = SmallVec::new();

        for (decl_idx, &node_id) in interface_node_ids.iter().enumerate() {
            let node = self.semantic().nodes().get_node(node_id);
            let AstKind::TSInterfaceDeclaration(decl) = node.kind() else {
                continue;
            };

            // For subsequent declarations, alias their type parameters to the
            // first declaration's canonical TypeParameter TypeIds. This ensures
            // that references to `T` in any declaration body resolve to the
            // same TypeParameter type.
            if decl_idx > 0 {
                let subsequent_arity = decl
                    .type_parameters
                    .as_ref()
                    .map_or(0, |tp| tp.params.len());
                if subsequent_arity != type_parameters.len() {
                    // Arity mismatch (includes generic vs non-generic) — skip
                    // this declaration's body. TS2428 diagnostic deferred.
                    continue;
                }
                // Alias subsequent type parameter SymbolIds to the canonical ones
                if let Some(tp_decl) = &decl.type_parameters {
                    for (i, param) in tp_decl.params.iter().enumerate() {
                        if let Some(sid) = param.name.symbol_id.get() {
                            self.caches.declared_type_cache[sid] = Some(type_parameters[i]);
                        }
                    }
                }
            }

            // Collect body members (properties, call signatures, index signatures)
            if is_merged {
                let mut decl_call_sigs = Vec::new();
                self.collect_interface_body_members(
                    &decl.body,
                    &mut properties,
                    &mut decl_call_sigs,
                    &mut string_index_type,
                    &mut number_index_type,
                );
                per_decl_call_sigs.push(decl_call_sigs);
            } else {
                self.collect_interface_body_members(
                    &decl.body,
                    &mut properties,
                    &mut flat_call_sigs,
                    &mut string_index_type,
                    &mut number_index_type,
                );
            }

            // Collect base types from extends clause.
            // Cycle detection via `resolving_symbols` in `get_declared_type_of_symbol`.
            for heritage in &decl.extends {
                let base_type = self.get_type_from_heritage_element(heritage);
                if base_type != self.any_type {
                    resolved_base_types.push(base_type);
                }
            }
        }

        // Merge call signatures: later declarations' signatures come first
        // (TypeScript overload resolution order).
        let call_signatures = if is_merged {
            per_decl_call_sigs.into_iter().rev().flatten().collect()
        } else {
            flat_call_sigs
        };

        sort_properties(&mut properties);
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Interface,
            TypeData::Structured(Box::new(StructuredType {
                properties,
                string_index_type,
                number_index_type,
                call_signatures,
                construct_signatures,
                kind: StructuredTypeKind::Interface {
                    target: None,
                    resolved_type_arguments: SmallVec::new(),
                    all_type_parameters: type_parameters,
                    this_type: None,
                    resolved_base_types,
                },
            })),
            Some((self.file_idx, symbol_id)),
        )
    }

    /// Collect properties, call signatures, and index signatures from an
    /// interface body. Used by `get_type_of_merged_interface` to process
    /// each declaration's body independently.
    fn collect_interface_body_members(
        &mut self,
        body: &oxc_ast::ast::TSInterfaceBody<'_>,
        properties: &mut Vec<PropertyInfo>,
        call_signatures: &mut Vec<Signature>,
        string_index_type: &mut Option<TypeId>,
        number_index_type: &mut Option<TypeId>,
    ) {
        use oxc_ast::ast::TSSignature;

        for sig in &body.body {
            match sig {
                TSSignature::TSPropertySignature(prop) => {
                    if let Some(name) = prop.key.static_name() {
                        let prop_type = if let Some(ann) = &prop.type_annotation {
                            self.get_type_from_type_node(&ann.type_annotation)
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
                TSSignature::TSCallSignatureDeclaration(call_sig) => {
                    let tp = self
                        .get_type_parameters_from_declaration(call_sig.type_parameters.as_deref());
                    let mut sig = self.build_signature_from_params(
                        &call_sig.params,
                        call_sig.return_type.as_deref(),
                    );
                    sig.type_parameters = tp;
                    call_signatures.push(sig);
                }
                TSSignature::TSMethodSignature(method) => {
                    if let Some(name) = method.key.static_name() {
                        let tp = self.get_type_parameters_from_declaration(
                            method.type_parameters.as_deref(),
                        );
                        let mut sig = self.build_signature_from_params(
                            &method.params,
                            method.return_type.as_deref(),
                        );
                        sig.type_parameters = tp;
                        let method_type = self.create_function_type(sig);
                        properties.push(PropertyInfo {
                            name: CompactStr::new(&name),
                            type_id: method_type,
                            optional: method.optional,
                            readonly: false,
                            decl_order: 0,
                        });
                    }
                }
                TSSignature::TSIndexSignature(idx_sig) => {
                    let value_type =
                        self.get_type_from_type_node(&idx_sig.type_annotation.type_annotation);
                    if let Some(param) = idx_sig.parameters.first() {
                        let key_type =
                            self.get_type_from_type_node(&param.type_annotation.type_annotation);
                        if self.type_arena.get_flags(key_type).intersects(TypeFlags::Number) {
                            *number_index_type = Some(value_type);
                        } else {
                            *string_index_type = Some(value_type);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Resolve a heritage element (from an interface `extends` clause) to a TypeId.
    ///
    /// Heritage elements have an `expression` (typically an IdentifierReference)
    /// and optional `type_arguments`. This mirrors `get_type_from_type_reference`
    /// but operates on `TSInterfaceHeritage` AST nodes.
    ///
    /// Handles both interfaces (creates TypeReference for lazy instantiation)
    /// and type aliases (instantiates body directly, since aliases are transparent).
    fn get_type_from_heritage_element(
        &mut self,
        heritage: &oxc_ast::ast::TSInterfaceHeritage<'_>,
    ) -> TypeId {
        use oxc_ast::AstKind;
        use oxc_ast::ast::Expression;

        // Only handle simple identifier references (not `A.B.C`)
        let Expression::Identifier(ident) = &heritage.expression else {
            return self.any_type;
        };

        let Some(reference_id) = ident.reference_id.get() else {
            return self.any_type;
        };

        let reference = self.semantic().scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else {
            // Unresolved — could be a global type
            let target = self.get_global_type(&ident.name);
            return self
                .maybe_create_type_reference_from_args(target, heritage.type_arguments.as_deref());
        };

        // Type aliases are transparent — instantiate body directly
        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let is_type_alias = matches!(
            self.semantic().nodes().get_node(node_id).kind(),
            AstKind::TSTypeAliasDeclaration(_)
        );

        let target = self.get_declared_type_of_symbol(symbol_id);

        if is_type_alias {
            return self.maybe_instantiate_type_alias_from_args(
                target,
                symbol_id,
                heritage.type_arguments.as_deref(),
            );
        }

        self.maybe_create_type_reference_from_args(target, heritage.type_arguments.as_deref())
    }

    /// Resolve the base (instance) type from a class `extends` clause.
    ///
    /// Class `extends` uses an Expression + optional type arguments, unlike
    /// interface `extends` which uses `TSInterfaceHeritage`. The logic mirrors
    /// `get_type_from_heritage_element`: resolve symbol → declared type →
    /// type reference with args.
    ///
    /// For complex expressions (mixins), falls back to extracting the instance
    /// type from the constructor's construct signatures.
    fn resolve_class_base_type(
        &mut self,
        super_class: &oxc_ast::ast::Expression<'_>,
        super_type_args: Option<&oxc_ast::ast::TSTypeParameterInstantiation<'_>>,
    ) -> TypeId {
        use oxc_ast::AstKind;
        use oxc_ast::ast::Expression;

        // Fast path: identifier → declared type (same pattern as interface heritage)
        if let Expression::Identifier(ident) = super_class {
            if let Some(reference_id) = ident.reference_id.get() {
                let reference = self.semantic().scoping().get_reference(reference_id);
                if let Some(symbol_id) = reference.symbol_id() {
                    let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
                    let is_type_alias = matches!(
                        self.semantic().nodes().get_node(node_id).kind(),
                        AstKind::TSTypeAliasDeclaration(_)
                    );

                    let target = self.get_declared_type_of_symbol(symbol_id);

                    if is_type_alias {
                        return self.maybe_instantiate_type_alias_from_args(
                            target,
                            symbol_id,
                            super_type_args,
                        );
                    }

                    return self
                        .maybe_create_type_reference_from_args(target, super_type_args);
                }

                // Unresolved — could be a global type
                let target = self.get_global_type(&ident.name);
                return self
                    .maybe_create_type_reference_from_args(target, super_type_args);
            }
        }

        // Slow path: complex expression (e.g., `extends Mixin(Base)`)
        // Get the constructor type, then extract the instance type from its
        // first construct signature's return type.
        let constructor_type =
            self.get_type_of_expression(super_class, None, CheckMode::TYPE_ONLY);
        if let TypeData::Structured(s) = self.type_arena.get_data(constructor_type) {
            if let Some(sig) = s.construct_signatures.first() {
                return sig.return_type;
            }
        }
        self.any_type
    }

    /// Build a class instance type from a Class declaration.
    /// Uses StructuredType with Interface kind and ObjectFlags::Class, matching tsc/tsgo's model.
    pub(crate) fn get_type_of_class_declaration(
        &mut self,
        decl: &oxc_ast::ast::Class<'_>,
        symbol_id: SymbolId,
    ) -> TypeId {
        let type_parameters =
            self.get_type_parameters_from_declaration(decl.type_parameters.as_deref());

        let mut properties = Vec::new();

        for element in &decl.body.body {
            use oxc_ast::ast::ClassElement;
            match element {
                ClassElement::PropertyDefinition(prop) => {
                    if prop.r#static {
                        continue; // static props go on constructor type, not instance
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
                        continue;
                    }
                    if let Some(name) = method.key.static_name() {
                        let sig = self.build_signature_from_function(&method.value);
                        let method_type = self.create_function_type(sig);
                        properties.push(PropertyInfo::new(CompactStr::new(&name), method_type));
                    }
                }
                ClassElement::AccessorProperty(_)
                | ClassElement::TSIndexSignature(_)
                | ClassElement::StaticBlock(_) => {
                    // TODO: accessor types, index signatures
                }
            }
        }

        // Handle extends clause — resolve base types (instance type, not constructor)
        let mut resolved_base_types = SmallVec::new();
        if let Some(super_class) = &decl.super_class {
            let base_type = self.resolve_class_base_type(
                super_class,
                decl.super_type_arguments.as_deref(),
            );
            if base_type != self.any_type {
                resolved_base_types.push(base_type);
            }
        }

        sort_properties(&mut properties);
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Class,
            TypeData::Structured(Box::new(StructuredType {
                properties,
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Interface {
                    target: None,
                    resolved_type_arguments: SmallVec::new(),
                    all_type_parameters: type_parameters,
                    this_type: None,
                    resolved_base_types,
                },
            })),
            Some((self.file_idx, symbol_id)),
        )
    }

    /// Build an enum type as a union of member literal types.
    /// The resulting union type gets the enum symbol attached so it displays
    /// as the enum name (e.g., "Choice") rather than "0 | 1 | 2".
    pub(crate) fn get_type_of_enum_declaration(
        &mut self,
        decl: &oxc_ast::ast::TSEnumDeclaration<'_>,
        symbol_id: SymbolId,
    ) -> TypeId {
        let mut member_types = Vec::new();
        let mut auto_value: f64 = 0.0;

        for member in &decl.body.members {
            let member_type = if let Some(init) = &member.initializer {
                let init_type = self.get_type_of_expression(init, None, CheckMode::TYPE_ONLY);
                if let TypeData::Literal(LiteralType::Number(n)) =
                    self.type_arena.get_data(init_type)
                {
                    auto_value = *n + 1.0;
                }
                init_type
            } else {
                let lit_type = self.get_or_create_number_literal_type(auto_value);
                auto_value += 1.0;
                lit_type
            };
            member_types.push(member_type);
        }

        if member_types.is_empty() {
            return self.never_type;
        }

        if member_types.len() == 1 {
            return member_types[0];
        }

        // Create the enum's union type directly (bypassing the dedup cache)
        // with the symbol attached so it displays as the enum name.
        // Enum unions are unique (distinct member sets) so dedup has no benefit,
        // and creating directly avoids mutating an existing cached type.
        let mut types = member_types;
        types.sort();
        types.dedup();
        let key: std::sync::Arc<SmallVec<[TypeId; 4]>> = SmallVec::from_vec(types).into();
        self.type_arena.new_type(
            TypeFlags::Union,
            ObjectFlags::None,
            TypeData::Union(oxc_types::UnionType { types: key }),
            Some((self.file_idx, symbol_id)),
        )
    }

    /// Extract type parameters from a declaration's type parameter list.
    /// Creates a TypeParameter type in the arena for each one.
    ///
    /// Constraints and defaults are NOT resolved here — they are resolved
    /// lazily via `get_constraint_of_type_parameter` when needed. This
    /// matches tsgo's approach and avoids deep recursion during lib.d.ts
    /// bootstrap (where type parameter constraints can chain through
    /// hundreds of interconnected declarations).
    pub(crate) fn get_type_parameters_from_declaration(
        &mut self,
        type_params: Option<&oxc_ast::ast::TSTypeParameterDeclaration<'_>>,
    ) -> SmallVec<[TypeId; 4]> {
        let Some(type_params) = type_params else {
            return SmallVec::new();
        };

        type_params
            .params
            .iter()
            .map(|param| {
                let symbol_id = param.name.symbol_id.get();

                // Reuse existing TypeParameter if already cached. This is
                // critical: the type alias body was resolved with the original
                // TypeParameter TypeIds. If we create fresh ones here, the
                // mapper in maybe_instantiate_type_alias won't match and
                // instantiation silently does nothing.
                if let Some(sid) = symbol_id {
                    if let Some(cached) = self.caches.declared_type_cache[sid] {
                        if self.type_arena.get_flags(cached).intersects(TypeFlags::TypeParameter) {
                            return cached;
                        }
                    }
                }

                let param_name = CompactStr::new(param.name.name.as_str());
                let type_id = self.type_arena.new_type(
                    TypeFlags::TypeParameter,
                    ObjectFlags::None,
                    TypeData::TypeParameter(Box::new(TypeParameterType {
                        name: Some(param_name),
                        constraint: None, // resolved lazily via get_constraint_of_type_parameter
                        target: None,
                        is_this_type: false,
                        resolved_default_type: None, // resolved lazily
                    })),
                    symbol_id.map(|s| (self.file_idx, s)), // store file-indexed symbol for lazy constraint lookup
                );

                // Cache the type parameter against its symbol so that
                // references to `T` within the interface/class body resolve
                // to this TypeParameter type via get_declared_type_of_symbol.
                if let Some(symbol_id) = symbol_id {
                    self.caches.declared_type_cache[symbol_id] = Some(type_id);
                }

                type_id
            })
            .collect()
    }

    /// Get the constraint of a type parameter, resolving lazily on first access.
    ///
    /// Mirrors tsgo's `getConstraintFromTypeParameter` — constraints are NOT
    /// resolved during declaration processing. They're resolved here on first
    /// access, avoiding deep recursion through lib.d.ts's interconnected types.
    ///
    /// Returns `None` if the type parameter has no constraint (unconstrained `T`).
    pub(crate) fn get_constraint_of_type_parameter(
        &mut self,
        type_param_id: TypeId,
    ) -> Option<TypeId> {
        // Check the side cache first
        if let Some(&cached) = self.caches.type_param_constraints.get(&type_param_id) {
            return Some(cached);
        }

        // Check if it was eagerly resolved (stored on the TypeParameter itself)
        if let TypeData::TypeParameter(tp) = self.type_arena.get_data(type_param_id) {
            if let Some(constraint) = tp.constraint {
                return Some(constraint);
            }
        }

        // Find the symbol → declaration → AST constraint.
        // Use file index to determine if the symbol is from this file or another.
        let (file_idx, symbol_id) = self.type_arena.get_symbol(type_param_id)?;
        if file_idx != self.file_idx {
            // Cross-file type parameter — use the host's constraint cache
            return self.host.get_type_param_constraint(type_param_id);
        }
        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        let constraint_type = match node.kind() {
            oxc_ast::AstKind::TSTypeParameter(param) => {
                let constraint_node = param.constraint.as_ref()?;
                self.get_type_from_type_node(constraint_node)
            }
            _ => return None,
        };

        // Cache for subsequent accesses
        self.caches.type_param_constraints.insert(type_param_id, constraint_type);
        Some(constraint_type)
    }
}
