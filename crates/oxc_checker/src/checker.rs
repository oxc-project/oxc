use std::sync::Arc;

use oxc_index::IndexVec;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::AstKind;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ForStatementInit, ForStatementLeft, Function, Program,
    Statement,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Semantic;
use oxc_span::{CompactStr, GetSpan};
use oxc_syntax::node::NodeId;
use oxc_syntax::symbol::SymbolId;
use oxc_types::{
    ObjectFlags, StructuredType, StructuredTypeKind, TypeArena, TypeData, TypeFlags, TypeId,
    TypeParameterType, UnionType,
};
use smallvec::SmallVec;

use oxc_checker_host::{CheckerHost, CheckerOptions};

/// Controls expression checking behavior.
///
/// Mirrors tsgo's `CheckMode`. Flags are bitwise-combinable.
///
///  - `NORMAL` (0): full checking with all diagnostics.
///  - `TYPE_ONLY`: type resolution without certain diagnostics — used by
///    `getTypeOfExpression` (CFA, declared-type inference). Currently
///    suppresses equality-comparison diagnostics that can false-fire
///    on transiently narrowed types during control-flow analysis.
///  - Future flags (`CONTEXTUAL`, `INFERENTIAL`, etc.) will be added as
///    the checker gains inference and overload resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckMode(u8);

impl CheckMode {
    /// Normal type checking — all diagnostics enabled.
    pub const NORMAL: Self = Self(0);
    /// Called from `get_type_of_expression` in non-checking contexts.
    /// Diagnostics may be omitted.
    pub const TYPE_ONLY: Self = Self(1 << 0);

    #[inline]
    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}

/// Per-file checker state that survives across `Checker` lifetimes.
///
/// Contains all type caches (dedup, resolution, assignability) and
/// per-checker types (this_type, empty_object_type, etc.) that accumulate
/// during checking. After checking completes, the `Project` stores this
/// struct so that a new `Checker` can be reconstructed for post-check
/// queries (conformance harness, LSP hover) without losing cached state.
///
/// # Ownership flow
///
/// 1. Created by `CheckerCaches::new()` before the Checker.
/// 2. Passed into `Checker::new_with_caches()` which takes ownership.
/// 3. After checking, extracted via `Checker::into_caches()`.
/// 4. Stored in `Project::checker_caches` for later use.
/// 5. For post-check queries, `Project::with_checker()` takes the caches
///    out, reconstructs a Checker, runs the callback, then puts them back.
pub struct CheckerCaches {
    // -- Dedup caches --
    /// Cache for deduplicating union types. Key is sorted constituent TypeIds.
    pub(crate) union_types: FxHashMap<Arc<SmallVec<[TypeId; 4]>>, TypeId>,
    /// Cache for deduplicating intersection types. Key preserves constituent
    /// order (unlike unions which are sorted), matching tsgo's approach.
    pub(crate) intersection_types: FxHashMap<SmallVec<[TypeId; 4]>, TypeId>,
    /// Cache for deduplicating TypeReference types. Key is (target, type_args).
    /// Ensures `Array<string>` is one TypeId regardless of how many code paths
    /// create it (type annotation, array literal inference, mapper instantiation).
    pub(crate) type_reference_types: FxHashMap<(TypeId, SmallVec<[TypeId; 4]>), TypeId>,
    /// Cache for deduplicating string literal types (regular/non-fresh). Key is the string value.
    pub(crate) string_literal_types: FxHashMap<CompactStr, TypeId>,
    /// Cache for deduplicating number literal types (regular/non-fresh). Key is f64::to_bits().
    pub(crate) number_literal_types: FxHashMap<u64, TypeId>,
    /// Cache for deduplicating bigint literal types (regular/non-fresh).
    pub(crate) bigint_literal_types: FxHashMap<CompactStr, TypeId>,
    /// Maps regular (non-fresh) literal TypeId → fresh literal TypeId.
    ///
    /// Fresh literals are created from source-code literal expressions (e.g., `"foo"`,
    /// `42`, `true`). They widen to their base types for mutable variables (`let`/`var`).
    /// Non-fresh literals (from type annotations, narrowing, etc.) do NOT widen.
    pub(crate) fresh_literal_map: FxHashMap<TypeId, TypeId>,
    /// Reverse map: fresh literal TypeId → regular literal TypeId.
    pub(crate) regular_literal_map: FxHashMap<TypeId, TypeId>,

