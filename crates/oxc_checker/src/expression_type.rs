use oxc_ast::ast::{
    BinaryExpression, ChainExpression, ComputedMemberExpression, Expression, UnaryExpression,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{CompactStr, GetSpan};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use oxc_syntax::symbol::SymbolId;
use oxc_types::{LiteralType, ObjectFlags, PropertyInfo, StructuredType, StructuredTypeKind, TypeData, TypeFlags, TypeId, build_member_map};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::Checker;

impl Checker<'_> {
    /// Get the type of an expression.
    ///
    /// For literals, returns the corresponding literal type.
    /// For identifiers, resolves the symbol and returns its declared type.
    /// Unimplemented expressions fall back to `any`.
    pub fn get_type_of_expression(
        &mut self,
        expr: &Expression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        // Guard against infinite recursion (e.g., `const x = x`)
        if self.recursion_depth > 100 {
            return self.any_type;
        }
        self.recursion_depth += 1;
        let result = self.get_type_of_expression_inner(expr, contextual_type);
        self.recursion_depth -= 1;
        result
    }

    fn get_type_of_expression_inner(
        &mut self,
        expr: &Expression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        match expr {
            Expression::StringLiteral(lit) => {
                self.get_or_create_string_literal_type(&lit.value)
            }
            Expression::NumericLiteral(lit) => {
                self.get_or_create_number_literal_type(lit.value)
            }
            Expression::BigIntLiteral(lit) => {
                self.get_or_create_bigint_literal_type(lit.value.as_str())
            }
            Expression::BooleanLiteral(lit) => {
                if lit.value {
                    self.true_type
                } else {
                    self.false_type
                }
            }
            Expression::NullLiteral(_) => self.null_type,
            Expression::Identifier(ident) => self.get_type_of_identifier(ident),
            Expression::ParenthesizedExpression(paren) => {
                self.get_type_of_expression(&paren.expression, contextual_type)
            }
            // Type assertions — return the asserted type
            Expression::TSAsExpression(expr) => {
                self.get_type_from_type_node(&expr.type_annotation)
            }
            Expression::TSTypeAssertion(expr) => {
                self.get_type_from_type_node(&expr.type_annotation)
            }
            // `satisfies` checks but returns the expression's type, not the annotation
            Expression::TSSatisfiesExpression(expr) => {
                self.get_type_of_expression(&expr.expression, contextual_type)
            }
            // Non-null assertion — return the expression type (TODO: remove null/undefined)
            Expression::TSNonNullExpression(expr) => {
                self.get_type_of_expression(&expr.expression, contextual_type)
            }

            // Unary expressions
            Expression::UnaryExpression(expr) => self.get_type_of_unary_expression(expr),

            // Binary expressions
            Expression::BinaryExpression(expr) => self.get_type_of_binary_expression(expr),

            // Conditional (ternary) — union of both branches
            Expression::ConditionalExpression(expr) => {
                let true_type = self.get_type_of_expression(&expr.consequent, contextual_type);
                let false_type = self.get_type_of_expression(&expr.alternate, contextual_type);
                self.get_or_create_union_type(vec![true_type, false_type])
            }

            // Template literals — always string (simplified; tsc can produce literal types)
            Expression::TemplateLiteral(_) => self.string_type,

            // Sequence expression — type of the last element
            Expression::SequenceExpression(expr) => {
                if let Some(last) = expr.expressions.last() {
                    self.get_type_of_expression(last, contextual_type)
                } else {
                    self.undefined_type
                }
            }

            // void x — always undefined
            // (handled in unary, but keeping note)

            // Logical expressions — simplified to union of both sides
            Expression::LogicalExpression(expr) => {
                let left_type = self.get_type_of_expression(&expr.left, contextual_type);
                let right_type = self.get_type_of_expression(&expr.right, contextual_type);
                self.get_or_create_union_type(vec![left_type, right_type])
            }

            // ++x, x++ etc — returns number (simplified; bigint not handled)
            Expression::UpdateExpression(_) => self.number_type,

            // Object literal: `{ x: 1, y: "hello" }`
            Expression::ObjectExpression(obj) => self.get_type_of_object_literal(obj, contextual_type),

            // Property access: foo.bar
            Expression::StaticMemberExpression(expr) => {
                self.get_type_of_static_member_expression(expr)
            }

            // Array literal: [1, 2, 3]
            Expression::ArrayExpression(arr) => self.get_type_of_array_literal(arr, contextual_type),

            // Assignment: result type is the RHS
            Expression::AssignmentExpression(assign) => {
                self.get_type_of_expression(&assign.right, contextual_type)
            }

            // Optional chaining: unwrap inner, union with undefined
            Expression::ChainExpression(chain) => self.get_type_of_chain_expression(chain),

            // Computed member access: obj["key"]
            Expression::ComputedMemberExpression(expr) => {
                self.get_type_of_computed_member_expression(expr)
            }

            // await expr — simplified: returns operand type directly
            Expression::AwaitExpression(expr) => self.get_type_of_expression(&expr.argument, None),

            // /regex/ — always RegExp
            Expression::RegExpLiteral(_) => self.get_global_type("RegExp"),

            // this — returns the enclosing class type or the implicit `this` type parameter
            Expression::ThisExpression(_) => {
                if let Some(&class_type) = self.class_type_stack.last() {
                    class_type
                } else {
                    self.this_type
                }
            }

            // These need more infrastructure (call signatures, generators, modules)
            Expression::YieldExpression(_)
            | Expression::ImportExpression(_)
            | Expression::TaggedTemplateExpression(_) => self.any_type,

            Expression::ArrowFunctionExpression(arrow) => {
                // Inline arena access to get contextual signature without
                // borrowing &self (avoids clone). self.type_arena is &'a,
                // so the returned reference has lifetime 'a.
                let contextual_sig = contextual_type.and_then(|c| {
                    match self.type_arena.get_data(c) {
                        TypeData::Function(f) => f.signatures.first(),
                        TypeData::Structured(s) => s.call_signatures.first(),
                        _ => None,
                    }
                });
                // Create type parameters before building the signature so that
                // references to T in parameter/return types resolve correctly.
                let type_parameters = self.get_type_parameters_from_declaration(
                    arrow.type_parameters.as_deref(),
                );
                let mut sig = self.build_signature_from_params_with_context(
                    &arrow.params,
                    arrow.return_type.as_deref(),
                    contextual_sig,
                );
                sig.type_parameters = type_parameters;
                // Infer return type when there's no annotation.
                if arrow.return_type.is_none() {
                    let return_contextual_type = contextual_sig.map(|s| s.return_type);
                    sig.return_type = if arrow.expression {
                        // Expression body: () => expr — return type is the expression type
                        if let Some(oxc_ast::ast::Statement::ExpressionStatement(expr_stmt)) =
                            arrow.body.statements.first()
                        {
                            self.get_type_of_expression(
                                &expr_stmt.expression,
                                return_contextual_type,
                            )
                        } else {
                            self.void_type
                        }
                    } else {
                        // Block body: () => { ... } — infer from return statements
                        self.infer_return_type_from_body(&arrow.body.statements)
                    };
                }
                self.create_function_type(sig)
            }

            Expression::FunctionExpression(func) => {
                let contextual_sig = contextual_type.and_then(|c| {
                    match self.type_arena.get_data(c) {
                        TypeData::Function(f) => f.signatures.first(),
                        TypeData::Structured(s) => s.call_signatures.first(),
                        _ => None,
                    }
                });
                let sig =
                    self.build_signature_from_function_with_context(func, contextual_sig);
                self.create_function_type(sig)
            }

            Expression::CallExpression(call) => self.get_type_of_call_expression(call),

            Expression::NewExpression(new_expr) => self.get_type_of_new_expression(new_expr),

            // super — represents the parent class constructor or prototype
            Expression::Super(_) => self.any_type,

            // Not yet implemented — return `any`
            Expression::MetaProperty(_)
            | Expression::ClassExpression(_)
            | Expression::PrivateInExpression(_)
            | Expression::JSXElement(_)
            | Expression::JSXFragment(_)
            | Expression::TSInstantiationExpression(_)
            | Expression::V8IntrinsicExpression(_)
            | Expression::PrivateFieldExpression(_) => self.any_type,
        }
    }

    /// Resolve an identifier reference to its type.
    ///
    /// Looks up the reference -> symbol -> declaration -> type annotation.
    pub(crate) fn get_type_of_identifier(
        &mut self,
        ident: &oxc_ast::ast::IdentifierReference<'_>,
    ) -> TypeId {
        let Some(reference_id) = ident.reference_id.get() else {
            return self.any_type;
        };

        let reference = self.semantic().scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else {
            // Unresolved reference — check well-known globals
            return self.get_type_of_global_identifier(&ident.name);
        };

        let declared_type = self.get_type_of_symbol(symbol_id);

        // Apply control flow narrowing if a flow graph is active.
        self.get_flow_type_of_reference(ident.node_id.get(), symbol_id, declared_type)
    }

    /// Get the type of a well-known global identifier.
    fn get_type_of_global_identifier(&self, name: &str) -> TypeId {
        match name {
            "undefined" => self.undefined_type,
            "NaN" | "Infinity" => self.number_type,
            _ => self.any_type,
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

        // Import binding — resolve via host (cross-file)
        let symbol_flags = self.semantic().scoping().symbol_flags(symbol_id);
        if symbol_flags.is_import() {
            return self.resolve_import_type(symbol_id);
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
                        self.get_type_of_expression(init, None)
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
                    let inferred = self.get_type_of_expression(init, None);
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
            AstKind::Class(_) => {
                // The value type of a class is the constructor — an anonymous
                // object type with the class symbol attached. Displays as
                // "typeof C". For binding identifiers, the conformance runner
                // uses get_declared_type_of_symbol instead, which returns the
                // instance type displayed as "C".
                self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Anonymous,
                    TypeData::Structured(StructuredType {
                        properties: Vec::new(),
                        member_map: FxHashMap::default(),
                        string_index_type: None,
                        number_index_type: None,
                        call_signatures: Vec::new(),
                        construct_signatures: Vec::new(),
                        kind: StructuredTypeKind::Anonymous { target: None },
                    }),
                    Some((self.file_idx, symbol_id)),
                )
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
                self.type_arena.new_type(
                    TypeFlags::Object,
                    ObjectFlags::Anonymous,
                    TypeData::Structured(StructuredType { member_map: build_member_map(&properties), properties, string_index_type: None, number_index_type: None, call_signatures: Vec::new(), construct_signatures: Vec::new(), kind: StructuredTypeKind::Anonymous { target: None } }),
                    Some((self.file_idx, symbol_id)),
                )
            }
            AstKind::TSEnumMember(member) => {
                // Individual enum member: resolve its literal type
                if let Some(init) = &member.initializer {
                    self.get_type_of_expression(init, None)
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
            _ => self.any_type,
        }
    }

    /// Compute the type of an enum member, updating the auto-increment counter.
    fn compute_enum_member_value(
        &mut self,
        member: &oxc_ast::ast::TSEnumMember<'_>,
        auto_value: &mut f64,
    ) -> TypeId {
        if let Some(init) = &member.initializer {
            let t = self.get_type_of_expression(init, None);
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

    /// Check if a VariableDeclarator is inside a for-of or for-in statement.
    /// If so, return the iterated element type; otherwise return `any_type`.
    fn get_type_from_for_loop_context(&mut self, declarator_node_id: oxc_syntax::node::NodeId) -> TypeId {
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
                let iterable_type = self.get_type_of_expression(&for_of.right, None);
                self.get_iterated_type_of_iterable(iterable_type)
            }
            AstKind::ForInStatement(_) => self.string_type,
            _ => unreachable!(),
        }
    }

    /// Get the iterated element type from an iterable type.
    /// For Array<T>, returns T. For tuples, returns union of element types.
    fn get_iterated_type_of_iterable(&mut self, type_id: TypeId) -> TypeId {
        // Check Array<T> → T
        if let TypeData::TypeReference(tr) = self.type_arena.get_data(type_id) {
            if tr.target == Some(self.array_type) && !tr.resolved_type_arguments.is_empty() {
                return tr.resolved_type_arguments[0];
            }
            return self.any_type;
        }
        // Collect tuple element types (releases the type_arena borrow before union creation)
        let tuple_element_types = if let TypeData::Tuple(tuple) = self.type_arena.get_data(type_id) {
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
    fn resolve_destructured_binding_type(
        &mut self,
        pattern: &oxc_ast::ast::BindingPattern<'_>,
        init_type: TypeId,
        target_symbol: SymbolId,
    ) -> Option<TypeId> {
        use oxc_ast::ast::BindingPattern;
        match pattern {
            BindingPattern::BindingIdentifier(ident) => {
                if ident.symbol_id.get() == Some(target_symbol) {
                    Some(init_type)
                } else {
                    None
                }
            }
            BindingPattern::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    if let Some(name) = prop.key.static_name() {
                        let prop_type = self.get_property_of_type(init_type, &name);
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

    /// Get the result type of a unary expression.
    fn get_type_of_unary_expression(&mut self, expr: &UnaryExpression<'_>) -> TypeId {
        match expr.operator {
            // typeof always returns a string
            UnaryOperator::Typeof => self.string_type,
            // void always returns undefined
            UnaryOperator::Void => self.undefined_type,
            // ! returns boolean
            UnaryOperator::LogicalNot => self.boolean_type,
            // delete returns boolean
            UnaryOperator::Delete => self.boolean_type,
            // +x always returns number
            UnaryOperator::UnaryPlus => self.number_type,
            // -x and ~x return number or bigint depending on operand
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                let operand_type = self.get_type_of_expression(&expr.argument, None);
                let operand_flags = self.type_arena.get_flags(operand_type);
                if operand_flags.intersects(TypeFlags::BigIntLike) {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }
        }
    }

    /// Get the type of an object literal expression.
    ///
    /// When a `contextual_type` is provided (e.g., from a variable declaration annotation),
    /// each property value is checked with its corresponding contextual property type.
    /// This enables contextual typing of nested callbacks.
    fn get_type_of_object_literal(
        &mut self,
        obj: &oxc_ast::ast::ObjectExpression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        use oxc_ast::ast::{ObjectPropertyKind, PropertyKind};

        let mut properties = Vec::new();

        for prop_kind in &obj.properties {
            match prop_kind {
                ObjectPropertyKind::ObjectProperty(prop) => {
                    if prop.kind != PropertyKind::Init {
                        continue;
                    }
                    if let Some(name) = prop.key.static_name() {
                        // Look up contextual type for this property
                        let prop_contextual_type = contextual_type.and_then(|ct| {
                            let pt = self.get_property_of_type(ct, &name);
                            if pt != self.any_type { Some(pt) } else { None }
                        });
                        let prop_type =
                            self.get_type_of_expression(&prop.value, prop_contextual_type);
                        // Widen literal types in object literal properties
                        let widened = self.get_widened_literal_type(prop_type);
                        properties.push(PropertyInfo::new(CompactStr::new(&name), widened));
                    }
                }
                ObjectPropertyKind::SpreadProperty(_) => {
                    // TODO: merge spread type properties
                }
            }
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous | ObjectFlags::ObjectLiteral,
            TypeData::Structured(StructuredType {
                member_map: build_member_map(&properties),
                properties,
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            }),
            None,
        )
    }

    /// Get the type of a static member expression (`obj.prop`).
    ///
    /// Resolves the object's type, then looks up the property by name.
    /// Handles Object, Interface, and TypeReference (via lazy instantiation).
    fn get_type_of_static_member_expression(
        &mut self,
        expr: &oxc_ast::ast::StaticMemberExpression<'_>,
    ) -> TypeId {
        let object_type = self.get_type_of_expression(&expr.object, None);
        let prop_name = expr.property.name.as_str();
        let result = self.get_property_of_type(object_type, prop_name);
        if result == self.any_type
            && !self.type_arena.get_flags(object_type).intersects(TypeFlags::Any)
        {
            let type_str = self.type_to_string(object_type);
            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Property '{prop_name}' does not exist on type '{type_str}'."
                ))
                .with_error_code("ts", "2339")
                .with_label(expr.property.span),
            );
        }
        result
    }

    /// Look up a property by name on a type. O(1) via HashMap.
    ///
    /// Handles Object, Interface, TypeReference, and Union types.
    /// For TypeReferences, resolves to the instantiated type first
    /// (cached via `instantiation_cache`).
    /// Returns `any_type` if the property is not found or the type
    /// doesn't support property access.
    pub(crate) fn get_property_of_type(&mut self, type_id: TypeId, name: &str) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        // any.prop → any
        if flags.intersects(TypeFlags::Any) {
            return self.any_type;
        }

        // Union type: look up property on each constituent, union the results
        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let prop_types: Vec<TypeId> = u.types
                    .iter()
                    .map(|&m| self.get_property_of_type(m, name))
                    .collect();
                if prop_types.iter().any(|&t| t == self.any_type) {
                    return self.any_type;
                }
                return self.get_or_create_union_type(prop_types);
            }
        }

        // Intersection type: look up property on each constituent, intersect results.
        // Property exists if found in ANY constituent (opposite of unions).
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(type_id) {
                let constituents: SmallVec<[TypeId; 4]> = i.types.clone();
                let mut prop_types = Vec::new();
                for &member in &constituents {
                    let prop = self.get_property_of_type(member, name);
                    if prop != self.any_type
                        || self.type_arena.get_flags(member).intersects(TypeFlags::Any)
                    {
                        prop_types.push(prop);
                    }
                }
                if prop_types.is_empty() {
                    return self.any_type;
                }
                if prop_types.len() == 1 {
                    return prop_types[0];
                }
                return self.get_or_create_intersection_type(prop_types);
            }
        }

        // TypeReference: resolve to instantiated type first (cached)
        let resolved_id = if let TypeData::TypeReference(_) = self.type_arena.get_data(type_id) {
            self.resolve_type_reference(type_id)
        } else {
            type_id
        };

        // O(1) HashMap lookup on resolved type
        match self.type_arena.get_data(resolved_id) {
            TypeData::Structured(s) => {
                if let Some(&prop_type) = s.member_map.get(name) {
                    return prop_type;
                }
                // Walk base types (interface inheritance)
                if let StructuredTypeKind::Interface { resolved_base_types, .. } = &s.kind {
                    for base in resolved_base_types.iter() {
                        let prop = self.get_property_of_type(*base, name);
                        if prop != self.any_type {
                            return prop;
                        }
                    }
                }
                // Fall back to index signature
                if let Some(idx_type) = s.string_index_type {
                    return idx_type;
                }
            }
            _ => {}
        }

        self.any_type
    }

    /// Get the type of an array literal expression.
    ///
    /// Infers the element type as the union of all element types,
    /// then creates an `Array<ElementType>` TypeReference.
    fn get_type_of_array_literal(
        &mut self,
        arr: &oxc_ast::ast::ArrayExpression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        use oxc_ast::ast::ArrayExpressionElement;

        // If contextual type is a tuple, check as tuple with per-position types
        if let Some(ct) = contextual_type {
            if matches!(self.type_arena.get_data(ct), TypeData::Tuple(_)) {
                return self.check_array_literal_as_tuple(arr, ct);
            }
        }

        let mut element_types = Vec::new();

        for element in &arr.elements {
            match element {
                ArrayExpressionElement::SpreadElement(spread) => {
                    // TODO: extract element type from spread
                    let spread_type = self.get_type_of_expression(&spread.argument, None);
                    element_types.push(spread_type);
                }
                ArrayExpressionElement::Elision(_) => {
                    element_types.push(self.undefined_type);
                }
                _ => {
                    // Expression elements (inherited from Expression)
                    let elem_expr = element.to_expression();
                    let elem_type = self.get_type_of_expression(elem_expr, None);
                    element_types.push(elem_type);
                }
            }
        }

        // Empty array: never[] (matching tsc's inference for uncontextualized [])
        if element_types.is_empty() {
            if self.array_type == self.any_type {
                return self.any_type;
            }
            return self.type_arena.new_type(
                TypeFlags::Object,
                ObjectFlags::Reference,
                TypeData::TypeReference(oxc_types::TypeReferenceType {
                    target: Some(self.array_type),
                    resolved_type_arguments: smallvec::smallvec![self.never_type],
                }),
                None,
            );
        }

        // Widen literal types in array elements and union them
        let widened: Vec<TypeId> = element_types
            .into_iter()
            .map(|t| self.get_widened_literal_type(t))
            .collect();
        let elem_type = self.get_or_create_union_type(widened);

        if self.array_type == self.any_type {
            return self.any_type;
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Reference,
            TypeData::TypeReference(oxc_types::TypeReferenceType {
                target: Some(self.array_type),
                resolved_type_arguments: smallvec::smallvec![elem_type],
            }),
            None,
        )
    }

    /// Check an array literal against a tuple contextual type.
    ///
    /// Each element is checked against the corresponding positional type from the
    /// contextual tuple. The result is a tuple type matching the contextual shape.
    fn check_array_literal_as_tuple(
        &mut self,
        arr: &oxc_ast::ast::ArrayExpression<'_>,
        tuple_contextual_type: TypeId,
    ) -> TypeId {
        use oxc_ast::ast::ArrayExpressionElement;

        // Access tuple data directly from the arena. The reference has lifetime 'a
        // (tied to type_arena, not to &mut self), so no clone needed.
        let TypeData::Tuple(tuple_data) = self.type_arena.get_data(tuple_contextual_type) else {
            unreachable!()
        };

        let mut element_types = smallvec::SmallVec::<[TypeId; 4]>::new();
        let mut element_infos = Vec::new();

        for (i, element) in arr.elements.iter().enumerate() {
            let ctx_elem_type = tuple_data.resolved_type_arguments.get(i).copied();

            let elem_type = match element {
                ArrayExpressionElement::SpreadElement(spread) => {
                    self.get_type_of_expression(&spread.argument, None)
                }
                ArrayExpressionElement::Elision(_) => self.undefined_type,
                _ => {
                    let elem_expr = element.to_expression();
                    self.get_type_of_expression(elem_expr, ctx_elem_type)
                }
            };

            element_types.push(elem_type);
            element_infos.push(oxc_types::TupleElementInfo {
                element_type: elem_type,
                flags: tuple_data
                    .element_infos
                    .get(i)
                    .map(|info| info.flags)
                    .unwrap_or(oxc_types::ElementFlags::Required),
                label_name: tuple_data.element_infos.get(i).and_then(|info| info.label_name.clone()),
            });
        }

        let min_length = element_types.len() as u32;
        let fixed_length = min_length;

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Tuple | ObjectFlags::Reference,
            TypeData::Tuple(oxc_types::TupleType {
                target: None,
                resolved_type_arguments: element_types,
                element_infos,
                min_length,
                fixed_length,
                combined_flags: oxc_types::ElementFlags::Required,
                readonly: false,
            }),
            None,
        )
    }

    /// Get the type of a chain expression (`foo?.bar`, `foo?.()`).
    ///
    /// Resolves the inner expression type and unions with `undefined`
    /// since the chain may short-circuit.
    fn get_type_of_chain_expression(&mut self, chain: &ChainExpression<'_>) -> TypeId {
        use oxc_ast::ast::ChainElement;

        let inner_type = match &chain.expression {
            ChainElement::StaticMemberExpression(e) => {
                self.get_type_of_static_member_expression(e)
            }
            ChainElement::ComputedMemberExpression(e) => {
                self.get_type_of_computed_member_expression(e)
            }
            ChainElement::TSNonNullExpression(e) => self.get_type_of_expression(&e.expression, None),
            _ => self.any_type, // CallExpression, PrivateFieldExpression
        };
        self.get_or_create_union_type(vec![inner_type, self.undefined_type])
    }

    /// Get the type of a computed member expression (`obj["key"]`, `obj[0]`).
    ///
    /// For string literal keys, performs a property lookup on the object type.
    fn get_type_of_computed_member_expression(
        &mut self,
        expr: &ComputedMemberExpression<'_>,
    ) -> TypeId {
        let object_type = self.get_type_of_expression(&expr.object, None);
        // String literal index → property lookup
        if let Expression::StringLiteral(lit) = &expr.expression {
            let result = self.get_property_of_type(object_type, &lit.value);
            if result == self.any_type
                && !self.type_arena.get_flags(object_type).intersects(TypeFlags::Any)
            {
                let type_str = self.type_to_string(object_type);
                let prop_name = &lit.value;
                self.diagnostics.push(
                    OxcDiagnostic::error(format!(
                        "Property '{prop_name}' does not exist on type '{type_str}'."
                    ))
                    .with_error_code("ts", "2339")
                    .with_label(lit.span()),
                );
            }
            return result;
        }
        // TODO: numeric index on arrays/tuples, keyof, index signatures
        self.any_type
    }

    /// Get the result type of a binary expression.
    fn get_type_of_binary_expression(&mut self, expr: &BinaryExpression<'_>) -> TypeId {
        match expr.operator {
            // Comparison and equality operators always return boolean
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::In
            | BinaryOperator::Instanceof => self.boolean_type,

            // Arithmetic operators (not +) return number or bigint
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd => {
                let left_type = self.get_type_of_expression(&expr.left, None);
                let right_type = self.get_type_of_expression(&expr.right, None);
                let left_flags = self.type_arena.get_flags(left_type);
                let right_flags = self.type_arena.get_flags(right_type);
                if left_flags.intersects(TypeFlags::BigIntLike)
                    && right_flags.intersects(TypeFlags::BigIntLike)
                {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }

            // + is special: string concat if either side is string-like, otherwise number
            BinaryOperator::Addition => {
                let left_type = self.get_type_of_expression(&expr.left, None);
                let right_type = self.get_type_of_expression(&expr.right, None);
                let left_flags = self.type_arena.get_flags(left_type);
                let right_flags = self.type_arena.get_flags(right_type);
                if left_flags.intersects(TypeFlags::StringLike)
                    || right_flags.intersects(TypeFlags::StringLike)
                {
                    self.string_type
                } else if left_flags.intersects(TypeFlags::BigIntLike)
                    && right_flags.intersects(TypeFlags::BigIntLike)
                {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }
        }
    }

    /// Get the type of a call expression, checking for TS2349, TS2554, TS2345.
    fn get_type_of_call_expression(
        &mut self,
        call: &oxc_ast::ast::CallExpression<'_>,
    ) -> TypeId {
        use oxc_ast::ast::Argument;

        let callee_type = self.get_type_of_expression(&call.callee, None);
        let callee_flags = self.type_arena.get_flags(callee_type);

        // any(...) → any
        if callee_flags.intersects(TypeFlags::Any) {
            // Still evaluate argument expressions for side-effect diagnostics
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    self.get_type_of_expression(expr, None);
                }
            }
            return self.any_type;
        }

        // Extract call signatures from the callee type.
        // Returns a slice reference — stable because type_arena uses AppendOnlyVec.
        let empty_sigs: &[oxc_types::Signature] = &[];
        let signatures: &[oxc_types::Signature] = match self.type_arena.get_data(callee_type) {
            TypeData::Function(f) => &f.signatures,
            TypeData::Structured(s) => &s.call_signatures,
            _ => empty_sigs,
        };

        if signatures.is_empty() {
            // TS2349: This expression is not callable
            let type_str = self.type_to_string(callee_type);
            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "This expression is not callable.\n  Type '{type_str}' has no call signatures."
                ))
                .with_error_code("ts", "2349")
                .with_label(call.callee.span()),
            );
            // Still evaluate arguments for diagnostics
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    self.get_type_of_expression(expr, None);
                }
            }
            return self.any_type;
        }

        // Use the first signature (overload resolution deferred).
        // No clone needed: `signatures` borrows from `self.type_arena` (lifetime 'a),
        // which is independent of `&mut self` borrows on the checker.
        let sig = &signatures[0];

        // Extract signature info before any &mut self calls.
        let sig_flags = sig.flags;
        let sig_min_args = sig.min_argument_count as usize;
        let sig_param_count = sig.parameters.len();
        let sig_return_type = sig.return_type;
        let sig_type_params: smallvec::SmallVec<[TypeId; 4]> = sig.type_parameters.clone();

        // Extract parameter TypeIds (SmallVec, stack-allocated for ≤8 params)
        let param_type_ids: smallvec::SmallVec<[TypeId; 8]> = sig
            .parameters
            .iter()
            .map(|p| p.type_id)
            .collect();

        // Count non-rest parameters for max argument count
        let max_args = if sig_flags.intersects(oxc_types::SignatureFlags::HasRestParameter) {
            usize::MAX
        } else {
            sig_param_count
        };
        let actual_args = call.arguments.len();

        // TS2554: Expected N arguments, but got M
        if actual_args < sig_min_args || actual_args > max_args {
            let expected = if sig_min_args == max_args {
                format!("{}", sig_min_args)
            } else if max_args == usize::MAX {
                format!("at least {}", sig_min_args)
            } else {
                format!("{}-{max_args}", sig_min_args)
            };
            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Expected {expected} arguments, but got {actual_args}."
                ))
                .with_error_code("ts", "2554")
                .with_label(call.span),
            );
        }

        // --- Evaluate argument types once ---
        // Collect all argument types up front to avoid double evaluation.
        // For generic calls, these are used for both inference and checking.
        let arg_types: smallvec::SmallVec<[TypeId; 8]> = call
            .arguments
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let param_ctx = self.get_param_type_at(&param_type_ids, i, sig_flags);
                match arg {
                    Argument::SpreadElement(spread) => {
                        self.get_type_of_expression(&spread.argument, None)
                    }
                    _ => {
                        let expr = arg.to_expression();
                        self.get_type_of_expression(expr, param_ctx)
                    }
                }
            })
            .collect();

        // --- Generic call handling ---
        // If the signature has type parameters, infer or use explicit type arguments,
        // then instantiate parameter/return types before checking.
        let (effective_param_types, effective_return_type) = if !sig_type_params.is_empty() {
            let type_args = if let Some(type_arg_node) = &call.type_arguments {
                // Explicit type arguments: id<string>("hello")
                type_arg_node
                    .params
                    .iter()
                    .map(|t| self.get_type_from_type_node(t))
                    .collect::<smallvec::SmallVec<[TypeId; 4]>>()
            } else {
                // Infer from arguments
                let mut infer_ctx =
                    crate::inference::InferenceContext::new(&sig_type_params);

                for (i, &arg_type) in arg_types.iter().enumerate() {
                    let raw_param_type = self.get_param_type_at(
                        &param_type_ids, i, sig_flags,
                    );
                    if let Some(param_type) = raw_param_type {
                        self.infer_from_types(&mut infer_ctx, arg_type, param_type);
                    }
                }
                self.get_inferred_types(&mut infer_ctx)
            };

            if let Some(mapper) =
                crate::instantiation::TypeMapper::from_type_parameters(&sig_type_params, &type_args)
            {
                let instantiated_params: smallvec::SmallVec<[TypeId; 8]> = param_type_ids
                    .iter()
                    .map(|&p| self.instantiate_type(p, &mapper))
                    .collect();
                let instantiated_return = self.instantiate_type(sig_return_type, &mapper);
                (instantiated_params, instantiated_return)
            } else {
                (param_type_ids, sig_return_type)
            }
        } else {
            (param_type_ids, sig_return_type)
        };

        // --- Argument type checking (TS2345) ---
        // Check each argument against the (possibly instantiated) parameter type.
        for (i, (arg, &arg_type)) in call.arguments.iter().zip(arg_types.iter()).enumerate() {
            let param_type = self.get_param_type_at(
                &effective_param_types, i, sig_flags,
            );

            if let Some(param_type) = param_type {
                if !self.type_arena.get_flags(param_type).intersects(TypeFlags::Any) {
                    if !self.is_type_assignable_to(arg_type, param_type) {
                        let arg_str = self.type_to_string(arg_type);
                        let param_str = self.type_to_string(param_type);
                        let span = match arg {
                            Argument::SpreadElement(s) => s.span,
                            _ => arg.to_expression().span(),
                        };
                        self.diagnostics.push(
                            OxcDiagnostic::error(format!(
                                "Argument of type '{arg_str}' is not assignable to parameter of type '{param_str}'."
                            ))
                            .with_error_code("ts", "2345")
                            .with_label(span),
                        );
                    }
                }
            }
        }

        effective_return_type
    }

    /// Get the parameter TypeId at a given argument index, handling rest parameters.
    fn get_param_type_at(
        &self,
        param_type_ids: &[TypeId],
        index: usize,
        sig_flags: oxc_types::SignatureFlags,
    ) -> Option<TypeId> {
        if index < param_type_ids.len() {
            Some(param_type_ids[index])
        } else if sig_flags.intersects(oxc_types::SignatureFlags::HasRestParameter) {
            param_type_ids.last().copied()
        } else {
            None
        }
    }

    /// Get the type of a `new` expression (`new Foo()`).
    ///
    /// If the callee resolves to a class, returns the class's instance type
    /// (declared type). Otherwise returns `any`.
    fn get_type_of_new_expression(
        &mut self,
        new_expr: &oxc_ast::ast::NewExpression<'_>,
    ) -> TypeId {
        use oxc_ast::AstKind;

        // Resolve callee: only handle simple identifiers for now
        let Expression::Identifier(ident) = &new_expr.callee else {
            return self.any_type;
        };

        let Some(reference_id) = ident.reference_id.get() else {
            return self.any_type;
        };

        let reference = self.semantic().scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else {
            return self.any_type;
        };

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        match node.kind() {
            AstKind::Class(_) => self.get_declared_type_of_symbol(symbol_id),
            _ => self.any_type,
        }
    }

    /// Resolve the type of an import binding via the host.
    ///
    /// Walks from the import specifier's declaration node up to its parent
    /// ImportDeclaration to extract the module specifier and import name,
    /// then calls `host.resolve_import()`.
    pub(crate) fn resolve_import_type(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        // Extract the imported name from the specifier
        let export_name = match node.kind() {
            AstKind::ImportSpecifier(spec) => {
                spec.imported.name().to_string()
            }
            AstKind::ImportDefaultSpecifier(_) => {
                "default".to_string()
            }
            _ => return self.any_type,
        };

        // Walk up to the ImportDeclaration to get the module specifier
        let parent_id = self.semantic().nodes().parent_id(node_id);
        let parent = self.semantic().nodes().get_node(parent_id);
        let AstKind::ImportDeclaration(import_decl) = parent.kind() else {
            return self.any_type;
        };

        let module_specifier = import_decl.source.value.as_str();
        self.host.resolve_import(&self.file_path, module_specifier, &export_name)
            .unwrap_or(self.any_type)
    }
}
