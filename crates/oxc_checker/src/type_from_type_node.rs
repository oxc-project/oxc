use oxc_ast::ast::TSType;
use oxc_span::CompactStr;
use oxc_types::{
    ElementFlags, MappedType, MappedTypeModifier, ObjectFlags, PropertyInfo, SignatureFlags,
    StructuredType, StructuredTypeKind, TupleElementInfo, TupleType, TypeData, TypeFlags, TypeId,
    TypeParameterType, TypeReferenceType, build_member_map,
};
use smallvec::SmallVec;

use crate::Checker;

impl Checker<'_> {
    /// Resolve a `TSType` AST node to a `TypeId`.
    ///
    /// This is the main entry point for converting type syntax into the
    /// checker's internal type representation.
    pub fn get_type_from_type_node(&mut self, ts_type: &TSType<'_>) -> TypeId {
        match ts_type {
            // Keyword types -> pre-allocated intrinsic types
            TSType::TSAnyKeyword(_) => self.any_type,
            TSType::TSUnknownKeyword(_) => self.unknown_type,
            TSType::TSStringKeyword(_) => self.string_type,
            TSType::TSNumberKeyword(_) => self.number_type,
            TSType::TSBigIntKeyword(_) => self.bigint_type,
            TSType::TSBooleanKeyword(_) => self.boolean_type,
            TSType::TSSymbolKeyword(_) => self.es_symbol_type,
            TSType::TSVoidKeyword(_) => self.void_type,
            TSType::TSUndefinedKeyword(_) => self.undefined_type,
            TSType::TSNullKeyword(_) => self.null_type,
            TSType::TSNeverKeyword(_) => self.never_type,
            TSType::TSObjectKeyword(_) => self.non_primitive_type,
            TSType::TSIntrinsicKeyword(_) => self.unknown_type, // intrinsic keyword is contextual

            // Parenthesized types unwrap to the inner type
            TSType::TSParenthesizedType(t) => self.get_type_from_type_node(&t.type_annotation),

            // Union types
            TSType::TSUnionType(union) => self.get_type_from_union_type_node(union),

            // Literal types in annotations: `let x: 42`, `let x: "hello"`, etc.
            TSType::TSLiteralType(lit) => self.get_type_from_literal_type_node(lit),

            // Intersection types: `A & B`
            TSType::TSIntersectionType(intersection) => {
                self.get_type_from_intersection_type_node(intersection)
            }

            // Type reference: `Foo`, `Array<T>`, `MyInterface`, etc.
            TSType::TSTypeReference(ref_type) => self.get_type_from_type_reference(ref_type),

            // Type literal: `{ x: number; y: string }`
            TSType::TSTypeLiteral(lit) => self.get_type_from_type_literal(lit),

            // Array type: `number[]` → TypeReference to Array<number>
            TSType::TSArrayType(arr) => {
                let element_type = self.get_type_from_type_node(&arr.element_type);
                if self.array_type == self.any_type {
                    return self.any_type;
                }
                self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Reference,
                    TypeData::TypeReference(TypeReferenceType {
                        target: Some(self.array_type),
                        resolved_type_arguments: smallvec::smallvec![element_type],
                    }),
                    None,
                )
            }

            // Tuple type: `[string, number]`
            TSType::TSTupleType(tuple) => self.get_type_from_tuple_type_node(tuple),

            // Named tuple member in type position (handled as part of tuple resolution)
            TSType::TSNamedTupleMember(_) => self.any_type,

            // Function type: `(x: number) => string`
            TSType::TSFunctionType(func_type) => {
                let tp =
                    self.get_type_parameters_from_declaration(func_type.type_parameters.as_deref());
                let mut sig = self
                    .build_signature_from_params(&func_type.params, Some(&func_type.return_type));
                sig.type_parameters = tp;
                self.create_function_type(sig)
            }

            // Constructor type: `new (x: number) => Foo`
            TSType::TSConstructorType(ctor_type) => {
                let tp =
                    self.get_type_parameters_from_declaration(ctor_type.type_parameters.as_deref());
                let mut sig = self
                    .build_signature_from_params(&ctor_type.params, Some(&ctor_type.return_type));
                sig.type_parameters = tp;
                sig.flags |= SignatureFlags::Construct;
                if ctor_type.r#abstract {
                    sig.flags |= SignatureFlags::Abstract;
                }
                self.create_constructor_type(sig)
            }

            // typeof x — resolve to value-side type
            TSType::TSTypeQuery(query) => self.get_type_from_type_query(query),

            // `this` type in type position
            TSType::TSThisType(_) => self.this_type,

            // Type operator: `keyof T`, `readonly T`, `unique symbol`
            TSType::TSTypeOperatorType(op) => {
                use oxc_ast::ast::TSTypeOperatorOperator;
                match op.operator {
                    TSTypeOperatorOperator::Keyof => {
                        let target = self.get_type_from_type_node(&op.type_annotation);
                        self.get_index_type(target)
                    }
                    TSTypeOperatorOperator::Readonly => {
                        // readonly modifier — return the inner type (simplified)
                        self.get_type_from_type_node(&op.type_annotation)
                    }
                    TSTypeOperatorOperator::Unique => {
                        // unique symbol
                        self.es_symbol_type
                    }
                }
            }

            // Indexed access type: `T[K]`
            TSType::TSIndexedAccessType(t) => {
                let obj_type = self.get_type_from_type_node(&t.object_type);
                let idx_type = self.get_type_from_type_node(&t.index_type);
                self.get_indexed_access_type(obj_type, idx_type)
            }

            // Mapped type: `{ [P in keyof T]: T[P] }`
            TSType::TSMappedType(mapped) => self.get_type_from_mapped_type_node(mapped),

            // Conditional type: `T extends U ? X : Y`
            TSType::TSConditionalType(cond) => {
                let check = self.get_type_from_type_node(&cond.check_type);

                // Swap infer buffer so nested conditionals don't interfere.
                // TSInferType nodes in the extends clause push to this buffer.
                let prev_infer = std::mem::take(&mut self.current_infer_type_params);
                let extends = self.get_type_from_type_node(&cond.extends_type);
                let infer_params: SmallVec<[TypeId; 2]> =
                    self.current_infer_type_params.drain(..).collect();
                self.current_infer_type_params = prev_infer;

                let true_type = self.get_type_from_type_node(&cond.true_type);
                let false_type = self.get_type_from_type_node(&cond.false_type);

                // Distributive if the check type is a bare type parameter
                let is_distributive =
                    self.type_arena.get_flags(check).intersects(TypeFlags::TypeParameter);

                self.get_conditional_type(
                    check,
                    extends,
                    true_type,
                    false_type,
                    is_distributive,
                    infer_params,
                )
            }

            // Infer type: `infer U` in extends clause of conditional type
            TSType::TSInferType(infer) => self.get_type_from_infer_type_node(infer),

            // Not yet implemented — return `any` as a placeholder
            TSType::TSImportType(_)
            | TSType::TSTemplateLiteralType(_)
            | TSType::TSTypePredicate(_)
            | TSType::JSDocNullableType(_)
            | TSType::JSDocNonNullableType(_)
            | TSType::JSDocUnknownType(_) => {
                // TODO: implement these
                self.any_type
            }
        }
    }

    fn get_type_from_union_type_node(&mut self, union: &oxc_ast::ast::TSUnionType<'_>) -> TypeId {
        let types: Vec<TypeId> =
            union.types.iter().map(|t| self.get_type_from_type_node(t)).collect();
        self.get_or_create_union_type(types)
    }

    fn get_type_from_intersection_type_node(
        &mut self,
        intersection: &oxc_ast::ast::TSIntersectionType<'_>,
    ) -> TypeId {
        let types: Vec<TypeId> =
            intersection.types.iter().map(|t| self.get_type_from_type_node(t)).collect();
        self.get_or_create_intersection_type(types)
    }

    fn get_type_from_literal_type_node(&mut self, lit: &oxc_ast::ast::TSLiteralType<'_>) -> TypeId {
        use oxc_ast::ast::TSLiteral;
        match &lit.literal {
            TSLiteral::BooleanLiteral(b) => {
                if b.value {
                    self.true_type
                } else {
                    self.false_type
                }
            }
            TSLiteral::NumericLiteral(n) => self.get_or_create_number_literal_type(n.value),
            TSLiteral::BigIntLiteral(n) => self.get_or_create_bigint_literal_type(n.value.as_str()),
            TSLiteral::StringLiteral(s) => self.get_or_create_string_literal_type(&s.value),
            TSLiteral::UnaryExpression(unary) => {
                use oxc_ast::ast::Expression;
                use oxc_syntax::operator::UnaryOperator;
                match unary.operator {
                    UnaryOperator::UnaryNegation => match &unary.argument {
                        Expression::NumericLiteral(n) => {
                            self.get_or_create_number_literal_type(-n.value)
                        }
                        _ => self.any_type,
                    },
                    _ => self.any_type,
                }
            }
            // Template literals in type position — fallback
            TSLiteral::TemplateLiteral(_) => self.any_type,
        }
    }

    /// Resolve a type literal (`{ x: number; y: string }`) to a StructuredType.
    fn get_type_from_type_literal(&mut self, lit: &oxc_ast::ast::TSTypeLiteral<'_>) -> TypeId {
        use oxc_ast::ast::TSSignature;

        let mut properties = Vec::new();
        let mut call_signatures = Vec::new();
        let mut construct_signatures: Vec<oxc_types::Signature> = Vec::new();
        let mut string_index_type: Option<TypeId> = None;
        let mut number_index_type: Option<TypeId> = None;

        for sig in &lit.members {
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
                            number_index_type = Some(value_type);
                        } else {
                            string_index_type = Some(value_type);
                        }
                    }
                }
                TSSignature::TSConstructSignatureDeclaration(ctor_sig) => {
                    let tp = self
                        .get_type_parameters_from_declaration(ctor_sig.type_parameters.as_deref());
                    let mut sig = self.build_signature_from_params(
                        &ctor_sig.params,
                        ctor_sig.return_type.as_deref(),
                    );
                    sig.type_parameters = tp;
                    sig.flags |= SignatureFlags::Construct;
                    construct_signatures.push(sig);
                }
            }
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous,
            TypeData::Structured(StructuredType {
                member_map: build_member_map(&properties),
                properties,
                string_index_type,
                number_index_type,
                call_signatures,
                construct_signatures,
                kind: StructuredTypeKind::Anonymous { target: None },
            }),
            None,
        )
    }

    /// Resolve a type reference (`Foo`, `Array<T>`, etc.) to a TypeId.
    ///
    /// Looks up the type name in the symbol table, then delegates to
    /// `get_declared_type_of_symbol` for type-namespace resolution.
    ///
    /// For interfaces/classes with type arguments: creates a `TypeReference`
    /// for lazy instantiation.
    /// For type aliases with type arguments: instantiates the body directly
    /// (type aliases are transparent — no TypeReference wrapper). This
    /// matches tsgo's approach and avoids embedding alias type parameters
    /// into every possible type-alias-body type kind.
    fn get_type_from_type_reference(
        &mut self,
        ref_type: &oxc_ast::ast::TSTypeReference<'_>,
    ) -> TypeId {
        use oxc_ast::AstKind;
        use oxc_ast::ast::TSTypeName;

        // Only handle simple identifier references for now (not `A.B.C`)
        let TSTypeName::IdentifierReference(ident) = &ref_type.type_name else {
            return self.any_type;
        };

        let Some(reference_id) = ident.reference_id.get() else {
            return self.any_type;
        };

        let reference = self.semantic().scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else {
            // Unresolved reference — could be a global type (Array, Promise, etc.)
            let target = self.get_global_type(&ident.name);
            return self.maybe_create_type_reference(target, ref_type);
        };

        // Check if this is a type alias — if so, instantiate the body directly
        // rather than creating a TypeReference wrapper.
        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let is_type_alias = matches!(
            self.semantic().nodes().get_node(node_id).kind(),
            AstKind::TSTypeAliasDeclaration(_)
        );

        let target = self.get_declared_type_of_symbol(symbol_id);

        if is_type_alias {
            return self.maybe_instantiate_type_alias(target, symbol_id, ref_type);
        }

        self.maybe_create_type_reference(target, ref_type)
    }

    /// Instantiate a type alias body with type arguments.
    ///
    /// Type aliases are transparent: `Partial<{a: string}>` directly
    /// instantiates the mapped type body with T = {a: string}, producing
    /// a concrete StructuredType. No TypeReference wrapper is created.
    ///
    /// If there are no type arguments, returns the target as-is.
    fn maybe_instantiate_type_alias(
        &mut self,
        target: TypeId,
        symbol_id: oxc_syntax::symbol::SymbolId,
        ref_type: &oxc_ast::ast::TSTypeReference<'_>,
    ) -> TypeId {
        let Some(type_args_node) = &ref_type.type_arguments else {
            return target;
        };

        // Resolve type arguments
        let type_arguments: SmallVec<[TypeId; 4]> =
            type_args_node.params.iter().map(|arg| self.get_type_from_type_node(arg)).collect();

        // Get the type alias's type parameters from the declaration
        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let type_params = match self.semantic().nodes().get_node(node_id).kind() {
            oxc_ast::AstKind::TSTypeAliasDeclaration(decl) => {
                self.get_type_parameters_from_declaration(decl.type_parameters.as_deref())
            }
            _ => return target,
        };

        if type_params.is_empty() {
            return target;
        }

        // Build mapper from alias type params → type arguments
        let Some(mapper) =
            crate::instantiation::TypeMapper::from_type_parameters(&type_params, &type_arguments)
        else {
            return target;
        };

        // Instantiate the body with the mapper — this handles MappedType,
        // ConditionalType, unions, intersections, and any other body type
        // that references the alias type parameters.
        self.instantiate_type(target, &mapper)
    }

    /// Resolve a tuple type node (`[string, number]`) to a TupleType.
    fn get_type_from_tuple_type_node(&mut self, tuple: &oxc_ast::ast::TSTupleType<'_>) -> TypeId {
        use oxc_ast::ast::TSTupleElement;

        let mut element_infos = Vec::new();
        let mut type_arguments = SmallVec::new();
        let mut min_length: u32 = 0;
        let mut has_rest = false;

        for element in &tuple.element_types {
            let (elem_type, flags, label) = match element {
                TSTupleElement::TSNamedTupleMember(named) => {
                    let inner_type = self.resolve_tuple_inner_element(&named.element_type);
                    let flags = if named.optional {
                        ElementFlags::Optional
                    } else {
                        ElementFlags::Required
                    };
                    let label = Some(CompactStr::new(named.label.name.as_str()));
                    (inner_type, flags, label)
                }
                TSTupleElement::TSOptionalType(opt) => {
                    let inner_type = self.get_type_from_type_node(&opt.type_annotation);
                    (inner_type, ElementFlags::Optional, None)
                }
                TSTupleElement::TSRestType(rest) => {
                    let inner_type = self.get_type_from_type_node(&rest.type_annotation);
                    (inner_type, ElementFlags::Rest, None)
                }
                // All other cases are TSType variants (inherited)
                other => {
                    let inner_type = self.resolve_tuple_inner_element(other);
                    (inner_type, ElementFlags::Required, None)
                }
            };

            type_arguments.push(elem_type);
            if flags.contains(ElementFlags::Required) {
                min_length += 1;
            }
            if flags.contains(ElementFlags::Rest) {
                has_rest = true;
            }
            element_infos.push(TupleElementInfo {
                element_type: elem_type,
                flags,
                label_name: label,
            });
        }

        let fixed_length =
            if has_rest { type_arguments.len() as u32 - 1 } else { type_arguments.len() as u32 };

        let combined_flags =
            element_infos.iter().fold(ElementFlags::empty(), |acc, info| acc | info.flags);

        let mut obj_flags = ObjectFlags::Tuple;
        if type_arguments.iter().any(|&t| self.type_could_contain_type_variables(t)) {
            obj_flags |= ObjectFlags::CouldContainTypeVariables;
        }
        self.type_arena.new_type(
            TypeFlags::Object,
            obj_flags,
            TypeData::Tuple(TupleType {
                target: None,
                resolved_type_arguments: type_arguments,
                element_infos,
                min_length,
                fixed_length,
                combined_flags,
                readonly: false,
            }),
            None,
        )
    }

    /// Resolve a `TSTupleElement` to its type.
    ///
    /// Uses `as_ts_type()` (generated by `inherit_variants!`) to convert
    /// shared TSType variants, falling back to handling TSTupleElement-specific
    /// variants (TSOptionalType, TSRestType) directly.
    fn resolve_tuple_inner_element(
        &mut self,
        element: &oxc_ast::ast::TSTupleElement<'_>,
    ) -> TypeId {
        use oxc_ast::ast::TSTupleElement;
        // Handle TSTupleElement-specific variants
        match element {
            TSTupleElement::TSOptionalType(opt) => {
                self.get_type_from_type_node(&opt.type_annotation)
            }
            TSTupleElement::TSRestType(rest) => self.get_type_from_type_node(&rest.type_annotation),
            _ => {
                // All other variants are inherited TSType variants.
                // as_ts_type() is generated by inherit_variants! and performs
                // a zero-cost transmute for shared variants.
                match element.as_ts_type() {
                    Some(ts_type) => self.get_type_from_type_node(ts_type),
                    None => self.any_type,
                }
            }
        }
    }

    /// If the type reference has type arguments, create a TypeReference
    /// wrapping the target type with resolved type arguments. Otherwise
    /// return the target directly.
    fn maybe_create_type_reference(
        &mut self,
        target: TypeId,
        ref_type: &oxc_ast::ast::TSTypeReference<'_>,
    ) -> TypeId {
        let Some(type_args_node) = &ref_type.type_arguments else {
            return target;
        };

        // Resolve each type argument
        let type_arguments: SmallVec<[TypeId; 4]> =
            type_args_node.params.iter().map(|arg| self.get_type_from_type_node(arg)).collect();

        // Check the target actually has type parameters — if not,
        // the type arguments are extraneous (e.g., `string<number>`).
        // In that case, just return the target.
        let has_type_params = match self.type_arena.get_data(target) {
            TypeData::Structured(StructuredType {
                kind: StructuredTypeKind::Interface { all_type_parameters, .. },
                ..
            }) => !all_type_parameters.is_empty(),
            _ => false,
        };

        if !has_type_params {
            return target;
        }

        // Create a TypeReference for lazy instantiation.
        // Properties are not instantiated now — they're resolved lazily
        // when accessed (e.g., during structural assignability checks).
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Reference,
            TypeData::TypeReference(TypeReferenceType {
                target: Some(target),
                resolved_type_arguments: type_arguments,
            }),
            None,
        )
    }

    /// Resolve `typeof x` in type position to the value-side type of `x`.
    fn get_type_from_type_query(&mut self, query: &oxc_ast::ast::TSTypeQuery<'_>) -> TypeId {
        use oxc_ast::ast::TSTypeQueryExprName;

        match &query.expr_name {
            TSTypeQueryExprName::IdentifierReference(ident) => {
                let Some(reference_id) = ident.reference_id.get() else {
                    return self.any_type;
                };
                let reference = self.semantic().scoping().get_reference(reference_id);
                let Some(symbol_id) = reference.symbol_id() else {
                    return self.any_type;
                };
                self.get_type_of_symbol(symbol_id)
            }
            // QualifiedName (typeof x.y.z), TSImportType, ThisExpression — not yet supported
            _ => self.any_type,
        }
    }

    /// Build a MappedType from a `{ [P in C]: T }` AST node.
    ///
    /// Creates a TypeParameter for the iteration variable P, resolves the
    /// constraint C and template T, and stores them in a MappedType for
    /// deferred resolution. Like a generic interface, the mapped type isn't
    /// resolved until instantiated with concrete type arguments.
    fn get_type_from_mapped_type_node(
        &mut self,
        mapped: &oxc_ast::ast::TSMappedType<'_>,
    ) -> TypeId {
        use oxc_ast::ast::TSMappedTypeModifierOperator;

        // Create a TypeParameter for the iteration variable (P in [P in keyof T])
        let type_param = self.type_arena.new_type(
            TypeFlags::TypeParameter,
            ObjectFlags::None,
            TypeData::TypeParameter(TypeParameterType {
                name: Some(CompactStr::new(mapped.key.name.as_str())),
                constraint: None, // set after resolving constraint
                target: None,
                is_this_type: false,
                resolved_default_type: None,
            }),
            None,
        );

        // If the key binding has a symbol, cache the type parameter against it
        // so references to P within the template resolve correctly.
        if let Some(symbol_id) = mapped.key.symbol_id.get() {
            self.declared_type_cache[symbol_id] = Some(type_param);
        }

        // Resolve the constraint (e.g., `keyof T`) and template (e.g., `T[P]`).
        // For generic mapped types, these resolve to deferred types
        // (IndexType, IndexedAccessType) that are instantiated later.
        let constraint_type = self.get_type_from_type_node(&mapped.constraint);

        let template_type =
            mapped.type_annotation.as_ref().map(|t| self.get_type_from_type_node(t));

        let name_type = mapped.name_type.as_ref().map(|t| self.get_type_from_type_node(t));

        // Convert AST modifiers to our representation
        let optional_modifier = match mapped.optional {
            None => MappedTypeModifier::None,
            Some(TSMappedTypeModifierOperator::True | TSMappedTypeModifierOperator::Plus) => {
                MappedTypeModifier::Add
            }
            Some(TSMappedTypeModifierOperator::Minus) => MappedTypeModifier::Remove,
        };
        let readonly_modifier = match mapped.readonly {
            None => MappedTypeModifier::None,
            Some(TSMappedTypeModifierOperator::True | TSMappedTypeModifierOperator::Plus) => {
                MappedTypeModifier::Add
            }
            Some(TSMappedTypeModifierOperator::Minus) => MappedTypeModifier::Remove,
        };

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Mapped,
            TypeData::Mapped(MappedType {
                type_parameter: type_param,
                constraint_type: Some(constraint_type),
                name_type,
                template_type,
                optional_modifier,
                readonly_modifier,
            }),
            None,
        )
    }

    /// Handle `infer U` in the extends clause of a conditional type.
    ///
    /// Creates a fresh TypeParameter for the infer declaration, caches it
    /// in `declared_type_cache` (so references in the true branch resolve
    /// to the same TypeId), and pushes it to `current_infer_type_params`
    /// for collection by the enclosing `TSConditionalType` handler.
    fn get_type_from_infer_type_node(&mut self, infer: &oxc_ast::ast::TSInferType<'_>) -> TypeId {
        let tp = &infer.type_parameter;
        let symbol_id = tp.name.symbol_id.get();

        // Reuse cached TypeParameter if already created (e.g., multiple
        // references to the same infer param within the extends clause).
        if let Some(sid) = symbol_id {
            if let Some(cached) = self.declared_type_cache[sid] {
                if self.type_arena.get_flags(cached).intersects(TypeFlags::TypeParameter) {
                    self.current_infer_type_params.push(cached);
                    return cached;
                }
            }
        }

        let param_name = CompactStr::new(tp.name.name.as_str());
        let type_id = self.type_arena.new_type(
            TypeFlags::TypeParameter,
            ObjectFlags::None,
            TypeData::TypeParameter(TypeParameterType {
                name: Some(param_name),
                constraint: None, // resolved lazily via get_constraint_of_type_parameter
                target: None,
                is_this_type: false,
                resolved_default_type: None,
            }),
            symbol_id.map(|s| (self.file_idx, s)),
        );

        // Cache so that references in the true branch resolve to this TypeId.
        if let Some(symbol_id) = symbol_id {
            self.declared_type_cache[symbol_id] = Some(type_id);
        }

        self.current_infer_type_params.push(type_id);
        type_id
    }
}