    // -- Resolution caches --
    /// Cache for assignability relation results. Key is packed
    /// `(source_id << 32) | target_id` as u64. Avoids recomputing
    /// expensive structural comparisons.
    pub(crate) assignability_cache: FxHashMap<u64, bool>,
    /// Cache of resolved intersection types. Maps an intersection TypeId to
    /// a StructuredType TypeId with merged properties from all constituents.
    pub(crate) intersection_resolved_cache: FxHashMap<TypeId, TypeId>,
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
    /// Cache of awaited types. Maps a type to its "awaited" form — the result
    /// of `await`ing it. `Promise<T>` → `T`, non-promises → themselves.
    /// Avoids recomputing recursive Promise unwrapping.
    pub(crate) awaited_type_cache: FxHashMap<TypeId, TypeId>,
    /// Cache for `get_widened_type` results — prevents re-widening and
    /// ensures stable TypeIds for the same widened type.
    pub(crate) widened_type_cache: FxHashMap<TypeId, TypeId>,

    // -- Per-symbol caches --
    /// Cache of resolved symbol types. Each symbol's type is computed at most
    /// once and stored here. Mirrors tsgo's valueSymbolLinks.resolvedType.
    /// Uses IndexVec for O(1) array indexing (standard oxc pattern for per-symbol data).
    pub(crate) symbol_type_cache: IndexVec<SymbolId, Option<TypeId>>,
    /// Per-symbol cache for definite-assignment analysis (TS2454).
    /// `None` = not yet computed.
    /// `Some((false, _))` = symbol is NOT potentially uninitialized.
    /// `Some((true, initial_type))` = symbol IS potentially uninitialized;
    ///   `initial_type` is `declared_type | undefined`.
    /// The scope-walk (outer-variable check) is still done per-reference.
    pub(crate) definite_assignment_cache: IndexVec<SymbolId, Option<(bool, TypeId)>>,
    /// Cache of declared types for type-namespace symbols (type aliases,
    /// interfaces, classes, enums). Mirrors tsgo's getDeclaredTypeOfSymbol
    /// caching. Uses IndexVec for O(1) array indexing.
    pub(crate) declared_type_cache: IndexVec<SymbolId, Option<TypeId>>,

    // -- Type query caches --
    /// Cache of expression types computed during `check_program()`.
    /// Keyed by packed span `(start << 32 | end)` as u64.
    /// Populated at checking call sites where flow graphs and contextual
    /// types are active — mirrors tsgo's `typeNodeLinks.resolvedType`.
    /// Queried by `get_type_at_location()` for post-checking type queries
    /// (conformance harness, LSP, etc.).
    pub(crate) expression_type_cache: FxHashMap<u64, TypeId>,

    // -- Type parameter state --
    /// Lazily resolved type parameter constraints. Keyed by TypeParameter
    /// TypeId, value is the resolved constraint TypeId. Populated on first
    /// access via `get_constraint_of_type_parameter`. Mirrors tsgo's
    /// `TypeParameter.constraint` field (nil until first access).
    pub(crate) type_param_constraints: FxHashMap<TypeId, TypeId>,

    // -- Per-checker types --
    // These are allocated in the arena during construction and compared by
    // TypeId identity. They MUST be preserved across Checker reconstructions.
    /// The implicit `this` type parameter (displays as "this").
    pub(crate) this_type: TypeId,
    /// Empty anonymous object type `{}` — initial accumulator for spread.
    pub(crate) empty_object_type: TypeId,
    /// Cached `number | bigint` union type for arithmetic operand validation.
    pub(crate) number_or_bigint_type: TypeId,
}

