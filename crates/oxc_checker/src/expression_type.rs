use oxc_ast::ast::{BinaryExpression, ChainExpression, Expression, UnaryExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{CompactStr, GetSpan};
use oxc_syntax::node::NodeId;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use oxc_syntax::symbol::SymbolId;
use oxc_types::{
    ObjectFlags, PropertyInfo, StructuredType, StructuredTypeKind, TypeData, TypeFlags, TypeId,
    sort_properties,
};

use crate::Checker;
use crate::checker::CheckMode;
use crate::nullable::NullableErrorReporter;

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
        check_mode: CheckMode,
    ) -> TypeId {
        let span = expr.span();
        let key = (span.start as u64) << 32 | span.end as u64;

        // Post-checking fallback: read the cache populated during check_program().
        // Only used when the flow graph is gone (post-check) and no contextual type
        // is requested, to avoid interfering with live checking. Matches tsgo's
        // getTypeOfExpression reading flowTypeCache before computing.
        if self.current_flow_graph.node_flow_map.is_empty() && contextual_type.is_none() {
            if let Some(&cached) = self.expression_type_cache.get(&key) {
                return cached;
            }
        }

        // Guard against infinite recursion (e.g., `const x = x`)
        if self.recursion_depth > 100 {
            return self.any_type;
        }
        self.recursion_depth += 1;
        let result = self.get_type_of_expression_inner(expr, contextual_type, check_mode);
        self.recursion_depth -= 1;

        // Cache if we're inside check_program() (flow graph is active).
        // This captures flow-narrowed and contextually-typed results so
        // post-checking queries return the same types the checker computed.
        if !self.current_flow_graph.node_flow_map.is_empty() {
            self.expression_type_cache.insert(key, result);
        }

        result
    }

    fn get_type_of_expression_inner(
        &mut self,
        expr: &Expression<'_>,
        contextual_type: Option<TypeId>,
        check_mode: CheckMode,
    ) -> TypeId {
        match expr {
            Expression::StringLiteral(lit) => {
                let regular = self.get_or_create_string_literal_type(&lit.value);
                self.get_fresh_type_of_literal(regular)
            }
            Expression::NumericLiteral(lit) => {
                let regular = self.get_or_create_number_literal_type(lit.value);
                self.get_fresh_type_of_literal(regular)
            }
            Expression::BigIntLiteral(lit) => {
                let regular = self.get_or_create_bigint_literal_type(lit.value.as_str());
                self.get_fresh_type_of_literal(regular)
            }
            Expression::BooleanLiteral(lit) => {
                let regular = if lit.value { self.true_type } else { self.false_type };
                self.get_fresh_type_of_literal(regular)
            }
            Expression::NullLiteral(_) => self.null_widening_type,
            Expression::Identifier(ident) => self.get_type_of_identifier(ident),
            Expression::ParenthesizedExpression(paren) => {
                self.get_type_of_expression(&paren.expression, contextual_type, check_mode)
            }
            // Type assertions
            Expression::TSAsExpression(expr) => self.check_assertion(
                &expr.expression,
                &expr.type_annotation,
                contextual_type,
                check_mode,
            ),
            Expression::TSTypeAssertion(expr) => self.check_assertion(
                &expr.expression,
                &expr.type_annotation,
                contextual_type,
                check_mode,
            ),
            // `satisfies` checks but returns the expression's type, not the annotation
            Expression::TSSatisfiesExpression(expr) => {
                self.get_type_of_expression(&expr.expression, contextual_type, check_mode)
            }
            // Non-null assertion — remove null/undefined from the expression type
            Expression::TSNonNullExpression(expr) => {
                let type_id =
                    self.get_type_of_expression(&expr.expression, contextual_type, check_mode);
                self.get_non_nullable_type(type_id)
            }

            // Unary expressions
            Expression::UnaryExpression(expr) => {
                self.get_type_of_unary_expression(expr, check_mode)
            }

            // Binary expressions
            Expression::BinaryExpression(expr) => {
                self.get_type_of_binary_expression(expr, check_mode)
            }

            // Conditional (ternary) — union of both branches
            Expression::ConditionalExpression(expr) => {
                let true_type =
                    self.get_type_of_expression(&expr.consequent, contextual_type, check_mode);
                let false_type =
                    self.get_type_of_expression(&expr.alternate, contextual_type, check_mode);
                self.get_or_create_union_type(vec![true_type, false_type])
            }

            // Template literals — always string (simplified; tsc can produce literal types)
            Expression::TemplateLiteral(_) => self.string_type,

            // Sequence expression — checks all sub-expressions, emits TS2695,
            // returns type of the last element.
            Expression::SequenceExpression(seq) => {
                self.check_sequence_expression(seq, contextual_type, check_mode)
            }

            // void x — always undefined
            // (handled in unary, but keeping note)

            // Logical expressions — simplified to union of both sides
            Expression::LogicalExpression(expr) => {
                let left_type =
                    self.get_type_of_expression(&expr.left, contextual_type, check_mode);
                let right_type =
                    self.get_type_of_expression(&expr.right, contextual_type, check_mode);
                self.get_or_create_union_type(vec![left_type, right_type])
            }

            // ++x, x++ etc — returns number or bigint depending on operand
            // TODO: add check_non_null_type once it supports SimpleAssignmentTarget
            Expression::UpdateExpression(update) => {
                self.check_update_expression(update, check_mode)
            }

            // Object literal: `{ x: 1, y: "hello" }`
            Expression::ObjectExpression(obj) => {
                self.get_type_of_object_literal(obj, contextual_type, check_mode)
            }

            // Property access: foo.bar / foo?.bar
            // Null check at dispatch; optional chains skip it.
            Expression::StaticMemberExpression(expr) => {
                let object_type = self.get_type_of_expression(&expr.object, None, check_mode);
                let object_type = if expr.optional {
                    object_type
                } else {
                    self.check_non_null_type(object_type, &expr.object)
                };
                self.resolve_static_member_type(object_type, expr)
            }

            // Array literal: [1, 2, 3]
            Expression::ArrayExpression(arr) => {
                self.get_type_of_array_literal(arr, contextual_type, check_mode)
            }

            // Assignment: checks assignability (TS2322), returns RHS type.
            Expression::AssignmentExpression(assign) => {
                self.check_assignment_expression(assign, contextual_type, check_mode)
            }

            // Optional chaining: unwrap inner, union with undefined
            Expression::ChainExpression(chain) => {
                self.get_type_of_chain_expression(chain, check_mode)
            }

            // Computed member access: obj["key"] / obj?.["key"]
            // Null check at dispatch; optional chains skip it.
            Expression::ComputedMemberExpression(expr) => {
                let object_type = self.get_type_of_expression(&expr.object, None, check_mode);
                let object_type = if expr.optional {
                    object_type
                } else {
                    self.check_non_null_type(object_type, &expr.object)
                };
                self.resolve_computed_member_type(object_type, expr, check_mode)
            }

            // await expr — unwrap Promise<T> to T
            Expression::AwaitExpression(expr) => {
                let operand_type = self.get_type_of_expression(&expr.argument, None, check_mode);
                self.get_awaited_type(operand_type, expr.span)
            }

            // /regex/ — always RegExp
            Expression::RegExpLiteral(_) => self.get_global_type("RegExp"),

            // this — resolve from AST ancestors: find the enclosing class,
            // stopping at regular functions (which reset `this`).
            // Arrow functions inherit `this` so we skip past them.
            Expression::ThisExpression(this_expr) => self.resolve_this_type(this_expr.node_id()),

            // These need more infrastructure (call signatures, generators, modules)
            Expression::YieldExpression(_)
            | Expression::ImportExpression(_)
            | Expression::TaggedTemplateExpression(_) => self.any_type,

            Expression::ArrowFunctionExpression(arrow) => {
                // Queue body for deferred checking (after enclosing scope is resolved).
                self.queue_deferred_body(arrow.node_id());
                // Inline arena access to get contextual signature without
                // borrowing &self (avoids clone). self.type_arena is &'a,
                // so the returned reference has lifetime 'a.
                let contextual_sig =
                    contextual_type.and_then(|c| match self.type_arena.get_data(c) {
                        TypeData::Function(f) => f.signatures.first(),
                        TypeData::Structured(s) => s.call_signatures.first(),
                        _ => None,
                    });
                // Create type parameters before building the signature so that
                // references to T in parameter/return types resolve correctly.
                let type_parameters =
                    self.get_type_parameters_from_declaration(arrow.type_parameters.as_deref());
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
                        let raw = if let Some(oxc_ast::ast::Statement::ExpressionStatement(
                            expr_stmt,
                        )) = arrow.body.statements.first()
                        {
                            self.get_type_of_expression(
                                &expr_stmt.expression,
                                return_contextual_type,
                                check_mode,
                            )
                        } else {
                            self.void_type
                        };
                        // Apply return-type widening (expression body doesn't go
                        // through infer_return_type_from_body, so widen here).
                        self.widen_return_type(raw, return_contextual_type)
                    } else {
                        // Block body: () => { ... } — infer from return statements
                        // (widening is applied inside infer_return_type_from_body)
                        self.infer_return_type_from_body(
                            &arrow.body.statements,
                            return_contextual_type,
                        )
                    };
                }
                self.create_function_type(sig)
            }

            Expression::FunctionExpression(func) => {
                // Queue body for deferred checking (after enclosing scope is resolved).
                self.queue_deferred_body(func.node_id());
                let contextual_sig =
                    contextual_type.and_then(|c| match self.type_arena.get_data(c) {
                        TypeData::Function(f) => f.signatures.first(),
                        TypeData::Structured(s) => s.call_signatures.first(),
                        _ => None,
                    });
                let sig = self.build_signature_from_function_with_context(func, contextual_sig);
                self.create_function_type(sig)
            }

            Expression::CallExpression(call) => self.get_type_of_call_expression(call, check_mode),

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
    /// Emits TS2454 if the variable is used before being assigned.
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

        // Skip TS2454 for write-only references (the variable is being assigned to).
        let is_write_only = reference.is_write() && !reference.is_read();
        // Extract scope_id before mutable borrows
        let ref_scope_id = reference.scope_id();
        if is_write_only {
            return self.get_type_of_symbol(symbol_id);
        }

        let declared_type = self.get_type_of_symbol(symbol_id);

        // Determine if this variable needs definite-assignment checking.
        // Symbol-level check is cached; scope walk is per-reference.
        let (is_potentially_uninit, cached_initial_type) =
            self.get_definite_assignment_info(symbol_id, declared_type);
        let needs_assignment_check =
            is_potentially_uninit && !self.is_outer_variable(symbol_id, ref_scope_id);
        let initial_type = if needs_assignment_check { cached_initial_type } else { declared_type };

        // Apply control flow narrowing.
        let flow_type = self.get_flow_type_of_reference(
            ident.node_id.get(),
            symbol_id,
            initial_type,
            declared_type,
        );

        // TS2454: Variable used before being assigned.
        // If the declared type doesn't include undefined but the flow type does,
        // the variable hasn't been assigned on all code paths reaching here.
        if needs_assignment_check
            && !self.contains_undefined_type(declared_type)
            && self.contains_undefined_type(flow_type)
        {
            let name = self.semantic().scoping().symbol_name(symbol_id).to_string();
            self.diagnostics.push(
                OxcDiagnostic::error(format!("Variable '{name}' is used before being assigned."))
                    .with_error_code("ts", "2454")
                    .with_label(ident.span),
            );
            // Return declared type to reduce follow-on errors.
            return declared_type;
        }

        flow_type
    }

    /// Get cached definite-assignment info for a symbol.
    ///
    /// Returns `(is_potentially_uninitialized, initial_type)`.
    /// Computed once per symbol and cached. The scope walk (outer-variable
    /// check) is NOT included here — it's per-reference.
    fn get_definite_assignment_info(
        &mut self,
        symbol_id: SymbolId,
        declared_type: TypeId,
    ) -> (bool, TypeId) {
        if let Some(cached) = self.definite_assignment_cache[symbol_id] {
            return cached;
        }
        let is_uninit = self.is_symbol_potentially_uninitialized(symbol_id);
        let initial_type =
            if is_uninit { self.get_optional_type(declared_type) } else { declared_type };
        let result = (is_uninit, initial_type);
        self.definite_assignment_cache[symbol_id] = Some(result);
        result
    }

    /// Symbol-level check: is this variable declared without an initializer?
    ///
    /// Returns true for `let x: T;` / `var x: T;` (no init), but NOT for
    /// parameters, for-in/for-of loop variables, ambient declarations,
    /// variables with `!` assertions, or any/unknown-typed variables.
    ///
    /// Does NOT check whether the reference is from a nested scope
    /// (that's `is_outer_variable`, done per-reference).
    fn is_symbol_potentially_uninitialized(&self, symbol_id: SymbolId) -> bool {
        use oxc_ast::AstKind;

        let symbol_flags = self.semantic().scoping().symbol_flags(symbol_id);
        if !symbol_flags.is_variable() {
            return false;
        }

        // Ambient declarations (declare var/let) are assumed to be initialized
        if symbol_flags.is_ambient() {
            return false;
        }

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        let AstKind::VariableDeclarator(decl) = node.kind() else {
            return false;
        };

        // Has initializer → not uninitialized
        if decl.init.is_some() {
            return false;
        }

        // Has definite-assignment assertion (x!: T) → skip
        if decl.definite {
            return false;
        }

        // Check for for-in/for-of loop context — those variables are always assigned
        let parent_id = self.semantic().nodes().parent_id(node_id);
        let grandparent_id = self.semantic().nodes().parent_id(parent_id);
        let grandparent = self.semantic().nodes().get_node(grandparent_id);
        if matches!(grandparent.kind(), AstKind::ForInStatement(_) | AstKind::ForOfStatement(_)) {
            return false;
        }

        // Declared type must not be any/unknown (those are always "safe")
        if let Some(dt) = self.symbol_type_cache[symbol_id] {
            let flags = self.type_arena.get_flags(dt);
            if flags.intersects(TypeFlags::Any | TypeFlags::Unknown) {
                return false;
            }
        }

        true
    }

    /// Per-reference check: is this reference from a nested function scope?
    ///
    /// If the reference crosses a function boundary relative to the
    /// declaration, the variable is assumed initialized (tsc's
    /// `isOuterVariable && !isNeverInitialized` logic).
    fn is_outer_variable(
        &self,
        symbol_id: SymbolId,
        ref_scope_id: oxc_syntax::scope::ScopeId,
    ) -> bool {
        use oxc_syntax::scope::ScopeFlags;

        let decl_scope = self.semantic().scoping().symbol_scope_id(symbol_id);
        if decl_scope == ref_scope_id {
            return false;
        }

        let scoping = self.semantic().scoping();
        for ancestor_scope in scoping.scope_ancestors(ref_scope_id) {
            if ancestor_scope == decl_scope {
                return false; // Reached declaration scope without crossing function boundary
            }
            let flags = scoping.scope_flags(ancestor_scope);
            if flags.intersects(ScopeFlags::Function | ScopeFlags::Arrow) {
                return true; // Crossed a function boundary → outer variable
            }
        }
        false
    }

    /// Resolve the type of `this` by walking up AST ancestors from the given node.
    ///
    /// - Stops at a `Class` node and returns its declared type.
    /// - Stops at a standalone `Function` (which resets `this` context) and returns
    ///   the generic `this` type parameter.
    /// - Skips `Function` nodes that belong to a `MethodDefinition` (class methods).
    /// - Skips `ArrowFunctionExpression` nodes (arrows inherit `this`).
    pub(crate) fn resolve_this_type(&mut self, node_id: NodeId) -> TypeId {
        use oxc_ast::AstKind;

        // First pass: walk ancestors (immutable borrow) to find the class symbol.
        let class_symbol = {
            let mut saw_function = false;
            let mut result = None;
            for ancestor in self.semantic().nodes().ancestor_kinds(node_id) {
                match ancestor {
                    AstKind::Class(class) => {
                        result = class.id.as_ref().and_then(|id| id.symbol_id.get());
                        break;
                    }
                    AstKind::Function(_) => {
                        saw_function = true;
                    }
                    AstKind::MethodDefinition(_) => {
                        saw_function = false;
                    }
                    AstKind::ArrowFunctionExpression(_) => {}
                    AstKind::Program(_) => break,
                    _ => {
                        if saw_function {
                            break;
                        }
                    }
                }
            }
            result
        };

        // Second pass: resolve the symbol to a type (mutable borrow).
        if let Some(symbol_id) = class_symbol {
            self.get_declared_type_of_symbol(symbol_id)
        } else {
            self.this_type
        }
    }

    /// Get the result type of a unary expression.
    fn get_type_of_unary_expression(
        &mut self,
        expr: &UnaryExpression<'_>,
        check_mode: CheckMode,
    ) -> TypeId {
        match expr.operator {
            // typeof always returns a string
            UnaryOperator::Typeof => self.string_type,
            // void always returns undefined
            UnaryOperator::Void => self.undefined_type,
            // ! returns boolean
            UnaryOperator::LogicalNot => self.boolean_type,
            // delete returns boolean
            UnaryOperator::Delete => self.boolean_type,
            // +x returns number; preserves literal type for numeric literals
            UnaryOperator::UnaryPlus => {
                if let Expression::NumericLiteral(lit) = &expr.argument {
                    let regular = self.get_or_create_number_literal_type(lit.value);
                    return self.get_fresh_type_of_literal(regular);
                }
                let operand_type = self.get_type_of_expression(&expr.argument, None, check_mode);
                self.check_non_null_type(operand_type, &expr.argument);
                self.number_type
            }
            // -x returns number or bigint; preserves literal type for numeric/bigint literals
            UnaryOperator::UnaryNegation => {
                if let Expression::NumericLiteral(lit) = &expr.argument {
                    let regular = self.get_or_create_number_literal_type(-lit.value);
                    return self.get_fresh_type_of_literal(regular);
                }
                if let Expression::BigIntLiteral(lit) = &expr.argument {
                    let negated = format!("-{}", lit.value.as_str());
                    let regular = self.get_or_create_bigint_literal_type(&negated);
                    return self.get_fresh_type_of_literal(regular);
                }
                let operand_type = self.get_type_of_expression(&expr.argument, None, check_mode);
                self.check_non_null_type(operand_type, &expr.argument);
                let operand_flags = self.type_arena.get_flags(operand_type);
                if operand_flags.intersects(TypeFlags::BigIntLike) {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }
            // ~x returns number or bigint depending on operand
            UnaryOperator::BitwiseNot => {
                let operand_type = self.get_type_of_expression(&expr.argument, None, check_mode);
                self.check_non_null_type(operand_type, &expr.argument);
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
    /// Uses a left-fold pattern when spreads are present: properties between
    /// spreads are grouped into segments, and each segment is merged into
    /// an accumulator via `get_spread_type`. When no spreads exist, takes a
    /// fast path that builds the type directly.
    fn get_type_of_object_literal(
        &mut self,
        obj: &oxc_ast::ast::ObjectExpression<'_>,
        contextual_type: Option<TypeId>,
        check_mode: CheckMode,
    ) -> TypeId {
        use oxc_ast::ast::{ObjectPropertyKind, PropertyKind};

        let mut spread = self.empty_object_type;
        let mut properties = Vec::new();
        let mut has_spread = false;

        for prop_kind in &obj.properties {
            match prop_kind {
                ObjectPropertyKind::ObjectProperty(prop) => {
                    if prop.kind != PropertyKind::Init {
                        continue;
                    }
                    if let Some(name) = prop.key.static_name() {
                        let prop_contextual_type =
                            contextual_type.and_then(|ct| self.get_property_of_type(ct, &name));
                        let prop_type = self.get_type_of_expression(
                            &prop.value,
                            prop_contextual_type,
                            check_mode,
                        );
                        let widened = self.get_widened_literal_type(prop_type);
                        properties.push(PropertyInfo::new(CompactStr::new(&name), widened));
                    }
                }
                ObjectPropertyKind::SpreadProperty(spread_prop) => {
                    has_spread = true;
                    // Flush accumulated properties into the spread accumulator
                    if !properties.is_empty() {
                        let segment =
                            self.create_object_literal_type(std::mem::take(&mut properties));
                        spread = self.get_spread_type(spread, segment);
                    }
                    let spread_type =
                        self.get_type_of_expression(&spread_prop.argument, None, check_mode);
                    if self.is_valid_spread_type(spread_type) {
                        spread = self.get_spread_type(spread, spread_type);
                    } else {
                        self.diagnostics.push(
                            OxcDiagnostic::error(
                                "Spread types may only be created from object types.",
                            )
                            .with_error_code("ts", "2698")
                            .with_label(spread_prop.argument.span()),
                        );
                        // Use any_type to suppress cascading errors
                        spread = self.any_type;
                    }
                }
            }
        }

        if !has_spread {
            // Fast path: no spreads, same as before
            return self.create_object_literal_type(properties);
        }

        // Flush any remaining properties after the last spread
        if !properties.is_empty() {
            let segment = self.create_object_literal_type(std::mem::take(&mut properties));
            spread = self.get_spread_type(spread, segment);
        }

        // If no spread was actually merged, produce a fresh empty literal
        if spread == self.empty_object_type {
            return self.create_object_literal_type(Vec::new());
        }

        spread
    }

    /// Create a fresh object literal type from a list of properties.
    fn create_object_literal_type(&mut self, mut properties: Vec<PropertyInfo>) -> TypeId {
        sort_properties(&mut properties);
        // Propagate widening/inferrability flags from property types upward
        // (e.g., ContainsWideningType from null/undefined widening properties).
        // Computed inline to avoid allocating an intermediate Vec of TypeIds.
        let mut propagated = ObjectFlags::None;
        for p in &properties {
            propagated |= self.type_arena.get_object_flags(p.type_id);
        }
        propagated &= ObjectFlags::PropagatingFlags;
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous
                | ObjectFlags::ObjectLiteral
                | ObjectFlags::FreshLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral
                | propagated,
            TypeData::Structured(Box::new(StructuredType {
                properties,
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            })),
            None,
        )
    }

    /// Get the type of an array literal expression.
    ///
    /// Infers the element type as the union of all element types,
    /// then creates an `Array<ElementType>` TypeReference.
    fn get_type_of_array_literal(
        &mut self,
        arr: &oxc_ast::ast::ArrayExpression<'_>,
        contextual_type: Option<TypeId>,
        check_mode: CheckMode,
    ) -> TypeId {
        use oxc_ast::ast::ArrayExpressionElement;

        // If contextual type is a tuple, check as tuple with per-position types
        if let Some(ct) = contextual_type {
            if matches!(self.type_arena.get_data(ct), TypeData::Tuple(_)) {
                return self.check_array_literal_as_tuple(arr, ct, check_mode);
            }
        }

        let mut element_types = Vec::new();

        for element in &arr.elements {
            match element {
                ArrayExpressionElement::SpreadElement(spread) => {
                    // TODO: extract element type from spread
                    let spread_type =
                        self.get_type_of_expression(&spread.argument, None, check_mode);
                    element_types.push(spread_type);
                }
                ArrayExpressionElement::Elision(_) => {
                    element_types.push(self.undefined_type);
                }
                _ => {
                    // Expression elements (inherited from Expression)
                    let elem_expr = element.to_expression();
                    let elem_type = self.get_type_of_expression(elem_expr, None, check_mode);
                    element_types.push(elem_type);
                }
            }
        }

        // Empty array: never[] (matching tsc's inference for uncontextualized [])
        if element_types.is_empty() {
            if self.array_type == self.any_type {
                return self.any_type;
            }
            return self.get_or_create_type_reference(
                self.array_type,
                smallvec::smallvec![self.never_type],
            );
        }

        // Widen literal types in array elements and union them
        let widened: Vec<TypeId> =
            element_types.into_iter().map(|t| self.get_widened_literal_type(t)).collect();
        let elem_type = self.get_or_create_union_type(widened);

        if self.array_type == self.any_type {
            return self.any_type;
        }

        self.get_or_create_type_reference(self.array_type, smallvec::smallvec![elem_type])
    }

    /// Check an array literal against a tuple contextual type.
    ///
    /// Each element is checked against the corresponding positional type from the
    /// contextual tuple. The result is a tuple type matching the contextual shape.
    fn check_array_literal_as_tuple(
        &mut self,
        arr: &oxc_ast::ast::ArrayExpression<'_>,
        tuple_contextual_type: TypeId,
        check_mode: CheckMode,
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
                    self.get_type_of_expression(&spread.argument, None, check_mode)
                }
                ArrayExpressionElement::Elision(_) => self.undefined_type,
                _ => {
                    let elem_expr = element.to_expression();
                    self.get_type_of_expression(elem_expr, ctx_elem_type, check_mode)
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
                label_name: tuple_data
                    .element_infos
                    .get(i)
                    .and_then(|info| info.label_name.clone()),
            });
        }

        let min_length = element_types.len() as u32;
        let fixed_length = min_length;

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Tuple | ObjectFlags::Reference,
            TypeData::Tuple(Box::new(oxc_types::TupleType {
                target: None,
                resolved_type_arguments: element_types,
                element_infos,
                min_length,
                fixed_length,
                combined_flags: oxc_types::ElementFlags::Required,
                readonly: false,
            })),
            None,
        )
    }

    /// Get the type of a chain expression (`foo?.bar`, `foo?.()`).
    ///
    /// Resolves the inner expression type and unions with `undefined`
    /// since the chain may short-circuit.
    fn get_type_of_chain_expression(
        &mut self,
        chain: &ChainExpression<'_>,
        check_mode: CheckMode,
    ) -> TypeId {
        use oxc_ast::ast::ChainElement;

        let inner_type = match &chain.expression {
            ChainElement::StaticMemberExpression(e) => {
                let object_type = self.get_type_of_expression(&e.object, None, check_mode);
                self.resolve_static_member_type(object_type, e)
            }
            ChainElement::ComputedMemberExpression(e) => {
                let object_type = self.get_type_of_expression(&e.object, None, check_mode);
                self.resolve_computed_member_type(object_type, e, check_mode)
            }
            ChainElement::TSNonNullExpression(e) => {
                self.get_type_of_expression(&e.expression, None, check_mode)
            }
            _ => self.any_type, // CallExpression, PrivateFieldExpression
        };
        self.get_or_create_union_type(vec![inner_type, self.undefined_type])
    }

    /// Resolve the type of an assignment target (LHS of `=` or `op=`).
    ///
    /// Dispatches to the appropriate `get_type_of_*` method based on the
    /// target variant, which may emit diagnostics (e.g., TS2339 for
    /// non-existent properties on member expression targets).
    ///
    /// Mirrors tsgo's approach of calling `checkExpressionEx(left)` on the
    /// LHS of assignments before checking assignability.
    pub(crate) fn get_type_of_assignment_target(
        &mut self,
        target: &oxc_ast::ast::AssignmentTarget<'_>,
    ) -> TypeId {
        use oxc_ast::ast::AssignmentTarget;
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.get_type_of_identifier(ident)
            }
            AssignmentTarget::StaticMemberExpression(expr) => {
                // Use declared type for the object — the object of a member
                // assignment target is in write context and should not
                // trigger TS2454 (used before assigned).
                let object_type = self.get_assignment_target_object_type(&expr.object);
                self.resolve_static_member_type(object_type, expr)
            }
            AssignmentTarget::ComputedMemberExpression(expr) => {
                let object_type = self.get_assignment_target_object_type(&expr.object);
                self.resolve_computed_member_type(object_type, expr, CheckMode::NORMAL)
            }
            AssignmentTarget::TSNonNullExpression(expr) => {
                self.get_type_of_expression(&expr.expression, None, CheckMode::NORMAL)
            }
            AssignmentTarget::TSAsExpression(expr) => {
                // The asserted type is what the LHS is declared as
                self.get_type_from_type_node(&expr.type_annotation)
            }
            AssignmentTarget::TSTypeAssertion(expr) => {
                self.get_type_from_type_node(&expr.type_annotation)
            }
            AssignmentTarget::TSSatisfiesExpression(expr) => {
                self.get_type_of_expression(&expr.expression, None, CheckMode::NORMAL)
            }
            // Destructuring patterns — not yet supported
            AssignmentTarget::ArrayAssignmentTarget(_)
            | AssignmentTarget::ObjectAssignmentTarget(_)
            | AssignmentTarget::PrivateFieldExpression(_) => self.any_type,
        }
    }

    /// Get the result type of a binary expression.
    fn get_type_of_binary_expression(
        &mut self,
        expr: &BinaryExpression<'_>,
        check_mode: CheckMode,
    ) -> TypeId {
        match expr.operator {
            // Relational operators: validate operands (checkNonNullType, comparability).
            // Emits TS18050 for null/undefined literals, TS2365 for incompatible types.
            BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => {
                let left_type = self.get_type_of_expression(&expr.left, None, check_mode);
                let right_type = self.get_type_of_expression(&expr.right, None, check_mode);

                // TODO: checkForDisallowedESSymbolOperand

                let left_non_null = self.check_non_null_type(left_type, &expr.left);
                let left_type = self.get_base_type_for_comparison(left_non_null);
                let right_non_null = self.check_non_null_type(right_type, &expr.right);
                let right_type = self.get_base_type_for_comparison(right_non_null);

                self.check_relational_operand_types(
                    left_type,
                    expr.operator,
                    right_type,
                    expr.span,
                );
                self.boolean_type
            }

            // Equality operators: evaluate operands to cache flow-narrowed types
            // for subexpressions (e.g., `x` in `else if (x !== "bar")`) so
            // post-checking queries return the narrowed type.
            // TODO: checkNaNEquality, isTypeEqualityComparableTo, literal object checks
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality => {
                self.get_type_of_expression(&expr.left, None, check_mode);
                self.get_type_of_expression(&expr.right, None, check_mode);
                self.boolean_type
            }

            BinaryOperator::In | BinaryOperator::Instanceof => self.boolean_type,

            // Arithmetic operators (not +): validate operands and return number or bigint.
            // Emits TS2362 (left) / TS2363 (right) for invalid operands,
            // and TS2365 for mixed number/bigint.
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
                let left_type = self.get_type_of_expression(&expr.left, None, check_mode);
                let right_type = self.get_type_of_expression(&expr.right, None, check_mode);

                let left_type = self.check_non_null_type(left_type, &expr.left);
                let right_type = self.check_non_null_type(right_type, &expr.right);

                // If both operands are boolean-like and operator is bitwise,
                // suggest the logical operator instead (TS2447).
                if self.type_arena.get_flags(left_type).intersects(TypeFlags::BooleanLike)
                    && self.type_arena.get_flags(right_type).intersects(TypeFlags::BooleanLike)
                {
                    let suggestion = match expr.operator {
                        BinaryOperator::BitwiseAnd => Some("&&"),
                        BinaryOperator::BitwiseXOR => Some("!=="),
                        BinaryOperator::BitwiseOR => Some("||"),
                        _ => None,
                    };
                    if let Some(suggestion) = suggestion {
                        self.diagnostics.push(
                            OxcDiagnostic::error(format!(
                                "The '{}' operator is not allowed for boolean types. Consider using '{}' instead.",
                                expr.operator.as_str(),
                                suggestion,
                            ))
                            .with_error_code("ts", "2447")
                            .with_label(expr.span),
                        );
                        return self.number_type;
                    }
                }

                // Validate each operand is assignable to number | bigint
                let _left_ok = self.check_arithmetic_operand_type(&expr.left, left_type, true);
                let _right_ok = self.check_arithmetic_operand_type(&expr.right, right_type, false);

                // Determine result type
                if (self.type_arena.get_flags(left_type).intersects(TypeFlags::AnyOrUnknown)
                    && self.type_arena.get_flags(right_type).intersects(TypeFlags::AnyOrUnknown))
                    || (!self.maybe_type_of_kind(left_type, TypeFlags::BigIntLike)
                        && !self.maybe_type_of_kind(right_type, TypeFlags::BigIntLike))
                {
                    self.number_type
                } else if self.maybe_type_of_kind(left_type, TypeFlags::BigIntLike)
                    && self.maybe_type_of_kind(right_type, TypeFlags::BigIntLike)
                {
                    self.bigint_type
                } else {
                    // Mixed number/bigint
                    self.report_operator_error(left_type, expr.operator, right_type, expr.span);
                    self.number_type
                }
            }

            // + is special: string concat if either side is string-like, otherwise number.
            // Emits TS2365 when operands are incompatible.
            // checkNonNullType is only applied when neither side is string-like
            // (string concat with null/undefined is valid — they coerce to "null"/"undefined").
            BinaryOperator::Addition => {
                let left_type = self.get_type_of_expression(&expr.left, None, check_mode);
                let right_type = self.get_type_of_expression(&expr.right, None, check_mode);

                let is_string_concat =
                    self.is_type_assignable_to_kind_ex(left_type, TypeFlags::StringLike, true)
                        || self.is_type_assignable_to_kind_ex(
                            right_type,
                            TypeFlags::StringLike,
                            true,
                        );
                let (left_type, right_type) = if is_string_concat {
                    (left_type, right_type)
                } else {
                    (
                        self.check_non_null_type(left_type, &expr.left),
                        self.check_non_null_type(right_type, &expr.right),
                    )
                };

                if self.is_type_assignable_to_kind_ex(left_type, TypeFlags::NumberLike, true)
                    && self.is_type_assignable_to_kind_ex(right_type, TypeFlags::NumberLike, true)
                {
                    self.number_type
                } else if self.is_type_assignable_to_kind_ex(left_type, TypeFlags::BigIntLike, true)
                    && self.is_type_assignable_to_kind_ex(right_type, TypeFlags::BigIntLike, true)
                {
                    self.bigint_type
                } else if self.is_type_assignable_to_kind_ex(left_type, TypeFlags::StringLike, true)
                    || self.is_type_assignable_to_kind_ex(right_type, TypeFlags::StringLike, true)
                {
                    self.string_type
                } else if self.type_arena.get_flags(left_type).intersects(TypeFlags::Any)
                    || self.type_arena.get_flags(right_type).intersects(TypeFlags::Any)
                {
                    self.any_type
                } else {
                    // No valid combination
                    self.report_operator_error(left_type, expr.operator, right_type, expr.span);
                    self.any_type
                }
            }
        }
    }

    /// Compute the result type of `left_type <op> right_type`.
    ///
    /// Simplified version of binary operator type logic for compound assignments
    /// (e.g., `+=` maps to `Addition`). Does not emit operand validation
    /// diagnostics — those are only relevant for standalone binary expressions.
    pub(crate) fn get_result_type_of_binary_operation(
        &self,
        op: BinaryOperator,
        left_type: TypeId,
        right_type: TypeId,
    ) -> TypeId {
        let left_flags = self.type_arena.get_flags(left_type);
        let right_flags = self.type_arena.get_flags(right_type);
        match op {
            BinaryOperator::Addition => {
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
                if left_flags.intersects(TypeFlags::BigIntLike)
                    && right_flags.intersects(TypeFlags::BigIntLike)
                {
                    self.bigint_type
                } else {
                    self.number_type
                }
            }
            // Comparison/equality/relational — unreachable from compound assignments
            _ => self.boolean_type,
        }
    }

    /// Check an update expression (`++x`, `x++`, `--x`, `x--`).
    ///
    /// Resolves the operand type and returns `number` or `bigint` accordingly.
    /// Emits TS2356 if the operand is not a numeric type (unless in TYPE_ONLY mode).
    fn check_update_expression(
        &mut self,
        update: &oxc_ast::ast::UpdateExpression<'_>,
        check_mode: CheckMode,
    ) -> TypeId {
        use oxc_ast::ast::SimpleAssignmentTarget;
        // Get the operand type from the SimpleAssignmentTarget
        let operand_type = match &update.argument {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.get_type_of_identifier(ident)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(expr) => {
                let object_type = self.get_type_of_expression(&expr.object, None, check_mode);
                self.resolve_computed_member_type(object_type, expr, check_mode)
            }
            SimpleAssignmentTarget::StaticMemberExpression(expr) => {
                let object_type = self.get_type_of_expression(&expr.object, None, check_mode);
                self.resolve_static_member_type(object_type, expr)
            }
            _ => self.any_type,
        };

        let flags = self.type_arena.get_flags(operand_type);

        // any → number (match tsc behavior)
        if flags.intersects(TypeFlags::AnyOrUnknown) {
            return self.number_type;
        }

        // Check the operand is assignable to number | bigint
        if !self.is_type_assignable_to(operand_type, self.number_or_bigint_type) {
            if !check_mode.contains(CheckMode::TYPE_ONLY) {
                self.diagnostics.push(
                    OxcDiagnostic::error(
                        "An arithmetic operand must be of type 'any', 'number', 'bigint' or an enum type.",
                    )
                    .with_error_code("ts", "2356")
                    .with_label(update.span),
                );
            }
            return self.number_type;
        }

        // Return bigint if operand is bigint-like, otherwise number
        if flags.intersects(TypeFlags::BigIntLike) { self.bigint_type } else { self.number_type }
    }

    /// Validate that an operand is assignable to `number | bigint`.
    /// Emits TS2362 (left-hand side) or TS2363 (right-hand side) if not.
    fn check_arithmetic_operand_type(
        &mut self,
        operand: &Expression<'_>,
        ty: TypeId,
        is_left: bool,
    ) -> bool {
        if self.is_type_assignable_to(ty, self.number_or_bigint_type) {
            return true;
        }
        let (code, msg) = if is_left {
            (
                "2362",
                "The left-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type.",
            )
        } else {
            (
                "2363",
                "The right-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type.",
            )
        };
        self.diagnostics
            .push(OxcDiagnostic::error(msg).with_error_code("ts", code).with_label(operand.span()));
        false
    }

    /// Emit TS2365: "Operator '{op}' cannot be applied to types '{left}' and '{right}'."
    fn report_operator_error(
        &mut self,
        left_type: TypeId,
        operator: BinaryOperator,
        right_type: TypeId,
        span: oxc_span::Span,
    ) {
        let left_str = self.type_to_string(left_type);
        let right_str = self.type_to_string(right_type);
        self.diagnostics.push(
            OxcDiagnostic::error(format!(
                "Operator '{}' cannot be applied to types '{}' and '{}'.",
                operator.as_str(),
                left_str,
                right_str,
            ))
            .with_error_code("ts", "2365")
            .with_label(span),
        );
    }

    /// Widen literal types to their base types for relational comparisons.
    /// Mirrors tsgo's `getBaseTypeOfLiteralTypeForComparison`.
    fn get_base_type_for_comparison(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(
            TypeFlags::StringLiteral | TypeFlags::TemplateLiteral | TypeFlags::StringMapping,
        ) {
            self.string_type
        } else if flags.intersects(TypeFlags::NumberLiteral | TypeFlags::Enum) {
            self.number_type
        } else if flags.intersects(TypeFlags::BigIntLiteral) {
            self.bigint_type
        } else if flags.intersects(TypeFlags::BooleanLiteral) {
            self.boolean_type
        } else if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let widened: Vec<TypeId> =
                    u.types.iter().map(|&m| self.get_base_type_for_comparison(m)).collect();
                return self.get_or_create_union_type(widened);
            }
            type_id
        } else {
            type_id
        }
    }

    /// Validate relational operator operand types.
    /// Emits TS2365 if the operands are not comparable.
    /// Mirrors tsgo's relational check in `checkBinaryLikeExpression`.
    fn check_relational_operand_types(
        &mut self,
        left_type: TypeId,
        operator: BinaryOperator,
        right_type: TypeId,
        span: oxc_span::Span,
    ) {
        let left_flags = self.type_arena.get_flags(left_type);
        let right_flags = self.type_arena.get_flags(right_type);

        // If either is any, the comparison is always valid.
        if left_flags.intersects(TypeFlags::Any) || right_flags.intersects(TypeFlags::Any) {
            return;
        }

        let left_numeric = self.is_type_assignable_to(left_type, self.number_or_bigint_type);
        let right_numeric = self.is_type_assignable_to(right_type, self.number_or_bigint_type);

        // Both numeric → OK; both non-numeric → check structural comparability.
        if left_numeric && right_numeric {
            return;
        }
        if !left_numeric && !right_numeric && self.are_types_comparable(left_type, right_type) {
            return;
        }

        self.report_operator_error(left_type, operator, right_type, span);
    }

    /// Get the type of a call expression, checking for TS2349, TS2554, TS2345.
    fn get_type_of_call_expression(
        &mut self,
        call: &oxc_ast::ast::CallExpression<'_>,
        check_mode: CheckMode,
    ) -> TypeId {
        let callee_type = self.get_type_of_expression(&call.callee, None, check_mode);
        let callee_flags = self.type_arena.get_flags(callee_type);

        // any(...) → any
        if callee_flags.intersects(TypeFlags::Any) {
            // Still evaluate argument expressions for side-effect diagnostics
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    self.get_type_of_expression(expr, None, check_mode);
                }
            }
            return self.any_type;
        }

        let callee_type = self.check_non_null_type_with_reporter(
            callee_type,
            &call.callee,
            NullableErrorReporter::CannotInvoke,
        );

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
                    self.get_type_of_expression(expr, None, check_mode);
                }
            }
            return self.any_type;
        }

        // Fast path: single signature (no overload resolution needed).
        if signatures.len() == 1 {
            return self.check_call_against_signature(&signatures[0], call, check_mode);
        }

        // Multi-signature: overload resolution.
        self.resolve_overloaded_call(signatures, call, check_mode)
    }

    /// Check a call expression against a single resolved signature.
    ///
    /// Evaluates arguments with contextual typing from the signature's parameter
    /// types, checks arity (TS2554), handles generics, and checks argument
    /// assignability (TS2345). Returns the effective return type.
    fn check_call_against_signature(
        &mut self,
        sig: &oxc_types::Signature,
        call: &oxc_ast::ast::CallExpression<'_>,
        check_mode: CheckMode,
    ) -> TypeId {
        use oxc_ast::ast::Argument;

        // Extract signature info before any &mut self calls.
        let sig_flags = sig.flags;
        let sig_min_args = sig.min_argument_count as usize;
        let sig_param_count = sig.parameters.len();
        let sig_return_type = sig.return_type;
        let sig_type_params: smallvec::SmallVec<[TypeId; 4]> = sig.type_parameters.clone();

        // Extract parameter TypeIds (SmallVec, stack-allocated for ≤8 params)
        let param_type_ids: smallvec::SmallVec<[TypeId; 8]> =
            sig.parameters.iter().map(|p| p.type_id).collect();

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
        let arg_types: smallvec::SmallVec<[TypeId; 8]> = call
            .arguments
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let param_ctx = self.get_param_type_at(&param_type_ids, i, sig_flags);
                match arg {
                    Argument::SpreadElement(spread) => {
                        self.get_type_of_expression(&spread.argument, None, check_mode)
                    }
                    _ => {
                        let expr = arg.to_expression();
                        self.get_type_of_expression(expr, param_ctx, check_mode)
                    }
                }
            })
            .collect();

        // --- Generic call handling ---
        let explicit_type_args: Option<smallvec::SmallVec<[TypeId; 4]>> =
            call.type_arguments.as_ref().map(|ta| {
                ta.params.iter().map(|t| self.get_type_from_type_node(t)).collect()
            });
        let (effective_param_types, effective_return_type) = self.instantiate_signature_for_call(
            &sig_type_params,
            param_type_ids,
            sig_return_type,
            sig_flags,
            &arg_types,
            &explicit_type_args,
        );

        // --- Argument type checking (TS2345) ---
        for (i, (arg, &arg_type)) in call.arguments.iter().zip(arg_types.iter()).enumerate() {
            let param_type = self.get_param_type_at(&effective_param_types, i, sig_flags);

            if let Some(param_type) = param_type {
                if !self.type_arena.get_flags(param_type).intersects(TypeFlags::Any) {
                    let span = match arg {
                        Argument::SpreadElement(s) => s.span,
                        _ => arg.to_expression().span(),
                    };
                    self.check_type_assignable_to_and_report(
                        arg_type, param_type, span, "2345",
                        |s, t| format!("Argument of type '{s}' is not assignable to parameter of type '{t}'."),
                    );
                }
            }
        }

        effective_return_type
    }

    /// Resolve an overloaded call expression.
    ///
    /// Iterates candidate signatures in declaration order. For each candidate:
    /// 1. Check argument count (arity).
    /// 2. Check type argument arity (if explicit type args provided).
    /// 3. If generic: infer/validate type args, instantiate.
    /// 4. Check each argument type against parameter type (applicability).
    /// First fully matching candidate wins.
    ///
    /// If no candidate matches, emits diagnostics from the best failing candidate.
    fn resolve_overloaded_call(
        &mut self,
        signatures: &[oxc_types::Signature],
        call: &oxc_ast::ast::CallExpression<'_>,
        check_mode: CheckMode,
    ) -> TypeId {
        use oxc_ast::ast::Argument;

        let actual_args = call.arguments.len();

        // Phase 1: Evaluate all argument types once without contextual typing
        // from any specific overload.
        let arg_types: smallvec::SmallVec<[TypeId; 8]> = call
            .arguments
            .iter()
            .map(|arg| match arg {
                Argument::SpreadElement(spread) => {
                    self.get_type_of_expression(&spread.argument, None, check_mode)
                }
                _ => {
                    let expr = arg.to_expression();
                    self.get_type_of_expression(expr, None, check_mode)
                }
            })
            .collect();

        // Extract explicit type arguments once (same across all candidates).
        let explicit_type_args: Option<smallvec::SmallVec<[TypeId; 4]>> =
            call.type_arguments.as_ref().map(|ta| {
                ta.params.iter().map(|t| self.get_type_from_type_node(t)).collect()
            });

        // Phase 2: Try each candidate.
        // Track failures separately: argument arity errors produce better
        // diagnostics (TS2554) than type argument arity errors.
        let mut last_arg_arity_failed: Option<usize> = None;
        let mut last_type_arg_arity_failed: Option<usize> = None;
        let mut last_type_failed: Option<usize> = None;

        for (i, sig) in signatures.iter().enumerate() {
            // 2.1 Arity check
            let sig_min_args = sig.min_argument_count as usize;
            let max_args = if sig.flags.intersects(oxc_types::SignatureFlags::HasRestParameter) {
                usize::MAX
            } else {
                sig.parameters.len()
            };
            if actual_args < sig_min_args || actual_args > max_args {
                last_arg_arity_failed = Some(i);
                continue;
            }

            // 2.2 Type argument arity check
            if let Some(ref ta) = explicit_type_args {
                if ta.len() != sig.type_parameters.len() {
                    last_type_arg_arity_failed = Some(i);
                    continue;
                }
            }

            // Extract parameter info from the arena-borrowed signature before
            // making &mut self calls.
            let sig_flags = sig.flags;
            let sig_return_type = sig.return_type;
            let sig_type_params: smallvec::SmallVec<[TypeId; 4]> = sig.type_parameters.clone();
            let param_type_ids: smallvec::SmallVec<[TypeId; 8]> =
                sig.parameters.iter().map(|p| p.type_id).collect();

            // 2.3 Generic handling + instantiation
            let (effective_params, effective_return) = self.instantiate_signature_for_call(
                &sig_type_params,
                param_type_ids,
                sig_return_type,
                sig_flags,
                &arg_types,
                &explicit_type_args,
            );

            // 2.4 Applicability check
            let mut applicable = true;
            for (j, &arg_type) in arg_types.iter().enumerate() {
                let param_type = self.get_param_type_at(&effective_params, j, sig_flags);
                if let Some(param_type) = param_type {
                    if !self.type_arena.get_flags(param_type).intersects(TypeFlags::Any)
                        && !self.is_type_assignable_to(arg_type, param_type)
                    {
                        applicable = false;
                        break;
                    }
                }
            }

            if !applicable {
                last_type_failed = Some(i);
                continue;
            }

            // 2.5 Match found — return without diagnostics.
            return effective_return;
        }

        // Phase 3: No match — emit diagnostics from the best failing candidate.
        // Priority: argument type error > argument arity error > type arg arity error.
        let error_idx = last_type_failed
            .or(last_arg_arity_failed)
            .or(last_type_arg_arity_failed)
            .unwrap_or(signatures.len() - 1);

        let error_sig = &signatures[error_idx];
        let sig_flags = error_sig.flags;
        let sig_min_args = error_sig.min_argument_count as usize;
        let sig_param_count = error_sig.parameters.len();
        let sig_return_type = error_sig.return_type;
        let sig_type_params: smallvec::SmallVec<[TypeId; 4]> = error_sig.type_parameters.clone();
        let param_type_ids: smallvec::SmallVec<[TypeId; 8]> =
            error_sig.parameters.iter().map(|p| p.type_id).collect();

        // Arity diagnostic (TS2554)
        let max_args = if sig_flags.intersects(oxc_types::SignatureFlags::HasRestParameter) {
            usize::MAX
        } else {
            sig_param_count
        };
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

        // Instantiate error signature if generic, computing both params and return.
        let (effective_params, effective_return) = self.instantiate_signature_for_call(
            &sig_type_params,
            param_type_ids,
            sig_return_type,
            sig_flags,
            &arg_types,
            &explicit_type_args,
        );

        // Argument type diagnostics (TS2345)
        for (j, (arg, &arg_type)) in call.arguments.iter().zip(arg_types.iter()).enumerate() {
            let param_type = self.get_param_type_at(&effective_params, j, sig_flags);
            if let Some(param_type) = param_type {
                if !self.type_arena.get_flags(param_type).intersects(TypeFlags::Any) {
                    let span = match arg {
                        Argument::SpreadElement(s) => s.span,
                        _ => arg.to_expression().span(),
                    };
                    self.check_type_assignable_to_and_report(
                        arg_type, param_type, span, "2345",
                        |s, t| format!("Argument of type '{s}' is not assignable to parameter of type '{t}'."),
                    );
                }
            }
        }

        effective_return
    }

    /// Instantiate a signature's parameter and return types for a call.
    ///
    /// If the signature has type parameters, uses explicit type arguments or
    /// infers them from `arg_types`, then instantiates via `TypeMapper`.
    /// Non-generic signatures are returned unchanged.
    fn instantiate_signature_for_call(
        &mut self,
        sig_type_params: &[TypeId],
        param_type_ids: smallvec::SmallVec<[TypeId; 8]>,
        sig_return_type: TypeId,
        sig_flags: oxc_types::SignatureFlags,
        arg_types: &[TypeId],
        explicit_type_args: &Option<smallvec::SmallVec<[TypeId; 4]>>,
    ) -> (smallvec::SmallVec<[TypeId; 8]>, TypeId) {
        if sig_type_params.is_empty() {
            return (param_type_ids, sig_return_type);
        }

        let type_args = if let Some(ta) = explicit_type_args {
            ta.clone()
        } else {
            let mut infer_ctx = crate::inference::InferenceContext::new(sig_type_params);
            for (i, &arg_type) in arg_types.iter().enumerate() {
                let raw_param = self.get_param_type_at(&param_type_ids, i, sig_flags);
                if let Some(param_type) = raw_param {
                    self.infer_from_types(&mut infer_ctx, arg_type, param_type);
                }
            }
            self.get_inferred_types(&mut infer_ctx)
        };

        if let Some(mapper) =
            oxc_types::TypeMapper::from_type_parameters(sig_type_params, &type_args)
        {
            let inst_params: smallvec::SmallVec<[TypeId; 8]> =
                param_type_ids.iter().map(|&p| self.instantiate_type(p, &mapper)).collect();
            let inst_return = self.instantiate_type(sig_return_type, &mapper);
            (inst_params, inst_return)
        } else {
            (param_type_ids, sig_return_type)
        }
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
    fn get_type_of_new_expression(&mut self, new_expr: &oxc_ast::ast::NewExpression<'_>) -> TypeId {
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
}
