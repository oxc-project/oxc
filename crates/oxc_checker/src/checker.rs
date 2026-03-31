use std::sync::Arc;

use oxc_index::IndexVec;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::{BindingPattern, Declaration, Function, Program, Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Semantic;
use oxc_span::{CompactStr, GetSpan};
use oxc_syntax::symbol::SymbolId;
use oxc_types::{FunctionType, ObjectFlags, ParameterInfo, Signature, SignatureFlags, TypeArena, TypeData, TypeFlags, TypeId, TypeParameterType, UnionType};
use smallvec::SmallVec;

use crate::host::CheckerHost;

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
    pub(crate) semantic: Semantic<'a>,

    /// Optional host for cross-file resolution (global types, imports).
    /// When `None`, the checker falls back to its local `global_types` HashMap.
    /// When `Some`, global type queries and import resolution go through the host.
    host: Option<&'a dyn CheckerHost>,

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

    /// Global types from lib.d.ts (Array, Promise, etc.).
    /// Used as fallback when no host is provided.
    pub(crate) global_types: FxHashMap<CompactStr, TypeId>,

    /// Reverse mapping from TypeId to name for global types (lib.d.ts).
    /// Used for display since SymbolIds from lib.d.ts are cleared after
    /// bootstrap extraction and are not valid in the user's Semantic.
    pub(crate) global_type_names: FxHashMap<TypeId, CompactStr>,

    /// Stack of return types for enclosing functions.
    /// `Some(type_id)` = function has a declared return type annotation.
    /// `None` = function has no return type annotation (returns are unchecked).
    /// Empty = we're at the top level (return statements are invalid but the
    /// parser handles that).
    pub(crate) return_type_stack: Vec<Option<TypeId>>,
    /// Stack of enclosing class instance types for resolving `this`.
    pub(crate) class_type_stack: Vec<TypeId>,

    /// The flow graph for the currently-checked function/program scope.
    /// Built as a pre-pass before type-checking, consumed by the backward
    /// walk during `get_type_of_identifier`.
    pub(crate) current_flow_graph: crate::flow::FlowGraph,

    /// Cache for resolved types at shared flow nodes. Keyed by
    /// (FlowNodeId, SymbolId) so that nodes used as antecedents by multiple
    /// successors don't re-traverse the same subgraph. Cleared per scope.
    pub(crate) flow_type_cache: FxHashMap<(crate::flow::FlowNodeId, SymbolId), TypeId>,

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
}

impl<'a> Checker<'a> {
    /// Create a new type checker for standalone (single-file) use.
    ///
    /// The caller provides a mutable reference to a `TypeArena`.
    /// Without a host, the checker parses lib.d.ts itself for global types
    /// and cannot resolve cross-file imports.
    pub fn new(semantic: Semantic<'a>, type_arena: &'a TypeArena) -> Self {
        use crate::global_types::allocate_intrinsics;
        let intrinsics = allocate_intrinsics(type_arena);
        let global_types =
            crate::global_types::GlobalTypes::from_lib(type_arena, &intrinsics);
        let base_count = type_arena.len() as u32;
        Self::new_inner(semantic, type_arena, global_types.types, global_types.type_names, None, intrinsics, base_count)
    }

    /// Create a new type checker with a host for cross-file resolution.
    ///
    /// The host provides global types (lib.d.ts) and cross-file import
    /// resolution. When a host is provided, the checker's local
    /// `global_types` HashMap is empty — all global lookups go through
    /// the host.
    ///
    /// The `type_arena` should be the same arena that the host's global
    /// types were allocated into, ensuring all TypeIds are compatible.
    pub fn new_with_host(
        semantic: Semantic<'a>,
        type_arena: &'a TypeArena,
        host: &'a dyn CheckerHost,
    ) -> Self {
        use crate::global_types::allocate_intrinsics;
        let intrinsics = allocate_intrinsics(type_arena);
        let base_count = type_arena.len() as u32;
        Self::new_inner(semantic, type_arena, FxHashMap::default(), FxHashMap::default(), Some(host), intrinsics, base_count)
    }

    pub(crate) fn new_inner(
        semantic: Semantic<'a>,
        type_arena: &'a TypeArena,
        global_types: FxHashMap<CompactStr, TypeId>,
        global_type_names: FxHashMap<TypeId, CompactStr>,
        host: Option<&'a dyn CheckerHost>,
        intrinsics: crate::global_types::IntrinsicIds,
        base_count: u32,
    ) -> Self {
        // Allocate the implicit `this` type parameter
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

        // Pre-compute values that need semantic/global_types before they're moved
        let symbols_len = semantic.scoping().symbols_len();
        let array_type = if let Some(host) = host {
            host.get_global_type("Array").unwrap_or(intrinsics.any_type)
        } else {
            global_types.get("Array").copied().unwrap_or(intrinsics.any_type)
        };

        Self {
            semantic,
            host,
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
            symbol_type_cache: IndexVec::from_vec(vec![None; symbols_len]),
            resolving_symbols: FxHashSet::default(),
            declared_type_cache: IndexVec::from_vec(vec![None; symbols_len]),
            instantiation_cache: FxHashMap::default(),
            mapped_type_cache: FxHashMap::default(),
            type_param_constraints: FxHashMap::default(),
            global_types,
            global_type_names,
            return_type_stack: Vec::new(),
            class_type_stack: Vec::new(),
            current_flow_graph: crate::flow::FlowGraph::empty(),
            flow_type_cache: FxHashMap::default(),
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
        }
    }