impl CheckerCaches {
    /// Create empty caches for a file with `symbols_len` symbols.
    ///
    /// Allocates per-checker types (`this_type`, `empty_object_type`,
    /// `number_or_bigint_type`) in the given `type_arena`. The `intrinsics`
    /// are needed to create the `number | bigint` union type.
    pub fn new(
        symbols_len: usize,
        type_arena: &TypeArena,
        intrinsics: oxc_checker_host::IntrinsicIds,
    ) -> Self {
        let this_type = type_arena.new_type(
            TypeFlags::TypeParameter,
            ObjectFlags::None,
            TypeData::TypeParameter(Box::new(TypeParameterType {
                name: None,
                constraint: None,
                target: None,
                is_this_type: true,
                resolved_default_type: None,
            })),
            None,
        );

        let empty_object_type = type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Anonymous,
            TypeData::Structured(Box::new(StructuredType {
                properties: Vec::new(),
                string_index_type: None,
                number_index_type: None,
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                kind: StructuredTypeKind::Anonymous { target: None },
            })),
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

        Self {
            union_types: FxHashMap::default(),
            intersection_types: FxHashMap::default(),
            type_reference_types: FxHashMap::default(),
            string_literal_types: FxHashMap::default(),
            number_literal_types: FxHashMap::default(),
            bigint_literal_types: FxHashMap::default(),
            fresh_literal_map: FxHashMap::default(),
            regular_literal_map: FxHashMap::default(),
            assignability_cache: FxHashMap::default(),
            intersection_resolved_cache: FxHashMap::default(),
            instantiation_cache: FxHashMap::default(),
            mapped_type_cache: FxHashMap::default(),
            awaited_type_cache: FxHashMap::default(),
            widened_type_cache: FxHashMap::default(),
            symbol_type_cache: IndexVec::from_vec(vec![None; symbols_len]),
            definite_assignment_cache: IndexVec::from_vec(vec![None; symbols_len]),
            declared_type_cache: IndexVec::from_vec(vec![None; symbols_len]),
            expression_type_cache: FxHashMap::default(),
            type_param_constraints: FxHashMap::default(),
            this_type,
            empty_object_type,
            number_or_bigint_type,
        }
    }
}

