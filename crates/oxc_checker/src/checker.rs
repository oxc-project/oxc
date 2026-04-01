use std::sync::Arc;

use oxc_index::IndexVec;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ForStatementInit, ForStatementLeft, Function, Program,
    Statement,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Semantic;
use oxc_span::{CompactStr, GetSpan};
use oxc_syntax::symbol::SymbolId;
use oxc_types::{
    FunctionType, ObjectFlags, ParameterInfo, PropertyInfo, Signature, SignatureFlags,
    StructuredType, StructuredTypeKind, TypeArena, TypeData, TypeFlags, TypeId, TypeParameterType,
    UnionType, build_member_map,
};
use smallvec::SmallVec;

use oxc_checker_host::CheckerHost;

/// TypeScript type checker.
///
/// The checker runs after semantic analysis and resolves types for all
/// expressions and declarations, emitting diagnostics for type errors.
///
/// Types are stored in a borrowed `TypeArena` and referenced by `TypeId`.
/// Well-known types (primitives) are pre-allocated during construction.
///
/// # Arena ownership
///
/// The checker borrows `&'a TypeArena` — it does not own the arena.
/// This allows multiple checkers (across files) to share a single arena,
/// keeping all TypeIds compatible. The caller (or `Project`) owns the arena.
///
/// ## Future parallelism note
///
/// For parallel checking, the single `&'a TypeArena` will be replaced
/// with per-thread arenas (each checker gets its own arena for temporary
/// types created during body checking). After all files are checked, arenas
/// are merged. The Checker's public API won't change — it still takes
/// `&mut TypeArena` — but the arena handed to it may be thread-local.
pub struct Checker<'a> {
    /// The semantic analysis result for the program being checked.
    /// Borrowed (not owned) so the Semantic can outlive the Checker —
    /// this is required for multi-file checking where Semantics are kept
    /// alive in the Project for cross-file resolution.
    pub(crate) semantic: &'a Semantic<'a>,

    /// Path of the file being checked. Used to identify this file when
    /// calling `host.resolve_import(from_file, ...)`.
    pub(crate) file_path: String,

    /// Host for global type lookups and cross-file import resolution.
    /// Provided by the Project.
    pub(crate) host: &'a dyn CheckerHost,

    /// Index of the file being checked. Stored on types created by this
    /// checker so cross-file lookups know which Semantic to query.
    pub(crate) file_idx: u16,

    /// Borrowed arena storing all types created during checking.
    /// Shared with global types (from lib.d.ts) and other checkers via
    /// the caller-owned arena.
    pub(crate) type_arena: &'a TypeArena,

    /// Number of types in the arena before checking begins (intrinsics +
    /// global types from lib.d.ts). Types below this index are "base" types
    /// that won't be mutated during checking. This threshold enables a future
    /// two-arena split: base types shared across threads, local types per-thread.
    #[allow(dead_code)]
    pub(crate) base_count: u32,

    /// Diagnostics collected during type checking.
    pub(crate) diagnostics: Vec<OxcDiagnostic>,

    /// Recursion depth counter to prevent stack overflow.
    pub(crate) recursion_depth: u32,

    /// Cache for deduplicating union types. Key is sorted constituent TypeIds.
    union_types: FxHashMap<Arc<SmallVec<[TypeId; 4]>>, TypeId>,

    /// Cache for deduplicating intersection types. Key preserves constituent
    /// order (unlike unions which are sorted), matching tsgo's approach.
    intersection_types: FxHashMap<SmallVec<[TypeId; 4]>, TypeId>,

    /// Cache for deduplicating string literal types. Key is the string value.
    string_literal_types: FxHashMap<CompactStr, TypeId>,

    /// Cache for deduplicating number literal types. Key is f64::to_bits().
    number_literal_types: FxHashMap<u64, TypeId>,

    /// Cache for deduplicating bigint literal types. Key is the bigint string value.
    bigint_literal_types: FxHashMap<CompactStr, TypeId>,

    /// Cache for assignability relation results. Key is packed
    /// `(source_id << 32) | target_id` as u64. Avoids recomputing
    /// expensive structural comparisons.
    pub(crate) assignability_cache: FxHashMap<u64, bool>,

    /// Set of `(source, target)` pairs currently being checked for
    /// assignability. Used to detect cycles (e.g., TypeParameter
    /// constraints `T extends U, U extends T`) and break infinite
    /// recursion by returning `false`. Mirrors the `resolving_symbols`
    /// pattern for symbol resolution cycle detection.
    pub(crate) in_flight_assignability: FxHashSet<u64>,

    /// Cache of resolved intersection types. Maps an intersection TypeId to
    /// a StructuredType TypeId with merged properties from all constituents.
    pub(crate) intersection_resolved_cache: FxHashMap<TypeId, TypeId>,

    /// Cache of resolved symbol types. Each symbol's type is computed at most
    /// once and stored here. Mirrors tsgo's valueSymbolLinks.resolvedType.
    /// Uses IndexVec for O(1) array indexing (standard oxc pattern for per-symbol data).
    pub(crate) symbol_type_cache: IndexVec<SymbolId, Option<TypeId>>,

    /// Set of symbols currently being resolved, for cycle detection.
    /// If we encounter a symbol already in this set, we have a circular
    /// reference and return `any_type` to break the cycle.
    /// Mirrors tsgo's pushTypeResolution/popTypeResolution.
    pub(crate) resolving_symbols: FxHashSet<SymbolId>,

    /// Cache of declared types for type-namespace symbols (type aliases,
    /// interfaces, classes, enums). Mirrors tsgo's getDeclaredTypeOfSymbol
    /// caching.
    /// Uses IndexVec for O(1) array indexing.
    pub(crate) declared_type_cache: IndexVec<SymbolId, Option<TypeId>>,

    /// Cache of resolved TypeReference types. When a TypeReference like
    /// `Array<string>` has its members resolved (properties instantiated),
    /// the result is a new type in the arena with populated member_map.
    /// This cache maps the original TypeReference TypeId to the resolved
    /// type TypeId. Mirrors tsgo's `ObjectType.instantiations` cache and
    /// lazy `resolveStructuredTypeMembers`.
    pub(crate) instantiation_cache: FxHashMap<TypeId, TypeId>,

    /// Cache of resolved mapped type instantiations. Keyed by packed
    /// `(mapped_type_id, constraint_type_id)` as u64 (same packing as
    /// assignability_cache). Avoids recomputing `Partial<Foo>` when the
    /// same mapped type is instantiated with the same constraint twice.
    pub(crate) mapped_type_cache: FxHashMap<u64, TypeId>,

    /// Lazily resolved type parameter constraints. Keyed by TypeParameter
    /// TypeId, value is the resolved constraint TypeId. Populated on first
    /// access via `get_constraint_of_type_parameter`. Mirrors tsgo's
    /// `TypeParameter.constraint` field (nil until first access).
    pub(crate) type_param_constraints: FxHashMap<TypeId, TypeId>,

    /// Buffer for collecting `infer` type parameters during extends clause
    /// processing. Swapped to an empty vec before processing each conditional
    /// type's extends clause, then drained into the `ConditionalType`.
    /// Supports nesting (nested conditionals swap/restore the buffer).
    pub(crate) current_infer_type_params: Vec<TypeId>,

    /// Cache of awaited types. Maps a type to its "awaited" form — the result
    /// of `await`ing it. `Promise<T>` → `T`, non-promises → themselves.
    /// Avoids recomputing recursive Promise unwrapping.
    pub(crate) awaited_type_cache: FxHashMap<TypeId, TypeId>,

    /// Stack of types currently being unwrapped by `get_awaited_type`.
    /// Used to detect circular self-referencing promises. Mirrors tsgo's
    /// `awaitedTypeStack` field.
    pub(crate) awaited_type_stack: Vec<TypeId>,
    /// Stack of return types for enclosing functions.
    /// `Some(type_id)` = function has a declared return type annotation.
    /// `None` = function has no return type annotation (returns are unchecked).
    /// Empty = we're at the top level (return statements are invalid but the
    /// parser handles that).
    pub(crate) return_type_stack: Vec<Option<TypeId>>,

    /// The flow graph for the currently-checked function/program scope.
    /// Built as a pre-pass before type-checking, consumed by the backward
    /// walk during `get_type_of_identifier`.
    pub(crate) current_flow_graph: crate::flow::FlowGraph,

    /// Cache for resolved types at shared flow nodes. Keyed by
    /// (FlowNodeId, SymbolId) so that nodes used as antecedents by multiple
    /// successors don't re-traverse the same subgraph. Cleared per scope.
    pub(crate) flow_type_cache: FxHashMap<(crate::flow::FlowNodeId, SymbolId), TypeId>,

    /// Cache of expression types computed during `check_program()`.
    /// Keyed by packed span `(start << 32 | end)` as u64.
    /// Populated at checking call sites where flow graphs and contextual
    /// types are active — mirrors tsgo's `typeNodeLinks.resolvedType`.
    /// Queried by `get_type_at_location()` for post-checking type queries
    /// (conformance harness, LSP, etc.).
    pub(crate) expression_type_cache: FxHashMap<u64, TypeId>,

    // Well-known types, pre-allocated during construction.
    pub any_type: TypeId,
    pub unknown_type: TypeId,
    pub string_type: TypeId,
    pub number_type: TypeId,
    pub bigint_type: TypeId,
    pub boolean_type: TypeId,
    pub es_symbol_type: TypeId,
    pub void_type: TypeId,
    pub undefined_type: TypeId,
    pub null_type: TypeId,
    pub never_type: TypeId,
    /// The `object` non-primitive type (not `Object` interface).
    pub non_primitive_type: TypeId,
    /// The `true` literal type.
    pub true_type: TypeId,
    /// The `false` literal type.
    pub false_type: TypeId,
    /// The implicit `this` type parameter (displays as "this").
    pub this_type: TypeId,
    /// Cached global `Array` type for display and array literal creation.
    pub(crate) array_type: TypeId,
    /// Cached global `Promise` type for await unwrapping.
    pub(crate) promise_type: Option<TypeId>,
    /// Cached global `PromiseLike` type for await unwrapping.
    pub(crate) promise_like_type: Option<TypeId>,
    /// Empty anonymous object type `{}` — used as the initial accumulator
    /// for spread type folding.
    pub(crate) empty_object_type: TypeId,
    /// Cached `number | bigint` union type for arithmetic operand validation.
    pub(crate) number_or_bigint_type: TypeId,
}