    /// Run the type checker on a program.
    ///
    /// Diagnostics are collected internally and can be retrieved via
    /// `take_diagnostics()`. This allows the checker to be reused or
    /// inspected after checking.
    pub fn check_program(&mut self, program: &Program<'a>) {
        // Build flow graph for the program's top-level statements.
        let flow_graph = crate::flow_builder::FlowGraphBuilder::build(
            &program.body,
            &self.semantic,
        );
        self.current_flow_graph = flow_graph;
        self.flow_type_cache.clear();
        self.check_source_elements(&program.body);
        self.current_flow_graph = crate::flow::FlowGraph::empty();
    }

    /// Drain and return all collected diagnostics.
    pub fn take_diagnostics(&mut self) -> Vec<OxcDiagnostic> {
        std::mem::take(&mut self.diagnostics)
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
                self.check_source_element(&if_stmt.consequent);
                if let Some(alt) = &if_stmt.alternate {
                    self.check_source_element(alt);
                }
            }
            Statement::ForStatement(for_stmt) => {
                self.check_source_element(&for_stmt.body);
            }
            Statement::ForInStatement(for_in) => {
                self.check_source_element(&for_in.body);
            }
            Statement::ForOfStatement(for_of) => {
                self.check_source_element(&for_of.body);
            }
            Statement::WhileStatement(while_stmt) => {
                self.check_source_element(&while_stmt.body);
            }
            Statement::DoWhileStatement(do_while) => {
                self.check_source_element(&do_while.body);
            }
            Statement::SwitchStatement(switch_stmt) => {
                for case in &switch_stmt.cases {
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

            // Leaf statements / not yet implemented — no-op
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::EmptyStatement(_)
            | Statement::ThrowStatement(_)
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

    fn check_variable_declarator(
        &mut self,
        decl: &oxc_ast::ast::VariableDeclarator<'a>,
    ) {
        // Always evaluate the initializer type — this triggers diagnostics
        // like TS2339 (property does not exist) even without a type annotation.
        let init_type = decl.init.as_ref().map(|init| self.get_type_of_expression(init));

        let Some(annotation) = &decl.type_annotation else {
            return;
        };
        let Some(init_type) = init_type else {
            return;
        };

        let declared_type = self.get_type_from_type_node(&annotation.type_annotation);

        if !self.is_type_assignable_to(init_type, declared_type) {
            let source_str = self.type_to_string(init_type);
            let target_str = self.type_to_string(declared_type);

            // Get the span of the binding identifier for the error label
            let label_span = if let BindingPattern::BindingIdentifier(id) = &decl.id {
                id.span
            } else {
                decl.id.span()
            };

            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Type '{source_str}' is not assignable to type '{target_str}'."
                ))
                .with_error_code("ts", "2322")
                .with_label(label_span),
            );
        }
    }

    /// Check a function's body with the return type context pushed.
    /// Regular functions create a new `this` context (unlike arrow functions),
    /// so the class type stack is saved and cleared.
    fn check_function_body(&mut self, func: &Function<'a>) {
        let return_type = func
            .return_type
            .as_ref()
            .map(|rt| self.get_type_from_type_node(&rt.type_annotation));
        self.return_type_stack.push(return_type);
        // Regular functions don't inherit `this` from the enclosing class.
        let prev_class_stack = std::mem::take(&mut self.class_type_stack);
        if let Some(body) = &func.body {
            // Build per-function flow graph, saving the outer one for nested functions.
            let flow_graph = crate::flow_builder::FlowGraphBuilder::build(
                &body.statements,
                &self.semantic,
            );
            let prev_graph = std::mem::replace(&mut self.current_flow_graph, flow_graph);
            let prev_cache = std::mem::take(&mut self.flow_type_cache);
            self.check_source_elements(&body.statements);
            self.current_flow_graph = prev_graph;
            self.flow_type_cache = prev_cache;
        }
        self.class_type_stack = prev_class_stack;
        self.return_type_stack.pop();
    }

    /// Check a class declaration's method bodies.
    fn check_class_declaration(&mut self, class: &oxc_ast::ast::Class<'a>) {
        // Resolve the class instance type and push it onto the stack
        // so that `this` in methods resolves correctly.
        if let Some(ident) = &class.id {
            if let Some(symbol_id) = ident.symbol_id.get() {
                let class_type = self.get_declared_type_of_symbol(symbol_id);
                self.class_type_stack.push(class_type);
            }
        }
        for element in &class.body.body {
            use oxc_ast::ast::ClassElement;
            if let ClassElement::MethodDefinition(method) = element {
                if let Some(body) = &method.value.body {
                    let return_type = method.value.return_type
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
        // Pop class type if we pushed one
        if class.id.as_ref().is_some_and(|id| id.symbol_id.get().is_some()) {
            self.class_type_stack.pop();
        }
        // TODO: check property initializer types against annotations
        // TODO: check that abstract members are implemented in subclasses
    }

    /// Check a return statement against the enclosing function's return type.
    fn check_return_statement(&mut self, ret: &oxc_ast::ast::ReturnStatement<'a>) {
        let Some(Some(expected_return_type)) = self.return_type_stack.last().copied() else {
            // No enclosing function or no return type annotation — skip
            return;
        };

        let actual_type = if let Some(arg) = &ret.argument {
            self.get_type_of_expression(arg)
        } else {
            self.undefined_type
        };

        if !self.is_type_assignable_to(actual_type, expected_return_type) {
            let source_str = self.type_to_string(actual_type);
            let target_str = self.type_to_string(expected_return_type);
            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Type '{source_str}' is not assignable to type '{target_str}'."
                ))
                .with_error_code("ts", "2322")
                .with_label(ret.span),
            );
        }
    }

    /// Look up a global type by name (e.g., "Array", "Promise").
    /// Tries the host first (if available), then falls back to the local
    /// `global_types` HashMap. Returns `any_type` if not found anywhere.
    pub fn get_global_type(&self, name: &str) -> TypeId {
        if let Some(host) = self.host {
            if let Some(type_id) = host.get_global_type(name) {
                return type_id;
            }
        }
        self.global_types
            .get(name)
            .copied()
            .unwrap_or(self.any_type)
    }

    /// Get the type arena (for testing/inspection).
    pub fn type_arena(&self) -> &TypeArena {
        self.type_arena
    }

    /// Get the semantic analysis result.
    pub fn semantic(&self) -> &Semantic<'a> {
        &self.semantic
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
                    || self.type_arena.get_object_flags(t)
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
                None
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
                    || self.type_arena.get_object_flags(t)
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
                TypeData::Intersection(oxc_types::IntersectionType {
                    types: key.clone(),
                }),
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
                let widened: Vec<TypeId> = u.types
                    .iter()
                    .map(|&m| self.get_widened_literal_type(m))
                    .collect();
                return self.get_or_create_union_type(widened);
            }
            type_id
        } else {
            type_id
        }
    }

    /// Build a Signature from a function's formal parameters and return type.
    ///
    /// Used for function declarations, function expressions, and arrow functions.
    /// When there is no return type annotation, infers the return type from the body.
    pub fn build_signature_from_function(&mut self, func: &Function<'_>) -> Signature {
        let mut sig = self.build_signature_from_params(&func.params, func.return_type.as_deref());
        // Infer return type from body when there's no annotation.
        if func.return_type.is_none() {
            if let Some(body) = &func.body {
                sig.return_type = self.infer_return_type_from_body(&body.statements);
            }
        }
        sig
    }

    /// Build a Signature from formal parameters and an optional return type annotation.
    pub fn build_signature_from_params(
        &mut self,
        params: &oxc_ast::ast::FormalParameters<'_>,
        return_type_ann: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
    ) -> Signature {
        let mut parameters = Vec::new();
        let mut min_argument_count: u32 = 0;
        let mut has_rest = false;

        for param in &params.items {
            let name = match &param.pattern {
                BindingPattern::BindingIdentifier(id) => CompactStr::new(id.name.as_str()),
                _ => CompactStr::new("_"),
            };
            let type_id = if let Some(ann) = &param.type_annotation {
                self.get_type_from_type_node(&ann.type_annotation)
            } else {
                self.any_type
            };
            let is_optional = param.optional || param.initializer.is_some();
            parameters.push(ParameterInfo {
                name,
                type_id,
                is_optional,
                is_rest: false,
            });
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
            parameters.push(ParameterInfo {
                name,
                type_id,
                is_optional: false,
                is_rest: true,
            });
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
        }
    }

    /// Infer the return type of a function from its body.
    ///
    /// Collects all return expression types, checks end-of-function reachability
    /// via the flow graph, and produces a union type. If the function end is
    /// reachable (implicit return), `void` is included in the union.
    pub(crate) fn infer_return_type_from_body(
        &mut self,
        stmts: &[Statement<'_>],
    ) -> TypeId {
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
    fn collect_return_types(
        &mut self,
        stmts: &[Statement<'_>],
        out: &mut Vec<TypeId>,
    ) {
        for stmt in stmts {
            self.collect_return_types_from_statement(stmt, out);
        }
    }

    fn collect_return_types_from_statement(
        &mut self,
        stmt: &Statement<'_>,
        out: &mut Vec<TypeId>,
    ) {
        match stmt {
            Statement::ReturnStatement(ret) => {
                let return_type = if let Some(arg) = &ret.argument {
                    self.get_type_of_expression(arg)
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

    /// Create a function type from a single signature.
    pub fn create_function_type(&mut self, signature: Signature) -> TypeId {
        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous,
            TypeData::Function(FunctionType {
                signatures: smallvec::smallvec![signature],
            }),
            None,
        )
    }
}