/// TypeScript type checker.
///
/// The checker runs after semantic analysis and resolves types for all
/// expressions and declarations, emitting diagnostics for type errors.
///
/// Types are stored in a borrowed `TypeArena` and referenced by `TypeId`.
/// Well-known types (primitives) are pre-allocated during construction.
///
/// # Cache ownership
///
/// Mutable checker state (all caches, dedup maps, per-checker types) lives
/// in the `caches: CheckerCaches` field. After checking, call
/// `into_caches()` to extract the caches for storage in the `Project`.
/// To reconstruct a Checker for post-check queries, pass the stored caches
/// back via `new_with_caches()`.
///
/// # Arena ownership
///
/// The checker borrows `&'a TypeArena` — it does not own the arena.
/// This allows multiple checkers (across files) to share a single arena,
/// keeping all TypeIds compatible. The caller (or `Project`) owns the arena.
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

    // -- Resolved compiler options --
    // These are not all used yet — diagnostics gated on them will be added
    // incrementally as the checker gains coverage. Suppress dead_code for now.
    /// `allowUnreachableCode` tristate, passed through from `CheckerOptions`.
    /// `Some(true)` = suppress, `Some(false)` = error, `None` = suggestion.
    pub(crate) allow_unreachable_code: Option<bool>,
    /// `allowUnusedLabels` tristate, passed through from `CheckerOptions`.
    /// `Some(true)` = suppress, `Some(false)` = error, `None` = suggestion.
    #[allow(dead_code)]
    pub(crate) allow_unused_labels: Option<bool>,
    /// Resolved `strictNullChecks` (from option or inherited from `strict`).
    /// Used at init to select widening vs regular null/undefined types;
    /// the field is retained for future use by assignability checks.
    #[allow(dead_code)]
    pub(crate) strict_null_checks: bool,
    /// Resolved `strictPropertyInitialization` (from option or `strict`).
    #[allow(dead_code)]
    pub(crate) strict_property_initialization: bool,
    /// Resolved `strictFunctionTypes` (from option or `strict`).
    #[allow(dead_code)]
    pub(crate) strict_function_types: bool,
    /// Resolved `noImplicitAny` (from option or `strict`).
    #[allow(dead_code)]
    pub(crate) no_implicit_any: bool,
    /// Resolved `noImplicitThis` (from option or `strict`).
    #[allow(dead_code)]
    pub(crate) no_implicit_this: bool,
    /// `noFallthroughCasesInSwitch` option.
    #[allow(dead_code)]
    pub(crate) no_fallthrough_cases_in_switch: bool,
    /// `noImplicitReturns` option.
    #[allow(dead_code)]
    pub(crate) no_implicit_returns: bool,

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

    /// All persistent mutable state — caches, dedup maps, per-checker types.
    /// Extracted via `into_caches()` after checking for storage in `Project`.
    pub(crate) caches: CheckerCaches,

    // -- Session-scoped state (not preserved across Checker lifetimes) --

    /// Set of `(source, target)` pairs currently being checked for
    /// assignability. Used to detect cycles (e.g., TypeParameter
    /// constraints `T extends U, U extends T`) and break infinite
    /// recursion by returning `false`. Mirrors the `resolving_symbols`
    /// pattern for symbol resolution cycle detection.
    pub(crate) in_flight_assignability: FxHashSet<u64>,

    /// Set of symbols currently being resolved, for cycle detection.
    /// If we encounter a symbol already in this set, we have a circular
    /// reference and return `any_type` to break the cycle.
    /// Mirrors tsgo's pushTypeResolution/popTypeResolution.
    pub(crate) resolving_symbols: FxHashSet<SymbolId>,

    /// Buffer for collecting `infer` type parameters during extends clause
    /// processing. Swapped to an empty vec before processing each conditional
    /// type's extends clause, then drained into the `ConditionalType`.
    /// Supports nesting (nested conditionals swap/restore the buffer).
    pub(crate) current_infer_type_params: Vec<TypeId>,

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

    /// Deferred function expression and arrow function bodies to check after
    /// all top-level statements have been processed. Mirrors tsgo's
    /// `deferredNodes` queue.
    ///
    /// Function expression bodies are deferred so that the enclosing scope is
    /// fully resolved before the body is checked — this ensures recursive and
    /// forward references inside the body find cached symbol types.
    ///
    /// Stores `NodeId` values which are looked up from `semantic.nodes()` at
    /// processing time, yielding `&'a Function<'a>` / `&'a ArrowFunctionExpression<'a>`
    /// with the correct lifetime without requiring lifetime annotations on the
    /// methods that queue them.
    deferred_bodies: Vec<NodeId>,
    /// Set for O(1) deduplication when queuing deferred bodies. A function
    /// expression may be encountered multiple times during type resolution
    /// (assignability, CFA, etc.) but should only be body-checked once.
    deferred_bodies_set: FxHashSet<NodeId>,

    // -- Well-known types, pre-allocated during construction. --
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
    /// Widening null type: equals `null_type` in strict mode, carries
    /// `ContainsWideningType` in non-strict mode. Returned by null literal
    /// expressions; `get_widened_type` maps it to `any`.
    pub(crate) null_widening_type: TypeId,
    /// Widening undefined type: equals `undefined_type` in strict mode,
    /// carries `ContainsWideningType` in non-strict mode.
    pub(crate) undefined_widening_type: TypeId,
    pub never_type: TypeId,
    /// The `object` non-primitive type (not `Object` interface).
    pub non_primitive_type: TypeId,
    /// The `true` literal type.
    pub true_type: TypeId,
    /// The `false` literal type.
    pub false_type: TypeId,
    /// Cached global `Array` type for display and array literal creation.
    pub(crate) array_type: TypeId,
    /// Cached global `Promise` type for await unwrapping.
    pub(crate) promise_type: Option<TypeId>,
    /// Cached global `PromiseLike` type for await unwrapping.
    pub(crate) promise_like_type: Option<TypeId>,
    /// Cached global `String` interface (apparent type for `string`/string literals).
    pub(crate) global_string_type: Option<TypeId>,
    /// Cached global `Number` interface (apparent type for `number`/number literals).
    pub(crate) global_number_type: Option<TypeId>,
    /// Cached global `Boolean` interface (apparent type for `boolean`/boolean literals).
    pub(crate) global_boolean_type: Option<TypeId>,
    /// Cached global `BigInt` interface (apparent type for `bigint`/bigint literals).
    pub(crate) global_bigint_type: Option<TypeId>,
    /// Cached global `Symbol` interface (apparent type for `symbol`).
    pub(crate) global_es_symbol_type: Option<TypeId>,
}