impl<'a> Checker<'a> {
    /// Create a new type checker with a host for cross-file resolution.
    ///
    /// The host provides global types (lib.d.ts) and cross-file import
    /// resolution. All global type lookups go through the host.
    ///
    /// The `type_arena` should be the same arena that the host's global
    /// types were allocated into, ensuring all TypeIds are compatible.
    pub fn new_with_host(
        semantic: &'a Semantic<'a>,
        type_arena: &'a TypeArena,
        host: &'a dyn CheckerHost,
        file_path: String,
        file_idx: u16,
    ) -> Self {
        let intrinsics = host.get_intrinsics();
        let base_count = type_arena.len() as u32;

        let this_type = type_arena.new_type(
            TypeFlags::TypeParameter,
            ObjectFlags::None,
            TypeData::TypeParameter(TypeParameterType {
                name: None,
                constraint: None,
                target: None,
                is_this_type: true,
                resolved_default_type: None,
            }),
            None,
        );

        let empty_object_type = type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous,
            TypeData::Structured(StructuredType {
                member_map: FxHashMap::default(),
                properties: Vec::new(),
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            }),
            None,
        );

        let number_or_bigint_type = {
            let types: SmallVec<[TypeId; 4]> =
                smallvec::smallvec![intrinsics.number_type, intrinsics.bigint_type];
            type_arena.new_type(
                TypeFlags::Union,
                ObjectFlags::None,
                TypeData::Union(UnionType { types: types.into() }),
                None,
            )
        };

        let symbols_len = semantic.scoping().symbols_len();
        let array_type = host.get_global_type("Array").unwrap_or(intrinsics.any_type);
        let promise_type = host.get_global_type("Promise");
        let promise_like_type = host.get_global_type("PromiseLike");

