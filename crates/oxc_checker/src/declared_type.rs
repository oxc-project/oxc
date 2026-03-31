use oxc_span::CompactStr;
use oxc_syntax::symbol::SymbolId;
use oxc_types::{InterfaceType, LiteralType, ObjectFlags, PropertyInfo, Signature, TypeData, TypeFlags, TypeId, TypeParameterType, build_member_map};
use smallvec::SmallVec;

use crate::Checker;

impl Checker<'_> {
    /// Get the declared type of a type-namespace symbol (type alias, interface,
    /// class, enum). Uses caching and cycle detection, mirroring tsgo's
    /// `getDeclaredTypeOfSymbol`.
    pub fn get_declared_type_of_symbol(&mut self, symbol_id: SymbolId) -> TypeId {
        // Check cache (IndexVec: O(1) array indexing)
        if let Some(cached) = self.declared_type_cache[symbol_id] {
            return cached;
        }

        // Cycle detection
        if !self.resolving_symbols.insert(symbol_id) {
            return self.any_type;
        }

        let result = self.resolve_declared_type(symbol_id);
        self.resolving_symbols.remove(&symbol_id);
        self.declared_type_cache[symbol_id] = Some(result);
        result
    }

    /// Resolve the declared type from a type-namespace declaration.
    fn resolve_declared_type(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        match node.kind() {
            AstKind::TSTypeAliasDeclaration(decl) => {
                // Resolve outer type parameters (e.g., T in `type Partial<T> = ...`)
                // so they exist in declared_type_cache for the body to reference.
                self.get_type_parameters_from_declaration(
                    decl.type_parameters.as_deref(),
                );

                // Resolve the body — type parameter references within resolve
                // via declared_type_cache. Instantiation with concrete type args
                // happens later in maybe_instantiate_type_alias (type aliases are
                // transparent — no TypeReference wrapper).
                self.get_type_from_type_node(&decl.type_annotation)
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                self.get_type_of_interface_declaration(decl, symbol_id)
            }
            AstKind::Class(decl) => {
                self.get_type_of_class_declaration(decl, symbol_id)
            }
            AstKind::TSEnumDeclaration(decl) => {
                self.get_type_of_enum_declaration(decl, symbol_id)
            }
            _ => self.any_type,
        }
    }

    /// Build an interface type from a TSInterfaceDeclaration.
    fn get_type_of_interface_declaration(
        &mut self,
        decl: &oxc_ast::ast::TSInterfaceDeclaration<'_>,
        symbol_id: SymbolId,
    ) -> TypeId {
        // Extract type parameters (e.g., T in interface Foo<T>)
        let type_parameters = self.get_type_parameters_from_declaration(
            decl.type_parameters.as_deref(),
        );

        let mut properties = Vec::new();
        let mut call_signatures = Vec::new();
        let construct_signatures: Vec<Signature> = Vec::new();

        for sig in &decl.body.body {
            use oxc_ast::ast::TSSignature;
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
                        });
                    }
                }
                TSSignature::TSCallSignatureDeclaration(call_sig) => {
                    let sig = self.build_signature_from_params(
                        &call_sig.params,
                        call_sig.return_type.as_deref(),
                    );
                    call_signatures.push(sig);
                }
                TSSignature::TSMethodSignature(method) => {
                    if let Some(name) = method.key.static_name() {
                        let sig = self.build_signature_from_params(
                            &method.params,
                            method.return_type.as_deref(),
                        );
                        let method_type = self.create_function_type(sig);
                        properties.push(PropertyInfo {
                            name: CompactStr::new(&name),
                            type_id: method_type,
                            optional: method.optional,
                            readonly: false,
                        });
                    }
                }
                // TODO: index signatures, construct signatures
                _ => {}
            }
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Interface,
            TypeData::Interface(InterfaceType {
                target: None,
                resolved_type_arguments: SmallVec::new(),
                all_type_parameters: type_parameters,
                this_type: None,
                resolved_base_types: SmallVec::new(),
                member_map: build_member_map(&properties),
                properties,
                call_signatures,
                construct_signatures,
            }),
            Some(symbol_id),
        )
    }

    /// Build a class instance type from a Class declaration.
    /// Uses InterfaceType with ObjectFlags::Class, matching tsc/tsgo's model.
    pub(crate) fn get_type_of_class_declaration(
        &mut self,
        decl: &oxc_ast::ast::Class<'_>,
        symbol_id: SymbolId,
    ) -> TypeId {
        let type_parameters = self.get_type_parameters_from_declaration(
            decl.type_parameters.as_deref(),
        );

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
                            self.get_type_of_expression(init)
                        } else {
                            self.any_type
                        };
                        properties.push(PropertyInfo {
                            name: CompactStr::new(&name),
                            type_id: prop_type,
                            optional: prop.optional,
                            readonly: prop.readonly,
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

        // Handle extends clause — resolve base types
        let mut resolved_base_types = SmallVec::new();
        if let Some(super_class) = &decl.super_class {
            let base_type = self.get_type_of_expression(super_class);
            if base_type != self.any_type {
                resolved_base_types.push(base_type);
            }
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Class,
            TypeData::Interface(InterfaceType {
                target: None,
                resolved_type_arguments: SmallVec::new(),
                all_type_parameters: type_parameters,
                this_type: None,
                resolved_base_types,
                member_map: build_member_map(&properties),
                properties,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
            }),
            Some(symbol_id),
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
                let init_type = self.get_type_of_expression(init);
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
            Some(symbol_id),
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
                    if let Some(cached) = self.declared_type_cache[sid] {
                        if self.type_arena.get_flags(cached).intersects(TypeFlags::TypeParameter) {
                            return cached;
                        }
                    }
                }

                let param_name = CompactStr::new(param.name.name.as_str());
                let type_id = self.type_arena.new_type(
                    TypeFlags::TypeParameter,
                    ObjectFlags::None,
                    TypeData::TypeParameter(TypeParameterType {
                        name: Some(param_name),
                        constraint: None, // resolved lazily via get_constraint_of_type_parameter
                        target: None,
                        is_this_type: false,
                        resolved_default_type: None, // resolved lazily
                    }),
                    symbol_id, // store symbol for lazy constraint lookup
                );

                // Cache the type parameter against its symbol so that
                // references to `T` within the interface/class body resolve
                // to this TypeParameter type via get_declared_type_of_symbol.
                if let Some(symbol_id) = symbol_id {
                    self.declared_type_cache[symbol_id] = Some(type_id);
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
    pub(crate) fn get_constraint_of_type_parameter(&mut self, type_param_id: TypeId) -> Option<TypeId> {
        // Check the side cache first
        if let Some(&cached) = self.type_param_constraints.get(&type_param_id) {
            return Some(cached);
        }

        // Check if it was eagerly resolved (stored on the TypeParameter itself)
        if let TypeData::TypeParameter(tp) = self.type_arena.get_data(type_param_id) {
            if let Some(constraint) = tp.constraint {
                return Some(constraint);
            }
        }

        // Find the symbol → declaration → AST constraint
        let symbol_id = self.type_arena.get_symbol(type_param_id)?;
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
        self.type_param_constraints.insert(type_param_id, constraint_type);
        Some(constraint_type)
    }
}