impl<'a> Checker<'a> {
    /// Build the session-scoped (non-cache) fields common to both constructors.
    fn build_session(
        semantic: &'a Semantic<'a>,
        type_arena: &'a TypeArena,
        host: &'a dyn CheckerHost,
        file_path: String,
        file_idx: u16,
        options: CheckerOptions,
        caches: CheckerCaches,
    ) -> Self {
        let intrinsics = host.get_intrinsics();
        let base_count = type_arena.len() as u32;

        let array_type = host.get_global_type("Array").unwrap_or(intrinsics.any_type);
        let promise_type = host.get_global_type("Promise");
        let promise_like_type = host.get_global_type("PromiseLike");
        let global_string_type = host.get_global_type("String");
        let global_number_type = host.get_global_type("Number");
        let global_boolean_type = host.get_global_type("Boolean");
        let global_bigint_type = host.get_global_type("BigInt");
        let global_es_symbol_type = host.get_global_type("Symbol");

        let resolve_strict = |opt: Option<bool>| opt.unwrap_or(options.strict);
        let strict_null_checks = resolve_strict(options.strict_null_checks);

        Self {
            semantic,
            file_path,
            host,
            file_idx,
            allow_unreachable_code: options.allow_unreachable_code,
            allow_unused_labels: options.allow_unused_labels,
            strict_null_checks,
            strict_property_initialization: resolve_strict(options.strict_property_initialization),
            strict_function_types: resolve_strict(options.strict_function_types),
            no_implicit_any: resolve_strict(options.no_implicit_any),
            no_implicit_this: resolve_strict(options.no_implicit_this),
            no_fallthrough_cases_in_switch: options.no_fallthrough_cases_in_switch,
            no_implicit_returns: options.no_implicit_returns,
            type_arena,
            base_count,
            diagnostics: Vec::new(),
            recursion_depth: 0,
            caches,
            // Session-scoped state
            in_flight_assignability: FxHashSet::default(),
            resolving_symbols: FxHashSet::default(),
            current_infer_type_params: Vec::new(),
            awaited_type_stack: Vec::new(),
            return_type_stack: Vec::new(),
            current_flow_graph: crate::flow::FlowGraph::empty(),
            flow_type_cache: FxHashMap::default(),
            deferred_bodies: Vec::new(),
            deferred_bodies_set: FxHashSet::default(),
            // Well-known types from intrinsics
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
            null_widening_type: intrinsics.null_widening_type,
            undefined_widening_type: intrinsics.undefined_widening_type,
            never_type: intrinsics.never_type,
            non_primitive_type: intrinsics.non_primitive_type,
            true_type: intrinsics.true_type,
            false_type: intrinsics.false_type,
            array_type,
            promise_type,
            promise_like_type,
            global_string_type,
            global_number_type,
            global_boolean_type,
            global_bigint_type,
            global_es_symbol_type,
        }
    }

    /// Create a new type checker with fresh caches.
    ///
    /// Allocates per-checker types (this_type, empty_object_type,
    /// number_or_bigint_type) in the arena and creates empty caches.
    pub fn new_with_host(
        semantic: &'a Semantic<'a>,
        type_arena: &'a TypeArena,
        host: &'a dyn CheckerHost,
        file_path: String,
        file_idx: u16,
        options: CheckerOptions,
    ) -> Self {
        let intrinsics = host.get_intrinsics();
        let symbols_len = semantic.scoping().symbols_len();
        let caches = CheckerCaches::new(symbols_len, type_arena, intrinsics);
        Self::build_session(semantic, type_arena, host, file_path, file_idx, options, caches)
    }

    /// Create a type checker with previously-stored caches.
    ///
    /// Used by `Project::with_checker()` to reconstruct a Checker for
    /// post-check queries. The caches contain all state from the original
    /// checking pass, so cache-dependent operations (expression type lookup,
    /// type dedup, assignability) produce the same results.
    pub fn new_with_caches(
        semantic: &'a Semantic<'a>,
        type_arena: &'a TypeArena,
        host: &'a dyn CheckerHost,
        file_path: String,
        file_idx: u16,
        options: CheckerOptions,
        caches: CheckerCaches,
    ) -> Self {
        Self::build_session(semantic, type_arena, host, file_path, file_idx, options, caches)
    }

    /// Consume the checker and return its caches for storage.
    ///
    /// The returned `CheckerCaches` contains all accumulated state
    /// (type caches, dedup maps, per-checker types). Pass it to
    /// `new_with_caches()` to reconstruct a Checker later.
    pub fn into_caches(self) -> CheckerCaches {
        self.caches
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
        // Process deferred function expression / arrow bodies.
        // Mirrors tsgo's checkDeferredNodes called after checkSourceElements.
        self.check_deferred_bodies();
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
        if let Some(&cached) = self.caches.expression_type_cache.get(&key) {
            return cached;
        }
        self.get_type_of_expression(expr, None, CheckMode::TYPE_ONLY)
    }