        Self {
            semantic,
            file_path,
            host,
            file_idx,
            type_arena,
            base_count,
            diagnostics: Vec::new(),
            recursion_depth: 0,
            union_types: FxHashMap::default(),
            intersection_types: FxHashMap::default(),
            string_literal_types: FxHashMap::default(),
            number_literal_types: FxHashMap::default(),
            bigint_literal_types: FxHashMap::default(),
            assignability_cache: FxHashMap::default(),
            in_flight_assignability: FxHashSet::default(),
            intersection_resolved_cache: FxHashMap::default(),
            symbol_type_cache: IndexVec::from_vec(vec![None; symbols_len]),
            resolving_symbols: FxHashSet::default(),
            declared_type_cache: IndexVec::from_vec(vec![None; symbols_len]),
            instantiation_cache: FxHashMap::default(),
            mapped_type_cache: FxHashMap::default(),
            type_param_constraints: FxHashMap::default(),
            current_infer_type_params: Vec::new(),
            awaited_type_cache: FxHashMap::default(),
            awaited_type_stack: Vec::new(),
            return_type_stack: Vec::new(),
            current_flow_graph: crate::flow::FlowGraph::empty(),
            flow_type_cache: FxHashMap::default(),
            expression_type_cache: FxHashMap::default(),
            any_type: intrinsics.any_type,
            unknown_type: intrinsics.unknown_type,
            string_type: intrinsics.string_type,
            number_type: intrinsics.number_type,
            bigint_type: intrinsics.bigint_type,
            boolean_type: intrinsics.boolean_type,
            es_symbol_type: intrinsics.es_symbol_type,
            void_type: intrinsics.void_type,
            undefined_type: intrinsics.undefined_type,
            null_type: intrinsics.null_type,
            never_type: intrinsics.never_type,
            non_primitive_type: intrinsics.non_primitive_type,
            true_type: intrinsics.true_type,
            false_type: intrinsics.false_type,
            this_type,
            array_type,
            promise_type,
            promise_like_type,
            empty_object_type,
            number_or_bigint_type,
        }
    }

    /// Run the type checker on a program.
    ///
    /// Diagnostics are collected internally and can be retrieved via
    /// `take_diagnostics()`. This allows the checker to be reused or
    /// inspected after checking.
    pub fn check_program(&mut self, program: &Program<'a>) {
        // Build flow graph for the program's top-level statements.
        let flow_graph =
            crate::flow_builder::FlowGraphBuilder::build(&program.body, &self.semantic);
        self.current_flow_graph = flow_graph;
        self.flow_type_cache.clear();
        self.check_source_elements(&program.body);
        self.current_flow_graph = crate::flow::FlowGraph::empty();
    }

    /// Drain and return all collected diagnostics.
    pub fn take_diagnostics(&mut self) -> Vec<OxcDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Get the type of an expression, using cached results from `check_program()`
    /// when available. Falls back to `get_type_of_expression` for cache misses
    /// (e.g., dead code or expressions in unchecked paths).
    ///
    /// This is the primary API for querying expression types after checking —
    /// used by the conformance harness and (future) LSP.
    /// Mirrors tsgo's `GetTypeAtLocation`.
    pub fn get_type_at_location(&mut self, expr: &Expression<'a>) -> TypeId {
        let span = expr.span();
        let key = (span.start as u64) << 32 | span.end as u64;
        if let Some(&cached) = self.expression_type_cache.get(&key) {
            return cached;
        }
        self.get_type_of_expression(expr, None)
    }

    /// Extract the type parameter constraint cache.
    ///
    /// Used by Project to accumulate constraints from all files into a
    /// global cache, so per-file checkers can look up constraints for
    /// type parameters declared in other files.
    pub fn take_type_param_constraints(&mut self) -> FxHashMap<TypeId, TypeId> {
        std::mem::take(&mut self.type_param_constraints)
    }

    /// Check assignability with excess property checking support.
    ///
    /// Unlike `is_type_assignable_to` which returns a bare bool, this method
    /// returns a `RelationResult` that distinguishes between "not assignable"
    /// and "excess property" failures, allowing call sites to produce the
    /// correct error code (TS2322 vs TS2353).
    pub fn check_type_assignable_to(
        &mut self,
        source: TypeId,
        target: TypeId,
        error_span: oxc_span::Span,
    ) -> crate::relater::RelationResult {
        let relater = crate::relater::Relater::new(error_span);
        relater.check_type_assignable_to(self, source, target)
    }

    /// Check assignability and emit diagnostics for failures.
    ///
    /// Handles both excess property errors (TS2353) and type mismatch errors.
    /// The `not_assignable_code` and `not_assignable_msg` closure are used
    /// only for the `NotAssignable` case — the closure receives `(source_str, target_str)`
    /// and returns the error message. This avoids computing type strings on the
    /// common success path.
    pub(crate) fn check_type_assignable_to_and_report(
        &mut self,
        source: TypeId,
        target: TypeId,
        span: oxc_span::Span,
        not_assignable_code: &'static str,
        not_assignable_msg: impl FnOnce(&str, &str) -> String,
    ) {
        let result = self.check_type_assignable_to(source, target, span);
        match result {
            crate::relater::RelationResult::Assignable => {}
            crate::relater::RelationResult::ExcessProperty { property_name, target_type } => {
                let target_str = self.type_to_string(target_type);
                self.diagnostics.push(
                    OxcDiagnostic::error(format!(
                        "Object literal may only specify known properties, and '{property_name}' does not exist in type '{target_str}'."
                    ))
                    .with_error_code("ts", "2353")
                    .with_label(span),
                );
            }
            crate::relater::RelationResult::NotAssignable => {
                let source_str = self.type_to_string(source);
                let target_str = self.type_to_string(target);
                self.diagnostics.push(
                    OxcDiagnostic::error(not_assignable_msg(&source_str, &target_str))
                        .with_error_code("ts", not_assignable_code)
                        .with_label(span),
                );
            }
        }
    }

    /// Eagerly resolve all type parameter constraints in the arena.
    ///
    /// Used after checking lib.d.ts to ensure all constraints are cached
    /// before the per-file Checker (and its Semantic) are dropped. Without
    /// this, constraints like `K extends keyof any` in `Record<K, T>`
    /// would be unresolvable by per-file checkers.
    pub fn eagerly_resolve_type_param_constraints(&mut self) {
        for idx in 0..self.type_arena.len() {
            let type_id = TypeId::from_usize(idx);
            if self.type_arena.get_flags(type_id).contains(TypeFlags::TypeParameter) {
                self.get_constraint_of_type_parameter(type_id);
            }
        }
    }

    /// Check a single AST node, dispatching by kind.
    /// Equivalent to tsgo's `checkSourceElementWorker`.
    fn check_source_element(&mut self, stmt: &Statement<'a>) {
        match stmt {
            // Declarations
            Statement::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    self.check_variable_declarator(declarator);
                }
            }
            Statement::FunctionDeclaration(func) => {
                self.check_function_body(func);
            }

            // Compound statements — recurse into children
            Statement::BlockStatement(block) => {
                self.check_source_elements(&block.body);
            }
            Statement::IfStatement(if_stmt) => {
                self.check_expression(&if_stmt.test, None);
                self.check_source_element(&if_stmt.consequent);
                if let Some(alt) = &if_stmt.alternate {
                    self.check_source_element(alt);
                }
            }
            Statement::ForStatement(for_stmt) => {
                if let Some(init) = &for_stmt.init {
                    match init {
                        ForStatementInit::VariableDeclaration(decl) => {
                            for declarator in &decl.declarations {
                                self.check_variable_declarator(declarator);
                            }
                        }
                        init => {
                            if let Some(expr) = init.as_expression() {
                                self.check_expression(expr, None);
                            }
                        }
                    }
                }
                if let Some(test) = &for_stmt.test {
                    self.check_expression(test, None);
                }
                if let Some(update) = &for_stmt.update {
                    self.check_expression(update, None);
                }
                self.check_source_element(&for_stmt.body);
            }
            Statement::ForInStatement(for_in) => {
                self.check_for_in_statement(for_in);
            }
            Statement::ForOfStatement(for_of) => {
                self.check_expression(&for_of.right, None);
                self.check_source_element(&for_of.body);
            }
            Statement::WhileStatement(while_stmt) => {
                self.check_expression(&while_stmt.test, None);
                self.check_source_element(&while_stmt.body);
            }
            Statement::DoWhileStatement(do_while) => {
                self.check_source_element(&do_while.body);
                self.check_expression(&do_while.test, None);
            }
            Statement::SwitchStatement(switch_stmt) => {
                self.check_expression(&switch_stmt.discriminant, None);
                for case in &switch_stmt.cases {
                    if let Some(test) = &case.test {
                        self.check_expression(test, None);
                    }
                    self.check_source_elements(&case.consequent);
                }
            }
            Statement::TryStatement(try_stmt) => {
                self.check_source_elements(&try_stmt.block.body);
                if let Some(handler) = &try_stmt.handler {
                    self.check_source_elements(&handler.body.body);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    self.check_source_elements(&finalizer.body);
                }
            }
            Statement::LabeledStatement(labeled) => {
                self.check_source_element(&labeled.body);
            }
            Statement::WithStatement(with_stmt) => {
                self.check_source_element(&with_stmt.body);
            }

            // Export declarations — recurse into inner declaration
            Statement::ExportNamedDeclaration(export) => {
                if let Some(decl) = &export.declaration {
                    self.check_exported_declaration(decl);
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                use oxc_ast::ast::ExportDefaultDeclarationKind;
                match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        self.check_function_body(func);
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        self.check_class_declaration(class);
                    }
                    _ => {}
                }
            }

            Statement::ExpressionStatement(expr_stmt) => {
                self.check_expression_statement(expr_stmt);
            }

            Statement::ReturnStatement(ret) => {
                self.check_return_statement(ret);
            }

            Statement::ClassDeclaration(class) => {
                self.check_class_declaration(class);
            }
            Statement::TSEnumDeclaration(_) => {
                // Enum declarations are checked via their declared type resolution.
                // TODO: check for duplicate member names, check initializer types
            }

            Statement::ThrowStatement(throw_stmt) => {
                self.check_expression(&throw_stmt.argument, None);
            }

            // Leaf statements / not yet implemented — no-op
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::EmptyStatement(_)
            | Statement::TSTypeAliasDeclaration(_)
            | Statement::TSInterfaceDeclaration(_)
            | Statement::TSModuleDeclaration(_)
            | Statement::TSGlobalDeclaration(_)
            | Statement::TSImportEqualsDeclaration(_)
            | Statement::ImportDeclaration(_)
            | Statement::ExportAllDeclaration(_)
            | Statement::TSExportAssignment(_)
            | Statement::TSNamespaceExportDeclaration(_) => {}
        }
    }

    /// Check a list of statements. Equivalent to tsgo's `checkSourceElements`.
    fn check_source_elements(&mut self, stmts: &[Statement<'a>]) {
        for stmt in stmts {
            self.check_source_element(stmt);
        }
    }

    /// Handle a declaration inside an export statement.
    fn check_exported_declaration(&mut self, decl: &Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    self.check_variable_declarator(declarator);
                }
            }
            Declaration::FunctionDeclaration(func) => {
                self.check_function_body(func);
            }
            Declaration::ClassDeclaration(class) => {
                self.check_class_declaration(class);
            }
            Declaration::TSEnumDeclaration(_) => {}
            Declaration::TSTypeAliasDeclaration(_)
            | Declaration::TSInterfaceDeclaration(_)
            | Declaration::TSModuleDeclaration(_)
            | Declaration::TSGlobalDeclaration(_)
            | Declaration::TSImportEqualsDeclaration(_) => {}
        }
    }

    fn check_variable_declarator(&mut self, decl: &oxc_ast::ast::VariableDeclarator<'a>) {
        // Compute declared type first so it can be used as contextual type
        // for the initializer (enables callback parameter inference, tuple context, etc.)
        let declared_type = decl
            .type_annotation
            .as_ref()
            .map(|ann| self.get_type_from_type_node(&ann.type_annotation));

        // Evaluate the initializer with the declared type as context.
        // This triggers diagnostics (e.g., TS2339, TS2695) even without a type annotation.
        let init_type = decl.init.as_ref().map(|init| self.check_expression(init, declared_type));

        let Some(declared_type) = declared_type else {
            return;
        };
        let Some(init_type) = init_type else {
            return;
        };

        // Get the span of the binding identifier for the error label
        let label_span = if let BindingPattern::BindingIdentifier(id) = &decl.id {
            id.span
        } else {
            decl.id.span()
        };

        self.check_type_assignable_to_and_report(
            init_type,
            declared_type,
            label_span,
            "2322",
            |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
        );
    }

    /// Check a function's body with the return type context pushed.
    fn check_function_body(&mut self, func: &Function<'a>) {
        let return_type =
            func.return_type.as_ref().map(|rt| self.get_type_from_type_node(&rt.type_annotation));
        self.return_type_stack.push(return_type);
        if let Some(body) = &func.body {
            // Build per-function flow graph, saving the outer one for nested functions.
            let flow_graph =
                crate::flow_builder::FlowGraphBuilder::build(&body.statements, &self.semantic);
            let prev_graph = std::mem::replace(&mut self.current_flow_graph, flow_graph);
            let prev_cache = std::mem::take(&mut self.flow_type_cache);
            self.check_source_elements(&body.statements);
            self.current_flow_graph = prev_graph;
            self.flow_type_cache = prev_cache;
        }
        self.return_type_stack.pop();
    }

    /// Check a class declaration's method bodies.
    fn check_class_declaration(&mut self, class: &oxc_ast::ast::Class<'a>) {
        for element in &class.body.body {
            use oxc_ast::ast::ClassElement;
            if let ClassElement::MethodDefinition(method) = element {
                if let Some(body) = &method.value.body {
                    let return_type = method
                        .value
                        .return_type
                        .as_ref()
                        .map(|rt| self.get_type_from_type_node(&rt.type_annotation));
                    self.return_type_stack.push(return_type);
                    let flow_graph = crate::flow_builder::FlowGraphBuilder::build(
                        &body.statements,
                        &self.semantic,
                    );
                    let prev_graph = std::mem::replace(&mut self.current_flow_graph, flow_graph);
                    let prev_cache = std::mem::take(&mut self.flow_type_cache);
                    self.check_source_elements(&body.statements);
                    self.current_flow_graph = prev_graph;
                    self.flow_type_cache = prev_cache;
                    self.return_type_stack.pop();
                }
            }
        }
        // TODO: check property initializer types against annotations
        // TODO: check that abstract members are implemented in subclasses
    }

    /// Check a return statement against the enclosing function's return type.
    ///
    /// Always evaluates the return expression (to catch errors like bad property
    /// accesses even in functions without return type annotations).
    /// Only checks assignability when there is a declared return type.
    fn check_return_statement(&mut self, ret: &oxc_ast::ast::ReturnStatement<'a>) {
        let expected_return_type = self.return_type_stack.last().copied().flatten();

        let Some(arg) = &ret.argument else {
            return;
        };

        let actual_type = self.check_expression(arg, expected_return_type);

        if let Some(expected) = expected_return_type {
            self.check_type_assignable_to_and_report(
                actual_type,
                expected,
                ret.span,
                "2322",
                |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
            );
        }
    }

    /// Check a for-in statement.
    ///
    /// Validates that:
    /// - The RHS expression is of type `any`, an object type, or a type parameter
    /// - The LHS variable (when not a declaration) is assignable from `string`
    fn check_for_in_statement(&mut self, for_in: &oxc_ast::ast::ForInStatement<'a>) {
        let right_type = self.check_expression(&for_in.right, None);
        let right_flags = self.type_arena.get_flags(right_type);

        // RHS must be any, an object type, or a type parameter.
        // Skip check for `any` and `unknown` (permissive) and `never` (already an error elsewhere).
        if !right_flags.intersects(
            TypeFlags::AnyOrUnknown
                | TypeFlags::Never
                | TypeFlags::NonPrimitive
                | TypeFlags::Object
                | TypeFlags::InstantiableNonPrimitive,
        ) {
            // For unions, check if every constituent is valid.
            let is_valid = if right_flags.intersects(TypeFlags::Union) {
                if let TypeData::Union(u) = self.type_arena.get_data(right_type) {
                    let types: SmallVec<[TypeId; 4]> = u.types.iter().copied().collect();
                    types.iter().all(|&t| {
                        let f = self.type_arena.get_flags(t);
                        f.intersects(
                            TypeFlags::AnyOrUnknown
                                | TypeFlags::NonPrimitive
                                | TypeFlags::Object
                                | TypeFlags::InstantiableNonPrimitive,
                        )
                    })
                } else {
                    false
                }
            } else {
                false
            };

            if !is_valid {
                let type_str = self.type_to_string(right_type);
                self.diagnostics.push(
                    OxcDiagnostic::error(format!(
                        "The right-hand side of a 'for...in' statement must be of type 'any', an object type or a type parameter, but here has type '{type_str}'."
                    ))
                    .with_error_code("ts", "2407")
                    .with_label(for_in.right.span()),
                );
            }
        }

        // LHS validation: when the LHS is an existing variable (not a declaration),
        // its type must be assignable from `string`.
        match &for_in.left {
            ForStatementLeft::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    self.check_variable_declarator(declarator);
                }
            }
            ForStatementLeft::AssignmentTargetIdentifier(ident) => {
                let left_type = self.get_type_of_identifier(ident);
                if !self.is_type_assignable_to(self.string_type, left_type) {
                    self.diagnostics.push(
                        OxcDiagnostic::error(
                            "The left-hand side of a 'for...in' statement must be of type 'string' or 'any'."
                        )
                        .with_error_code("ts", "2405")
                        .with_label(ident.span()),
                    );
                }
            }
            _ => {
                // Member expressions, destructuring patterns, etc. — check expression
                // but skip detailed validation for now.
            }
        }

        self.check_source_element(&for_in.body);
    }

    /// Look up a global type by name (type-side, e.g., "Array", "Promise").
    /// Returns `any_type` if not found.
    pub fn get_global_type(&self, name: &str) -> TypeId {
        self.host.get_global_type(name).unwrap_or(self.any_type)
    }

    /// Look up a global value type by name (value-side, e.g., "RegExp" → RegExpConstructor).
    /// Returns `None` if not found.
    pub fn get_global_value_type(&self, name: &str) -> Option<TypeId> {
        self.host.get_global_value_type(name)
    }

    /// Get the type arena (for testing/inspection).
    pub fn type_arena(&self) -> &TypeArena {
        self.type_arena
    }

    /// Get the semantic analysis result.
    pub fn semantic(&self) -> &Semantic<'a> {
        self.semantic
    }

    /// Get the cached type for a symbol (value-side), if resolved.
    pub fn get_cached_symbol_type(
        &self,
        symbol_id: oxc_syntax::symbol::SymbolId,
    ) -> Option<TypeId> {
        self.symbol_type_cache[symbol_id]
    }

    /// Get the cached declared type for a symbol (type-side), if resolved.
    pub fn get_cached_declared_type(
        &self,
        symbol_id: oxc_syntax::symbol::SymbolId,
    ) -> Option<TypeId> {
        self.declared_type_cache[symbol_id]
    }

    /// Get or create a deduplicated union type from a list of constituent type IDs.
    ///
    /// Handles normalization: filters `never`, deduplicates, sorts.
    /// Returns `never` for empty, unwraps single-element unions.
    pub fn get_or_create_union_type(&mut self, mut types: Vec<TypeId>) -> TypeId {
        // Filter out `never` types
        types.retain(|&t| !self.type_arena.get_flags(t).intersects(TypeFlags::Never));

        if types.is_empty() {
            return self.never_type;
        }
        if types.len() == 1 {
            return types[0];
        }

        types.sort();
        types.dedup();

        if types.len() == 1 {
            return types[0];
        }

        let key: Arc<SmallVec<[TypeId; 4]>> = SmallVec::from_vec(types).into();

        let type_id = self.union_types.entry(key.clone()).or_insert_with_key(|key| {
            // Propagate CouldContainTypeVariables so is_generic_type
            // can check a single flag instead of walking constituents.
            let has_instantiable = key.iter().any(|&t| {
                let f = self.type_arena.get_flags(t);
                f.intersects(TypeFlags::Instantiable)
                    || self
                        .type_arena
                        .get_object_flags(t)
                        .intersects(ObjectFlags::CouldContainTypeVariables)
            });
            let obj_flags = if has_instantiable {
                ObjectFlags::CouldContainTypeVariables
            } else {
                ObjectFlags::None
            };
            self.type_arena.new_type(
                TypeFlags::Union,
                obj_flags,
                TypeData::Union(UnionType { types: key.clone() }),
                None,
            )
        });

        *type_id
    }

    /// Create a deduplicated intersection type from a list of constituent type IDs.
    ///
    /// Handles normalization: deduplicates while preserving constituent order
    /// (unlike unions which are sorted), matching tsgo's approach.
    /// Returns `unknown` for empty, unwraps single-element intersections.
    pub fn get_or_create_intersection_type(&mut self, mut types: Vec<TypeId>) -> TypeId {
        // 1. Flatten nested intersections: (A & B) & C → [A, B, C]
        let mut i = 0;
        while i < types.len() {
            if self.type_arena.get_flags(types[i]).intersects(TypeFlags::Intersection) {
                if let TypeData::Intersection(inter) = self.type_arena.get_data(types[i]) {
                    let children: SmallVec<[TypeId; 4]> = inter.types.clone();
                    types.remove(i);
                    for (j, child) in children.into_iter().enumerate() {
                        types.insert(i + j, child);
                    }
                    continue; // re-check at same index
                }
            }
            i += 1;
        }

        // 2. Never propagation: A & never → never
        if types.iter().any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::Never)) {
            return self.never_type;
        }

        // 3. Contradictory primitive reduction: string & number → never
        // Collect which disjoint primitive groups are present.
        let mut groups = 0u8;
        for &t in &types {
            let f = self.type_arena.get_flags(t);
            if f.intersects(TypeFlags::StringLike) {
                groups |= 1;
            }
            if f.intersects(TypeFlags::NumberLike) {
                groups |= 2;
            }
            if f.intersects(TypeFlags::BigIntLike) {
                groups |= 4;
            }
            if f.intersects(TypeFlags::BooleanLike) {
                groups |= 8;
            }
            if f.intersects(TypeFlags::ESSymbolLike) {
                groups |= 16;
            }
            if f.intersects(TypeFlags::Void) {
                groups |= 32;
            }
            if f.intersects(TypeFlags::Undefined) {
                groups |= 64;
            }
            if f.intersects(TypeFlags::Null) {
                groups |= 128;
            }
        }
        if groups.count_ones() > 1 {
            return self.never_type;
        }

        // 4. Supertype removal: string & "hello" → "hello"
        {
            let has_string_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::StringLiteral));
            let has_number_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::NumberLiteral));
            let has_boolean_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::BooleanLiteral));
            let has_bigint_literal = types
                .iter()
                .any(|&t| self.type_arena.get_flags(t).intersects(TypeFlags::BigIntLiteral));
            types.retain(|&t| {
                let f = self.type_arena.get_flags(t);
                !(has_string_literal && f.intersects(TypeFlags::String))
                    && !(has_number_literal && f.intersects(TypeFlags::Number))
                    && !(has_boolean_literal && f.intersects(TypeFlags::Boolean))
                    && !(has_bigint_literal && f.intersects(TypeFlags::BigInt))
            });
        }

        // Order-preserving dedup: retain first occurrence of each type.
        let mut seen = FxHashSet::default();
        types.retain(|t| seen.insert(*t));

        if types.is_empty() {
            return self.unknown_type;
        }
        if types.len() == 1 {
            return types[0];
        }

        let key: SmallVec<[TypeId; 4]> = SmallVec::from_vec(types);

        let type_id = self.intersection_types.entry(key).or_insert_with_key(|key| {
            let has_instantiable = key.iter().any(|&t| {
                let f = self.type_arena.get_flags(t);
                f.intersects(TypeFlags::Instantiable)
                    || self
                        .type_arena
                        .get_object_flags(t)
                        .intersects(ObjectFlags::CouldContainTypeVariables)
            });
            let obj_flags = if has_instantiable {
                ObjectFlags::CouldContainTypeVariables
            } else {
                ObjectFlags::None
            };
            self.type_arena.new_type(
                TypeFlags::Intersection,
                obj_flags,
                TypeData::Intersection(oxc_types::IntersectionType { types: key.clone() }),
                None,
            )
        });
        *type_id
    }

    /// Get or create a deduplicated string literal type.
    pub fn get_or_create_string_literal_type(&mut self, value: &str) -> TypeId {
        let key = CompactStr::new(value);
        let type_id = self.string_literal_types.entry(key).or_insert_with_key(|key| {
            self.type_arena.new_type(
                TypeFlags::StringLiteral,
                ObjectFlags::None,
                TypeData::Literal(oxc_types::LiteralType::String(key.clone())),
                None,
            )
        });
        *type_id
    }

    /// Get or create a deduplicated number literal type.
    pub fn get_or_create_number_literal_type(&mut self, value: f64) -> TypeId {
        let key = value.to_bits();
        let type_id = self.number_literal_types.entry(key).or_insert_with(|| {
            self.type_arena.new_type(
                TypeFlags::NumberLiteral,
                ObjectFlags::None,
                TypeData::Literal(oxc_types::LiteralType::Number(value)),
                None,
            )
        });
        *type_id
    }

    /// Get or create a deduplicated bigint literal type.
    pub fn get_or_create_bigint_literal_type(&mut self, value: &str) -> TypeId {
        let key = CompactStr::new(value);
        let type_id = self.bigint_literal_types.entry(key).or_insert_with_key(|key| {
            self.type_arena.new_type(
                TypeFlags::BigIntLiteral,
                ObjectFlags::None,
                TypeData::Literal(oxc_types::LiteralType::BigInt(key.clone())),
                None,
            )
        });
        *type_id
    }

    /// Widen a literal type to its base type.
    ///
    /// `1` → `number`, `"hello"` → `string`, `true` → `boolean`, `1n` → `bigint`.
    /// Unions have each constituent widened. Non-literal types are returned unchanged.
    pub fn get_widened_literal_type(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);
        if flags.intersects(TypeFlags::StringLiteral) {
            self.string_type
        } else if flags.intersects(TypeFlags::NumberLiteral) {
            self.number_type
        } else if flags.intersects(TypeFlags::BigIntLiteral) {
            self.bigint_type
        } else if flags.intersects(TypeFlags::BooleanLiteral) {
            self.boolean_type
        } else if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(type_id) {
                let widened: Vec<TypeId> =
                    u.types.iter().map(|&m| self.get_widened_literal_type(m)).collect();
                return self.get_or_create_union_type(widened);
            }
            type_id
        } else {
            type_id
        }
    }

    // ── Spread type validation ─────────────────────────────────────────

    /// Check whether a type is valid as the argument of an object spread (`{ ...x }`).
    ///
    /// Valid spread types are: `any`, `object`, object types, instantiable
    /// non-primitive types (type parameters, conditionals, substitutions),
    /// and unions/intersections where every constituent is itself a valid spread type.
    ///
    /// Mirrors TypeScript's `isValidSpreadType`.
    pub(crate) fn is_valid_spread_type(&mut self, type_id: TypeId) -> bool {
        let resolved = self.get_base_constraint_or_type(type_id);
        let filtered = self.remove_definitely_falsy_types(resolved);
        let flags = self.type_arena.get_flags(filtered);

        if flags.intersects(
            TypeFlags::Any
                | TypeFlags::NonPrimitive
                | TypeFlags::Object
                | TypeFlags::InstantiableNonPrimitive,
        ) {
            return true;
        }

        if flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(u) = self.type_arena.get_data(filtered) {
                let members = u.types.clone();
                return members.iter().all(|&t| self.is_valid_spread_type(t));
            }
        }
        if flags.intersects(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = self.type_arena.get_data(filtered) {
                let members = i.types.clone();
                return members.iter().all(|&t| self.is_valid_spread_type(t));
            }
        }

        // Never is valid (it's a bottom type — spreading it produces nothing)
        flags.intersects(TypeFlags::Never)
    }

    /// If `type_id` is an instantiable type (e.g. type parameter), resolve its
    /// base constraint; otherwise return the type unchanged.
    ///
    /// Mirrors TypeScript's `getBaseConstraintOrType`.
    fn get_base_constraint_or_type(&mut self, type_id: TypeId) -> TypeId {
        let flags = self.type_arena.get_flags(type_id);

        if flags.intersects(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(type_id) {
                return constraint;
            }
        }
        // TODO: handle other instantiable types (conditional, substitution,
        // indexed access, index, template literal, string mapping)
        // and unions/intersections of such types.
        type_id
    }

    /// Filter out definitely-falsy constituents (null, undefined, void,
    /// false literal, 0 literal, "" literal) from a type.
    ///
    /// Mirrors TypeScript's `removeDefinitelyFalsyTypes`.
    fn remove_definitely_falsy_types(&mut self, type_id: TypeId) -> TypeId {
        self.narrow_type_by_predicate(type_id, |checker, t| !checker.is_falsy_type(t))
    }

    // ── Spread type merging ───────────────────────────────────────────

    /// Merge two types in a spread operation (left-fold).
    ///
    /// Called repeatedly as `spread = get_spread_type(spread, next_element)`
    /// while iterating through an object literal's properties and spreads.
    ///
    /// Mirrors TypeScript's `getSpreadType`.
    pub(crate) fn get_spread_type(&mut self, left: TypeId, right: TypeId) -> TypeId {
        let left_flags = self.type_arena.get_flags(left);
        let right_flags = self.type_arena.get_flags(right);

        // Any absorbs everything
        if left_flags.intersects(TypeFlags::Any) || right_flags.intersects(TypeFlags::Any) {
            return self.any_type;
        }
        // Unknown absorbs everything
        if left_flags.intersects(TypeFlags::Unknown) || right_flags.intersects(TypeFlags::Unknown) {
            return self.unknown_type;
        }
        // Never is identity
        if left_flags.intersects(TypeFlags::Never) {
            return right;
        }
        if right_flags.intersects(TypeFlags::Never) {
            return left;
        }

        // TODO (Tier 3): Union distribution
        // if left is union → mapType(left, |t| get_spread_type(t, right))
        // if right is union → mapType(right, |t| get_spread_type(left, t))

        // Primitive on right → return left unchanged (spreading a primitive is a no-op)
        if right_flags.intersects(
            TypeFlags::BooleanLike
                | TypeFlags::NumberLike
                | TypeFlags::BigIntLike
                | TypeFlags::StringLike
                | TypeFlags::EnumLike
                | TypeFlags::NonPrimitive
                | TypeFlags::Index,
        ) {
            return left;
        }

        // TODO (Tier 4): Generic object types → create intersection
        // if is_generic_object_type(left) || is_generic_object_type(right) {
        //     return get_intersection_type([left, right])
        // }

        // Concrete object merge
        self.merge_spread_types(left, right)
    }

    /// Merge two concrete object types for a spread operation.
    ///
    /// Right properties override left properties. When both sides have the
    /// same property and the right's is optional, a union of both types is
    /// created with the optionality of the left property preserved.
    fn merge_spread_types(&mut self, left: TypeId, right: TypeId) -> TypeId {
        // Phase 1: Read — extract property data from the arena as Copy/cheap types.
        // Arena references are stable (AppendOnlyVec), but we still need to release
        // the borrows before calling &mut self methods like get_or_create_union_type.
        // We extract only the data we need: names (CompactStr is cheap to clone),
        // TypeIds (Copy), and flags (Copy).

        let right_entries: Vec<(CompactStr, TypeId, bool, bool)> =
            if let TypeData::Structured(s) = self.type_arena.get_data(right) {
                s.properties
                    .iter()
                    .map(|p| (p.name.clone(), p.type_id, p.optional, p.readonly))
                    .collect()
            } else {
                Vec::new()
            };

        let left_entries: Vec<(CompactStr, TypeId, bool, bool)> =
            if let TypeData::Structured(s) = self.type_arena.get_data(left) {
                s.properties
                    .iter()
                    .map(|p| (p.name.clone(), p.type_id, p.optional, p.readonly))
                    .collect()
            } else {
                Vec::new()
            };

        let (l_str_idx, l_num_idx) = if let TypeData::Structured(s) = self.type_arena.get_data(left)
        {
            (s.string_index_type, s.number_index_type)
        } else {
            (None, None)
        };
        let (r_str_idx, r_num_idx) =
            if let TypeData::Structured(s) = self.type_arena.get_data(right) {
                (s.string_index_type, s.number_index_type)
            } else {
                (None, None)
            };

        // Phase 2: Merge — build result properties, calling &mut self as needed.

        // Start with right properties, tracking positions for O(1) overlap updates
        let mut result_props: Vec<PropertyInfo> = right_entries
            .iter()
            .map(|(name, type_id, optional, readonly)| {
                let mut p = PropertyInfo::new(name.clone(), *type_id);
                p.optional = *optional;
                p.readonly = *readonly;
                p
            })
            .collect();
        let right_index: FxHashMap<&CompactStr, usize> =
            right_entries.iter().enumerate().map(|(i, (name, ..))| (name, i)).collect();

        for (name, left_type, left_optional, left_readonly) in &left_entries {
            if let Some(&idx) = right_index.get(name) {
                // Both sides have this property
                if result_props[idx].optional {
                    // Right is optional: union the types, keep left's optionality
                    let union_type =
                        self.get_or_create_union_type(vec![*left_type, result_props[idx].type_id]);
                    result_props[idx].type_id = union_type;
                    result_props[idx].optional = *left_optional;
                }
                // If right is required, it wins — already in result_props
            } else {
                let mut p = PropertyInfo::new(name.clone(), *left_type);
                p.optional = *left_optional;
                p.readonly = *left_readonly;
                result_props.push(p);
            }
        }

        // Merge index signatures
        let string_index = if left == self.empty_object_type {
            r_str_idx
        } else {
            match (l_str_idx, r_str_idx) {
                (Some(l), Some(r)) => Some(self.get_or_create_union_type(vec![l, r])),
                (some, None) | (None, some) => some,
            }
        };
        let number_index = if left == self.empty_object_type {
            r_num_idx
        } else {
            match (l_num_idx, r_num_idx) {
                (Some(l), Some(r)) => Some(self.get_or_create_union_type(vec![l, r])),
                (some, None) | (None, some) => some,
            }
        };

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous
                | ObjectFlags::ObjectLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral
                | ObjectFlags::ContainsSpread,
            TypeData::Structured(StructuredType {
                member_map: build_member_map(&result_props),
                properties: result_props,
                string_index_type: string_index,
                number_index_type: number_index,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            }),
            None,
        )
    }

    /// Build a Signature from a function's formal parameters and return type.
    ///
    /// Used for function declarations, function expressions, and arrow functions.
    /// When there is no return type annotation, infers the return type from the body.
    pub fn build_signature_from_function(&mut self, func: &Function<'_>) -> Signature {
        self.build_signature_from_function_with_context(func, None)
    }

    /// Build a Signature from formal parameters and an optional return type annotation.
    pub fn build_signature_from_params(
        &mut self,
        params: &oxc_ast::ast::FormalParameters<'_>,
        return_type_ann: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
    ) -> Signature {
        self.build_signature_from_params_with_context(params, return_type_ann, None)
    }

    /// Build a Signature from formal parameters with contextual type information.
    ///
    /// When a contextual signature is provided (e.g., from a variable declaration
    /// annotation or a call site parameter type), parameters without type annotations
    /// use the corresponding type from the contextual signature.
    pub fn build_signature_from_params_with_context(
        &mut self,
        params: &oxc_ast::ast::FormalParameters<'_>,
        return_type_ann: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
        contextual_sig: Option<&Signature>,
    ) -> Signature {
        let mut parameters = Vec::new();
        let mut min_argument_count: u32 = 0;
        let mut has_rest = false;

        for (i, param) in params.items.iter().enumerate() {
            let name = match &param.pattern {
                BindingPattern::BindingIdentifier(id) => CompactStr::new(id.name.as_str()),
                _ => CompactStr::new("_"),
            };
            let type_id = if let Some(ann) = &param.type_annotation {
                // Explicit annotation takes priority
                self.get_type_from_type_node(&ann.type_annotation)
            } else if let Some(ctx_sig) = contextual_sig {
                // Fall back to contextual parameter type
                ctx_sig.parameters.get(i).map(|p| p.type_id).unwrap_or(self.any_type)
            } else {
                self.any_type
            };
            let is_optional = param.optional || param.initializer.is_some();
            parameters.push(ParameterInfo { name, type_id, is_optional, is_rest: false });
            if !is_optional {
                min_argument_count += 1;
            }
        }

        // Handle rest parameter
        if let Some(rest) = &params.rest {
            let name = match &rest.rest.argument {
                BindingPattern::BindingIdentifier(id) => CompactStr::new(id.name.as_str()),
                _ => CompactStr::new("_"),
            };
            let type_id = if let Some(ann) = &rest.type_annotation {
                self.get_type_from_type_node(&ann.type_annotation)
            } else {
                self.any_type
            };
            parameters.push(ParameterInfo { name, type_id, is_optional: false, is_rest: true });
            has_rest = true;
        }

        let return_type = if let Some(rt) = return_type_ann {
            self.get_type_from_type_node(&rt.type_annotation)
        } else {
            self.any_type
        };

        let mut flags = SignatureFlags::None;
        if has_rest {
            flags |= SignatureFlags::HasRestParameter;
        }

        Signature {
            flags,
            min_argument_count,
            parameters,
            return_type,
            type_parameters: SmallVec::new(),
        }
    }

    /// Build a Signature from a function declaration/expression with contextual typing.
    ///
    /// Like `build_signature_from_function`, but passes through a contextual signature
    /// for parameter type inference.
    pub fn build_signature_from_function_with_context(
        &mut self,
        func: &Function<'_>,
        contextual_sig: Option<&Signature>,
    ) -> Signature {
        // Create type parameters FIRST so that parameter/return type annotations
        // that reference them (e.g., `T` in `function id<T>(x: T): T`) resolve
        // to the correct TypeParameter TypeIds via the declared_type_cache.
        let type_parameters =
            self.get_type_parameters_from_declaration(func.type_parameters.as_deref());
        let mut sig = self.build_signature_from_params_with_context(
            &func.params,
            func.return_type.as_deref(),
            contextual_sig,
        );
        sig.type_parameters = type_parameters;
        // Infer return type from body when there's no annotation.
        if func.return_type.is_none() {
            if let Some(body) = &func.body {
                sig.return_type = self.infer_return_type_from_body(&body.statements);
            }
        }
        sig
    }

    /// Infer the return type of a function from its body.
    ///
    /// Collects all return expression types, checks end-of-function reachability
    /// via the flow graph, and produces a union type. If the function end is
    /// reachable (implicit return), `void` is included in the union.
    pub(crate) fn infer_return_type_from_body(&mut self, stmts: &[Statement<'_>]) -> TypeId {
        // Collect types from all return statements (non-recursive into nested functions).
        let mut return_types = Vec::new();
        self.collect_return_types(stmts, &mut return_types);

        // Build a flow graph to check end-of-function reachability.
        let flow_graph = crate::flow_builder::FlowGraphBuilder::build(stmts, &self.semantic);
        let end_reachable = flow_graph.is_end_reachable();

        // If the function end is reachable without a return, add void.
        if end_reachable {
            return_types.push(self.void_type);
        }

        if return_types.is_empty() {
            return self.void_type;
        }

        self.get_or_create_union_type(return_types)
    }

    /// Walk statements collecting return expression types.
    /// Does NOT descend into nested function/arrow bodies.
    fn collect_return_types(&mut self, stmts: &[Statement<'_>], out: &mut Vec<TypeId>) {
        for stmt in stmts {
            self.collect_return_types_from_statement(stmt, out);
        }
    }

    fn collect_return_types_from_statement(&mut self, stmt: &Statement<'_>, out: &mut Vec<TypeId>) {
        match stmt {
            Statement::ReturnStatement(ret) => {
                let return_type = if let Some(arg) = &ret.argument {
                    self.get_type_of_expression(arg, None)
                } else {
                    self.undefined_type
                };
                if !out.contains(&return_type) {
                    out.push(return_type);
                }
            }
            // Recurse into compound statements but NOT into nested functions.
            Statement::BlockStatement(block) => {
                self.collect_return_types(&block.body, out);
            }
            Statement::IfStatement(if_stmt) => {
                self.collect_return_types_from_statement(&if_stmt.consequent, out);
                if let Some(alt) = &if_stmt.alternate {
                    self.collect_return_types_from_statement(alt, out);
                }
            }
            Statement::ForStatement(for_stmt) => {
                self.collect_return_types_from_statement(&for_stmt.body, out);
            }
            Statement::ForInStatement(for_in) => {
                self.collect_return_types_from_statement(&for_in.body, out);
            }
            Statement::ForOfStatement(for_of) => {
                self.collect_return_types_from_statement(&for_of.body, out);
            }
            Statement::WhileStatement(while_stmt) => {
                self.collect_return_types_from_statement(&while_stmt.body, out);
            }
            Statement::DoWhileStatement(do_while) => {
                self.collect_return_types_from_statement(&do_while.body, out);
            }
            Statement::SwitchStatement(switch_stmt) => {
                for case in &switch_stmt.cases {
                    self.collect_return_types(&case.consequent, out);
                }
            }
            Statement::TryStatement(try_stmt) => {
                self.collect_return_types(&try_stmt.block.body, out);
                if let Some(handler) = &try_stmt.handler {
                    self.collect_return_types(&handler.body.body, out);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    self.collect_return_types(&finalizer.body, out);
                }
            }
            Statement::LabeledStatement(labeled) => {
                self.collect_return_types_from_statement(&labeled.body, out);
            }
            Statement::WithStatement(with_stmt) => {
                self.collect_return_types_from_statement(&with_stmt.body, out);
            }
            // Function/class declarations — do NOT descend (separate scope).
            Statement::FunctionDeclaration(_)
            | Statement::ClassDeclaration(_)
            | Statement::ExportNamedDeclaration(_)
            | Statement::ExportDefaultDeclaration(_) => {}
            // All other statements — no return statements possible.
            _ => {}
        }
    }

    /// Check if a type could contain type variables (type parameters or
    /// composite types that transitively contain them).
    ///
    /// This is the single-level check used at type creation time to propagate
    /// `CouldContainTypeVariables`. It works because the flag is set
    /// transitively: if a child has it, the parent gets it too.
    pub(crate) fn type_could_contain_type_variables(&self, type_id: TypeId) -> bool {
        let flags = self.type_arena.get_flags(type_id);
        flags.intersects(TypeFlags::Instantiable)
            || self
                .type_arena
                .get_object_flags(type_id)
                .intersects(ObjectFlags::CouldContainTypeVariables)
    }

    /// Check if a signature could contain type variables in its parameter
    /// types or return type.
    pub(crate) fn signature_could_contain_type_variables(&self, sig: &Signature) -> bool {
        sig.parameters.iter().any(|p| self.type_could_contain_type_variables(p.type_id))
            || self.type_could_contain_type_variables(sig.return_type)
    }

    /// Create a function type from a single signature.
    pub fn create_function_type(&mut self, signature: Signature) -> TypeId {
        let mut obj_flags = ObjectFlags::Anonymous;
        if self.signature_could_contain_type_variables(&signature) {
            obj_flags |= ObjectFlags::CouldContainTypeVariables;
        }
        self.type_arena.new_type(
            TypeFlags::Object,
            obj_flags,
            TypeData::Function(FunctionType { signatures: smallvec::smallvec![signature] }),
            None,
        )
    }

    /// Create a constructor type from a single construct signature.
    pub fn create_constructor_type(&mut self, signature: Signature) -> TypeId {
        let mut obj_flags = ObjectFlags::Anonymous;
        if self.signature_could_contain_type_variables(&signature) {
            obj_flags |= ObjectFlags::CouldContainTypeVariables;
        }
        self.type_arena.new_type(
            TypeFlags::Object,
            obj_flags,
            TypeData::Structured(StructuredType {
                properties: Vec::new(),
                member_map: FxHashMap::default(),
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: vec![signature],
                kind: StructuredTypeKind::Anonymous { target: None },
            }),
            None,
        )
    }
}