    /// Extract the type parameter constraint cache.
    ///
    /// Used by Project to accumulate constraints from all files into a
    /// global cache, so per-file checkers can look up constraints for
    /// type parameters declared in other files.
    pub fn take_type_param_constraints(&mut self) -> FxHashMap<TypeId, TypeId> {
        std::mem::take(&mut self.caches.type_param_constraints)
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

            Statement::TSInterfaceDeclaration(decl) => {
                self.check_interface_declaration(decl);
            }

            // Leaf statements / not yet implemented — no-op
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::EmptyStatement(_)
            | Statement::TSTypeAliasDeclaration(_)
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
            Declaration::TSInterfaceDeclaration(decl) => {
                self.check_interface_declaration(decl);
            }
            Declaration::TSEnumDeclaration(_) => {}
            Declaration::TSTypeAliasDeclaration(_)
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

    /// Check a function body: resolve the return type annotation, build a
    /// per-function flow graph, and walk the statements.
    ///
    /// Saves and restores the outer flow graph and cache so nested function
    /// bodies don't interfere with the enclosing scope's CFA state.
    fn check_function_body(&mut self, func: &Function<'a>) {
        let return_type =
            func.return_type.as_ref().map(|rt| self.get_type_from_type_node(&rt.type_annotation));
        if let Some(body) = &func.body {
            self.check_body_statements(&body.statements, return_type);
        }
    }

    /// Queue a function expression or arrow function body for deferred checking.
    ///
    /// Called from `get_type_of_expression_inner` when a function expression or
    /// arrow function is encountered. The body will be checked after all
    /// top-level statements are processed (in `check_deferred_bodies`).
    pub(crate) fn queue_deferred_body(&mut self, node_id: NodeId) {
        if self.deferred_bodies_set.insert(node_id) {
            self.deferred_bodies.push(node_id);
        }
    }

    /// Process all deferred function expression and arrow function bodies.
    ///
    /// Called after `check_source_elements` in `check_program`. Uses an index
    /// loop because checking a body may discover nested function expressions
    /// that are appended to the queue during iteration.
    fn check_deferred_bodies(&mut self) {
        let mut i = 0;
        while i < self.deferred_bodies.len() {
            let node_id = self.deferred_bodies[i];
            i += 1;
            let kind = self.semantic.nodes().get_node(node_id).kind();
            match kind {
                AstKind::Function(func) => self.check_function_body(func),
                AstKind::ArrowFunctionExpression(arrow) => {
                    let return_type = arrow
                        .return_type
                        .as_ref()
                        .map(|rt| self.get_type_from_type_node(&rt.type_annotation));
                    self.check_body_statements(&arrow.body.statements, return_type);
                }
                _ => {}
            }
        }
    }

    /// Shared implementation for checking a function/arrow/method body.
    ///
    /// Pushes the return type context, builds a per-scope flow graph, walks
    /// the body statements, then restores the outer flow graph and return
    /// type stack.
    fn check_body_statements(&mut self, stmts: &[Statement<'a>], return_type: Option<TypeId>) {
        self.return_type_stack.push(return_type);
        let flow_graph = crate::flow_builder::FlowGraphBuilder::build(stmts, &self.semantic);
        let prev_graph = std::mem::replace(&mut self.current_flow_graph, flow_graph);
        let prev_cache = std::mem::take(&mut self.flow_type_cache);
        self.check_source_elements(stmts);
        self.current_flow_graph = prev_graph;
        self.flow_type_cache = prev_cache;
        self.return_type_stack.pop();
    }

    /// Check a class declaration's members: method bodies and property initializers.
    fn check_class_declaration(&mut self, class: &oxc_ast::ast::Class<'a>) {
        for element in &class.body.body {
            use oxc_ast::ast::ClassElement;
            match element {
                ClassElement::MethodDefinition(method) => {
                    if let Some(body) = &method.value.body {
                        let return_type = method
                            .value
                            .return_type
                            .as_ref()
                            .map(|rt| self.get_type_from_type_node(&rt.type_annotation));
                        self.check_body_statements(&body.statements, return_type);
                    }
                }
                ClassElement::PropertyDefinition(prop) => {
                    let declared_type = prop
                        .type_annotation
                        .as_ref()
                        .map(|ann| self.get_type_from_type_node(&ann.type_annotation));

                    let init_type =
                        prop.value.as_ref().map(|init| self.check_expression(init, declared_type));

                    if let (Some(declared_type), Some(init_type)) = (declared_type, init_type) {
                        let label_span = prop.key.span();
                        self.check_type_assignable_to_and_report(
                            init_type,
                            declared_type,
                            label_span,
                            "2322",
                            |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
                        );
                    }
                }
                _ => {}
            }
        }
        // TS2416: Check that each property in the derived class is assignable to
        // the same property in the base class.
        self.check_class_heritage_members(class);
        // TODO: check that abstract members are implemented in subclasses
    }

    /// Check that each property in a class is compatible with the same-named
    /// property in its base class. Emits TS2416 on mismatch.
    fn check_class_heritage_members(&mut self, class: &oxc_ast::ast::Class<'a>) {
        // Get the class symbol to look up its declared type (which has base types)
        let Some(ident) = &class.id else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        let class_type = self.get_declared_type_of_symbol(symbol_id);

        // Extract base types and own properties in a single arena read.
        // Both are needed for the check loop which calls &mut self methods.
        let (base_types, own_properties) = {
            let TypeData::Structured(s) = self.type_arena.get_data(class_type) else {
                return;
            };
            let StructuredTypeKind::Interface { resolved_base_types, .. } = &s.kind else {
                return;
            };
            if resolved_base_types.is_empty() {
                return;
            }
            let bases: SmallVec<[TypeId; 4]> = resolved_base_types.clone();
            let props: Vec<(CompactStr, TypeId)> =
                s.properties.iter().map(|p| (p.name.clone(), p.type_id)).collect();
            (bases, props)
        };

        let class_name = &ident.name;

        // For each own property, check assignability against base type properties
        for (prop_name, prop_type) in &own_properties {
            for &base_type in &base_types {
                if let Some(base_prop_type) = self.get_property_of_type(base_type, prop_name) {
                    if base_prop_type != self.any_type
                        && !self.is_type_assignable_to(*prop_type, base_prop_type)
                    {
                        let base_type_str = self.type_to_string(base_type);
                        self.diagnostics.push(
                            OxcDiagnostic::error(format!(
                                "Property '{prop_name}' in type '{class_name}' is not assignable to the same property in base type '{base_type_str}'.",
                            ))
                            .with_error_code("ts", "2416")
                            .with_label(ident.span),
                        );
                    }
                }
            }
        }
    }

    /// Check an interface declaration's heritage for compatibility.
    /// Emits TS2430 when the interface has properties that conflict with base types.
    fn check_interface_declaration(&mut self, decl: &oxc_ast::ast::TSInterfaceDeclaration<'a>) {
        // Resolve the interface type (triggers type building if not cached)
        let Some(symbol_id) = decl.id.symbol_id.get() else { return };
        let iface_type = self.get_declared_type_of_symbol(symbol_id);

        // Extract base types and own properties in a single arena read.
        let (base_types, own_properties) = {
            let TypeData::Structured(s) = self.type_arena.get_data(iface_type) else {
                return;
            };
            let StructuredTypeKind::Interface { resolved_base_types, .. } = &s.kind else {
                return;
            };
            if resolved_base_types.is_empty() {
                return;
            }
            let bases: SmallVec<[TypeId; 4]> = resolved_base_types.clone();
            let props: Vec<(CompactStr, TypeId)> =
                s.properties.iter().map(|p| (p.name.clone(), p.type_id)).collect();
            (bases, props)
        };

        let iface_name = &decl.id.name;

        // For each own property, check assignability against base type properties
        for (prop_name, prop_type) in &own_properties {
            for &base_type in &base_types {
                if let Some(base_prop_type) = self.get_property_of_type(base_type, prop_name) {
                    if base_prop_type != self.any_type
                        && !self.is_type_assignable_to(*prop_type, base_prop_type)
                    {
                        let base_type_str = self.type_to_string(base_type);
                        self.diagnostics.push(
                            OxcDiagnostic::error(format!(
                                "Interface '{iface_name}' incorrectly extends interface '{base_type_str}'.",
                            ))
                            .with_error_code("ts", "2430")
                            .with_label(decl.id.span),
                        );
                        // Only emit one TS2430 per base type mismatch (tsc emits one per base)
                        break;
                    }
                }
            }
        }
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
        self.caches.symbol_type_cache[symbol_id]
    }

    /// Get the cached declared type for a symbol (type-side), if resolved.
    pub fn get_cached_declared_type(
        &self,
        symbol_id: oxc_syntax::symbol::SymbolId,
    ) -> Option<TypeId> {
        self.caches.declared_type_cache[symbol_id]
    }
}
