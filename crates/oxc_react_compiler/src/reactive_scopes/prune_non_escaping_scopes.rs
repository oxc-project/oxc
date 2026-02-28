/// Prune non-escaping reactive scopes.
///
/// Port of `ReactiveScopes/PruneNonEscapingScopes.ts` from the React Compiler.
///
/// Removes reactive scopes whose output values don't "escape" the function.
/// A value escapes if it's returned, passed to a hook, used in JSX, etc.
/// If a scope only produces local values that don't escape, memoizing them
/// provides no benefit because React can't observe the difference.
///
/// Algorithm (3 phases, matching TS):
///   Phase 1: Walk the reactive function tree to build a dependency graph.
///            - Track `definitions` (LoadLocal indirections: lvalue → source)
///            - Classify each value's `MemoizationLevel` (Memoized/Conditional/Never)
///            - Build dependency edges between DeclarationIds
///            - Collect `escapingValues` (return values, hook args)
///   Phase 2: DFS from escaping values through the dependency graph.
///            - `Memoized` values are always marked if reachable
///            - `Conditional` values are marked only if a dependency is memoized
///            - When a value is memoized, force-memoize its scope's dependencies
///   Phase 3: Prune scopes whose declarations aren't in the memoized set.
///
/// NOTE: Uses DeclarationId throughout, as noted in TS: "this pass uses DeclarationId
/// rather than IdentifierId because the pass is not aware of control-flow, only data
/// flow via mutation."
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    DeclarationId, InstructionValue, Place, ReactiveBlock, ReactiveFunction, ReactiveStatement,
    ReactiveTerminal, ReactiveValue, ScopeId, object_shape::ShapeRegistry,
};

// =====================================================================================
// MemoizationLevel
// =====================================================================================

/// Classification of a value's memoization needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MemoizationLevel {
    /// The value will never be memoized (primitives, cheaply comparable values).
    Never = 0,
    /// Only memoize if forced (JSX without memoizeJsxElements).
    Unmemoized = 1,
    /// Memoize only if dependencies are memoized (LoadLocal, StoreLocal, conditionals).
    Conditional = 2,
    /// Always memoize if reachable from escaping values (CallExpression, ArrayExpression, etc.).
    Memoized = 3,
}

fn join_levels(a: MemoizationLevel, b: MemoizationLevel) -> MemoizationLevel {
    // Take the higher level
    if a >= b { a } else { b }
}

// =====================================================================================
// Graph nodes
// =====================================================================================

#[derive(Debug)]
struct IdentifierNode {
    level: MemoizationLevel,
    memoized: bool,
    dependencies: FxHashSet<DeclarationId>,
    scopes: FxHashSet<ScopeId>,
    seen: bool,
}

#[derive(Debug)]
struct ScopeNode {
    dependencies: Vec<DeclarationId>,
    seen: bool,
}

// =====================================================================================
// State — built during Phase 1
// =====================================================================================

struct State {
    /// Maps lvalue DeclarationId → source DeclarationId for LoadLocal indirections.
    definitions: FxHashMap<DeclarationId, DeclarationId>,
    /// Per-identifier graph nodes.
    identifiers: FxHashMap<DeclarationId, IdentifierNode>,
    /// Per-scope graph nodes.
    scopes: FxHashMap<ScopeId, ScopeNode>,
    /// DeclarationIds that escape (returned, passed to hooks).
    escaping_values: FxHashSet<DeclarationId>,
}

impl State {
    fn new() -> Self {
        Self {
            definitions: FxHashMap::default(),
            identifiers: FxHashMap::default(),
            scopes: FxHashMap::default(),
            escaping_values: FxHashSet::default(),
        }
    }

    fn resolve(&self, id: DeclarationId) -> DeclarationId {
        self.definitions.get(&id).copied().unwrap_or(id)
    }

    fn ensure_identifier(&mut self, id: DeclarationId) -> &mut IdentifierNode {
        self.identifiers.entry(id).or_insert_with(|| IdentifierNode {
            level: MemoizationLevel::Never,
            memoized: false,
            dependencies: FxHashSet::default(),
            scopes: FxHashSet::default(),
            seen: false,
        })
    }
}

// =====================================================================================
// Phase 1: Collect — build the dependency graph
// =====================================================================================

/// Options controlling memoization behavior during pruning.
pub struct PruneOptions<'a> {
    /// When true, JSX elements are treated as `Memoized` (always memoized when
    /// reachable from escaping values). When false, they are `Unmemoized` (only
    /// memoized if explicitly forced). Corresponds to `!enableForest` in the TS.
    pub memoize_jsx_elements: bool,

    /// When true, primitive-producing instructions (Primitive, BinaryExpression,
    /// UnaryExpression, TemplateLiteral, LoadGlobal, etc.) are treated as
    /// `Conditional` instead of `Never`. This ensures scopes that transitively
    /// depend on primitives are preserved when manual memoization validation is
    /// enabled. Corresponds to `forceMemoizePrimitives` in the TS, which is set
    /// to `enableForest || enablePreserveExistingMemoizationGuarantees`.
    pub force_memoize_primitives: bool,

    /// Reference to the shape registry for looking up function signatures
    /// (e.g., to check `noAlias` on call expressions).
    pub shapes: &'a ShapeRegistry,

    /// Reference to the environment for type-based hook detection.
    pub env: &'a crate::hir::environment::Environment,
}

/// Prune reactive scopes whose values don't escape the function.
pub fn prune_non_escaping_scopes(func: &mut ReactiveFunction, opts: &PruneOptions<'_>) {
    let mut state = State::new();

    // Pre-declare parameters so they exist in the dependency graph.
    // Matches TS: `for (const param of fn.params) { state.declare(param...) }`
    for param in &func.params {
        let decl_id = match param {
            crate::hir::ReactiveParam::Place(p) => p.identifier.declaration_id,
            crate::hir::ReactiveParam::Spread(s) => s.place.identifier.declaration_id,
        };
        state.ensure_identifier(decl_id);
    }

    // Phase 1: Walk all instructions/terminals to build the dependency graph
    collect_in_block(&func.body, &mut state, &[], opts);

    // Phase 2: Compute memoized set via DFS from escaping values
    let memoized = compute_memoized_identifiers(&mut state);

    // Phase 3: Prune scopes whose declarations aren't in the memoized set
    let mut pruned_scopes = FxHashSet::default();
    let mut reassignments = FxHashMap::default();
    prune_in_block(&mut func.body, &memoized, &mut pruned_scopes, &mut reassignments);
}

/// Determine the MemoizationLevel for an instruction value.
fn classify_value(value: &InstructionValue, opts: &PruneOptions<'_>) -> MemoizationLevel {
    match value {
        // Allocating values that should always be memoized
        InstructionValue::CallExpression(_)
        | InstructionValue::MethodCall(_)
        | InstructionValue::ArrayExpression(_)
        | InstructionValue::ObjectExpression(_)
        | InstructionValue::NewExpression(_)
        | InstructionValue::FunctionExpression(_)
        | InstructionValue::ObjectMethod(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::TaggedTemplateExpression(_)
        | InstructionValue::PropertyStore(_) => MemoizationLevel::Memoized,

        // ComputedStore: outer lvalue is Conditional (TS lines 711-726).
        // The object acts as an lvalue and only value.value is an rvalue.
        InstructionValue::ComputedStore(_) => MemoizationLevel::Conditional,

        // StoreContext: the instruction's outer lvalue is Conditional,
        // but the inner target (context variable) is Memoized (handled in collect_in_block).
        // This matches TS where StoreContext's outer lvalue is Conditional.
        InstructionValue::StoreContext(_) => MemoizationLevel::Conditional,

        // DeclareContext: the instruction's outer lvalue is Unmemoized,
        // but the inner target (context variable) is Memoized (handled in collect_in_block).
        // This matches TS where DeclareContext's outer lvalue is Unmemoized.
        InstructionValue::DeclareContext(_) => MemoizationLevel::Unmemoized,

        // Values that propagate memoization from their dependencies
        InstructionValue::LoadLocal(_)
        | InstructionValue::LoadContext(_)
        | InstructionValue::StoreLocal(_)
        | InstructionValue::PropertyLoad(_)
        | InstructionValue::ComputedLoad(_)
        | InstructionValue::Destructure(_)
        | InstructionValue::Await(_)
        | InstructionValue::TypeCastExpression(_)
        | InstructionValue::PrefixUpdate(_)
        | InstructionValue::PostfixUpdate(_)
        | InstructionValue::GetIterator(_)
        | InstructionValue::IteratorNext(_) => MemoizationLevel::Conditional,

        // JSX — memoize when `memoizeJsxElements` is true (the default when `enableForest` is false)
        InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_) => {
            if opts.memoize_jsx_elements {
                MemoizationLevel::Memoized
            } else {
                MemoizationLevel::Unmemoized
            }
        }

        // Primitives and other cheap-to-compare values — never memoize by default.
        // When `force_memoize_primitives` is true (enablePreserveExistingMemoizationGuarantees
        // or enableForest), these become `Conditional` so that scopes transitively
        // reachable from them are preserved for manual memoization validation.
        InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::BinaryExpression(_)
        | InstructionValue::UnaryExpression(_)
        | InstructionValue::TemplateLiteral(_)
        | InstructionValue::LoadGlobal(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::ComputedDelete(_)
        | InstructionValue::PropertyDelete(_)
        | InstructionValue::StartMemoize(_)
        | InstructionValue::FinishMemoize(_)
        | InstructionValue::NextPropertyOf(_) => {
            if opts.force_memoize_primitives {
                MemoizationLevel::Conditional
            } else {
                MemoizationLevel::Never
            }
        }

        // DeclareLocal: outer lvalue is Unmemoized (TS line 627),
        // inner target is Unmemoized (handled in collect_in_block).
        InstructionValue::DeclareLocal(_) => MemoizationLevel::Unmemoized,

        // StoreGlobal: outer lvalue is Unmemoized (TS line 678).
        InstructionValue::StoreGlobal(_) => MemoizationLevel::Unmemoized,

        // UnsupportedNode: Never
        InstructionValue::UnsupportedNode(_) => MemoizationLevel::Never,
    }
}

/// Collect operands (rvalues) from a ReactiveValue.
fn collect_operands(
    value: &ReactiveValue,
    operands: &mut Vec<DeclarationId>,
    shapes: &ShapeRegistry,
    force_memoize_primitives: bool,
) {
    match value {
        ReactiveValue::Instruction(iv) => {
            collect_instruction_operands(iv, operands, shapes, force_memoize_primitives);
        }
        ReactiveValue::Logical(v) => {
            collect_operands(&v.left, operands, shapes, force_memoize_primitives);
            collect_operands(&v.right, operands, shapes, force_memoize_primitives);
        }
        ReactiveValue::Ternary(v) => {
            // Conditionals do not alias their test value (matching TS behavior).
            // Only consequent and alternate are rvalues.
            collect_operands(&v.consequent, operands, shapes, force_memoize_primitives);
            collect_operands(&v.alternate, operands, shapes, force_memoize_primitives);
        }
        ReactiveValue::Sequence(v) => {
            for instr in &v.instructions {
                if let Some(lvalue) = &instr.lvalue {
                    operands.push(lvalue.identifier.declaration_id);
                }
                collect_operands(&instr.value, operands, shapes, force_memoize_primitives);
            }
            collect_operands(&v.value, operands, shapes, force_memoize_primitives);
        }
        ReactiveValue::OptionalCall(v) => {
            collect_operands(&v.value, operands, shapes, force_memoize_primitives);
        }
    }
}

/// Collect operands (rvalues) from a single InstructionValue.
///
/// Port of TS `computeMemoizationInputs` rvalues logic:
/// - For `CallExpression`, `MethodCall`, `TaggedTemplateExpression`: checks the `noAlias`
///   flag and returns empty if true.
/// - For primitive-producing instructions (BinaryExpression, UnaryExpression, Primitive,
///   LoadGlobal, etc.): returns empty rvalues when `force_memoize_primitives = false`.
///   This is critical because in TS, these instructions have `rvalues = []` by default,
///   which prevents their operands from being followed during the memoization DFS.
///   Without this, intermediate allocating values (e.g., `bar(props)` in
///   `foo(bar(props).b + 1)`) would be incorrectly kept when they should be pruned.
fn collect_instruction_operands(
    value: &InstructionValue,
    operands: &mut Vec<DeclarationId>,
    shapes: &ShapeRegistry,
    force_memoize_primitives: bool,
) {
    // Primitive-producing instructions: when forceMemoizePrimitives is false,
    // return empty rvalues (matching TS `computeMemoizationInputs` behavior).
    // This prevents DFS from traversing through primitive values to their operands.
    if !force_memoize_primitives {
        match value {
            InstructionValue::Primitive(_)
            | InstructionValue::JsxText(_)
            | InstructionValue::BinaryExpression(_)
            | InstructionValue::UnaryExpression(_)
            | InstructionValue::TemplateLiteral(_)
            | InstructionValue::LoadGlobal(_)
            | InstructionValue::MetaProperty(_)
            | InstructionValue::Debugger(_)
            | InstructionValue::ComputedDelete(_)
            | InstructionValue::PropertyDelete(_)
            | InstructionValue::StartMemoize(_)
            | InstructionValue::FinishMemoize(_)
            | InstructionValue::NextPropertyOf(_) => {
                return;
            }
            _ => {}
        }
    }

    // Check noAlias for call-like instructions: if the callee's function
    // signature has noAlias=true, the call cannot alias its arguments and
    // we should return no rvalues.
    match value {
        InstructionValue::CallExpression(v) => {
            if has_no_alias(shapes, &v.callee) {
                return;
            }
        }
        InstructionValue::MethodCall(v) => {
            if has_no_alias(shapes, &v.property) {
                return;
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            if has_no_alias(shapes, &v.tag) {
                return;
            }
        }
        _ => {}
    }

    // Use the visitors to get all operands
    for place in crate::hir::visitors::each_instruction_value_operand(value) {
        operands.push(place.identifier.declaration_id);
    }
}

/// Check if a place's identifier type has a function signature with `noAlias=true`.
fn has_no_alias(shapes: &ShapeRegistry, place: &Place) -> bool {
    let Some(shape_id) = place.identifier.type_.shape_id() else {
        return false;
    };
    let Some(shape) = shapes.get(shape_id) else {
        return false;
    };
    let Some(sig) = &shape.function_type else {
        return false;
    };
    sig.no_alias
}

/// Collect additional lvalue entries for mutable operands of allocating instructions.
///
/// In the TS, for `CallExpression`, `MethodCall`, `TaggedTemplateExpression`,
/// `ArrayExpression`, `ObjectExpression`, `NewExpression`, `PropertyStore`,
/// `FunctionExpression`, `ObjectMethod`, `RegExpLiteral`:
/// mutable operands are treated as additional lvalues (they alias the result).
fn collect_mutable_operand_lvalues(
    value: &InstructionValue,
    shapes: &ShapeRegistry,
) -> Vec<(DeclarationId, MemoizationLevel)> {
    let mut lvalues = Vec::new();

    match value {
        // Call-like expressions: check noAlias first
        InstructionValue::CallExpression(v) => {
            if has_no_alias(shapes, &v.callee) {
                return lvalues;
            }
            for place in crate::hir::visitors::each_instruction_value_operand(value) {
                if place.effect.is_mutable() {
                    lvalues.push((place.identifier.declaration_id, MemoizationLevel::Memoized));
                }
            }
        }
        InstructionValue::MethodCall(v) => {
            if has_no_alias(shapes, &v.property) {
                return lvalues;
            }
            for place in crate::hir::visitors::each_instruction_value_operand(value) {
                if place.effect.is_mutable() {
                    lvalues.push((place.identifier.declaration_id, MemoizationLevel::Memoized));
                }
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            if has_no_alias(shapes, &v.tag) {
                return lvalues;
            }
            for place in crate::hir::visitors::each_instruction_value_operand(value) {
                if place.effect.is_mutable() {
                    lvalues.push((place.identifier.declaration_id, MemoizationLevel::Memoized));
                }
            }
        }
        // Non-call allocating expressions: always check mutable operands
        InstructionValue::ArrayExpression(_)
        | InstructionValue::ObjectExpression(_)
        | InstructionValue::NewExpression(_)
        | InstructionValue::PropertyStore(_)
        | InstructionValue::FunctionExpression(_)
        | InstructionValue::ObjectMethod(_)
        | InstructionValue::RegExpLiteral(_) => {
            for place in crate::hir::visitors::each_instruction_value_operand(value) {
                if place.effect.is_mutable() {
                    lvalues.push((place.identifier.declaration_id, MemoizationLevel::Memoized));
                }
            }
        }
        _ => {}
    }

    lvalues
}

/// Get the MemoizationLevel for a ReactiveValue (handles compound values).
fn classify_reactive_value(value: &ReactiveValue, opts: &PruneOptions<'_>) -> MemoizationLevel {
    match value {
        ReactiveValue::Instruction(iv) => classify_value(iv, opts),
        ReactiveValue::Logical(_)
        | ReactiveValue::Ternary(_)
        | ReactiveValue::Sequence(_)
        | ReactiveValue::OptionalCall(_) => MemoizationLevel::Conditional,
    }
}

fn collect_in_block(
    block: &ReactiveBlock,
    state: &mut State,
    active_scopes: &[ScopeId],
    opts: &PruneOptions<'_>,
) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                let instr = &instr_stmt.instruction;
                let level = classify_reactive_value(&instr.value, opts);

                // Collect operands (rvalues) — may be overridden for specific instruction kinds
                let mut operand_ids = Vec::new();
                let mut custom_rvalues = false;

                // Collect all lvalues (instruction lvalue + any inner lvalues from StoreLocal etc.)
                let mut lvalue_entries: Vec<(DeclarationId, MemoizationLevel)> = Vec::new();

                // For specific instruction kinds, use custom rvalue/lvalue logic matching TS
                if let ReactiveValue::Instruction(iv) = &instr.value {
                    match iv.as_ref() {
                        // ComputedStore (TS lines 711-726):
                        // object is an lvalue at Conditional, outer lvalue at Conditional,
                        // only value.value is an rvalue
                        InstructionValue::ComputedStore(v) => {
                            custom_rvalues = true;
                            operand_ids.push(v.value.identifier.declaration_id);
                            let object_id = state.resolve(v.object.identifier.declaration_id);
                            lvalue_entries.push((object_id, MemoizationLevel::Conditional));
                            if let Some(lvalue) = &instr.lvalue {
                                let lvalue_id = state.resolve(lvalue.identifier.declaration_id);
                                lvalue_entries.push((lvalue_id, MemoizationLevel::Conditional));
                            }
                        }
                        // ComputedLoad / PropertyLoad (TS lines 698-710):
                        // Only value.object is an rvalue, not value.property
                        InstructionValue::ComputedLoad(v) => {
                            custom_rvalues = true;
                            operand_ids.push(v.object.identifier.declaration_id);
                            if let Some(lvalue) = &instr.lvalue {
                                let lvalue_id = state.resolve(lvalue.identifier.declaration_id);
                                lvalue_entries.push((lvalue_id, level));
                            }
                        }
                        InstructionValue::PropertyLoad(v) => {
                            custom_rvalues = true;
                            operand_ids.push(v.object.identifier.declaration_id);
                            if let Some(lvalue) = &instr.lvalue {
                                let lvalue_id = state.resolve(lvalue.identifier.declaration_id);
                                lvalue_entries.push((lvalue_id, level));
                            }
                        }
                        // PrefixUpdate / PostfixUpdate (TS lines 634-647):
                        // value.lvalue is an extra lvalue at Conditional,
                        // only value.value is an rvalue
                        InstructionValue::PrefixUpdate(v) => {
                            custom_rvalues = true;
                            operand_ids.push(v.value.identifier.declaration_id);
                            let inner_lvalue_id = state.resolve(v.lvalue.identifier.declaration_id);
                            lvalue_entries.push((inner_lvalue_id, MemoizationLevel::Conditional));
                            if let Some(lvalue) = &instr.lvalue {
                                let lvalue_id = state.resolve(lvalue.identifier.declaration_id);
                                lvalue_entries.push((lvalue_id, MemoizationLevel::Conditional));
                            }
                        }
                        InstructionValue::PostfixUpdate(v) => {
                            custom_rvalues = true;
                            operand_ids.push(v.value.identifier.declaration_id);
                            let inner_lvalue_id = state.resolve(v.lvalue.identifier.declaration_id);
                            lvalue_entries.push((inner_lvalue_id, MemoizationLevel::Conditional));
                            if let Some(lvalue) = &instr.lvalue {
                                let lvalue_id = state.resolve(lvalue.identifier.declaration_id);
                                lvalue_entries.push((lvalue_id, MemoizationLevel::Conditional));
                            }
                        }
                        _ => {}
                    }
                }

                if !custom_rvalues {
                    collect_operands(
                        &instr.value,
                        &mut operand_ids,
                        opts.shapes,
                        opts.force_memoize_primitives,
                    );
                }

                // Resolve all operand IDs upfront
                let resolved_operands: Vec<DeclarationId> =
                    operand_ids.iter().map(|&id| state.resolve(id)).collect();

                // Ensure all operand nodes exist
                for &resolved in &resolved_operands {
                    state.ensure_identifier(resolved);
                }

                // For non-custom cases, add the outer lvalue with the classified level
                if !custom_rvalues {
                    if let Some(lvalue) = &instr.lvalue {
                        let lvalue_id = state.resolve(lvalue.identifier.declaration_id);
                        lvalue_entries.push((lvalue_id, level));
                    }
                }

                // For StoreLocal/DeclareLocal/StoreContext/DeclareContext/Destructure,
                // also process the inner target. Each instruction kind has specific
                // memoization levels matching TS `computeMemoizationInputs`.
                if let ReactiveValue::Instruction(iv) = &instr.value {
                    match iv.as_ref() {
                        InstructionValue::StoreLocal(v) => {
                            // TS: StoreLocal inner target = Conditional
                            let target_id = state.resolve(v.lvalue.place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Conditional));
                        }
                        InstructionValue::StoreContext(v) => {
                            // TS: StoreContext inner target = Memoized ("Should never be pruned")
                            let target_id = state.resolve(v.lvalue_place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Memoized));
                        }
                        InstructionValue::DeclareLocal(v) => {
                            // TS: DeclareLocal inner target = Unmemoized
                            let target_id = state.resolve(v.lvalue.place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Unmemoized));
                        }
                        InstructionValue::DeclareContext(v) => {
                            // TS: DeclareContext inner target = Memoized
                            let target_id = state.resolve(v.lvalue_place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Memoized));
                        }
                        InstructionValue::Destructure(v) => {
                            // TS: computePatternLValues — spread elements get Memoized,
                            // regular elements get Conditional (TS lines 354-391)
                            match &v.lvalue.pattern {
                                crate::hir::Pattern::Array(arr) => {
                                    for item in &arr.items {
                                        match item {
                                            crate::hir::ArrayPatternElement::Place(p) => {
                                                let target_id =
                                                    state.resolve(p.identifier.declaration_id);
                                                lvalue_entries.push((
                                                    target_id,
                                                    MemoizationLevel::Conditional,
                                                ));
                                            }
                                            crate::hir::ArrayPatternElement::Spread(s) => {
                                                let target_id = state
                                                    .resolve(s.place.identifier.declaration_id);
                                                lvalue_entries
                                                    .push((target_id, MemoizationLevel::Memoized));
                                            }
                                            crate::hir::ArrayPatternElement::Hole => {}
                                        }
                                    }
                                }
                                crate::hir::Pattern::Object(obj) => {
                                    for prop in &obj.properties {
                                        match prop {
                                            crate::hir::ObjectPatternProperty::Property(p) => {
                                                let target_id = state
                                                    .resolve(p.place.identifier.declaration_id);
                                                lvalue_entries.push((
                                                    target_id,
                                                    MemoizationLevel::Conditional,
                                                ));
                                            }
                                            crate::hir::ObjectPatternProperty::Spread(s) => {
                                                let target_id = state
                                                    .resolve(s.place.identifier.declaration_id);
                                                lvalue_entries
                                                    .push((target_id, MemoizationLevel::Memoized));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    // Collect mutable operands as additional lvalues.
                    // In the TS, for CallExpression, MethodCall, TaggedTemplateExpression,
                    // ArrayExpression, ObjectExpression, NewExpression, PropertyStore,
                    // FunctionExpression, ObjectMethod, RegExpLiteral: mutable operands
                    // are treated as additional lvalues because they alias the result.
                    for (mutable_id, mutable_level) in
                        collect_mutable_operand_lvalues(iv, opts.shapes)
                    {
                        let resolved_id = state.resolve(mutable_id);
                        lvalue_entries.push((resolved_id, mutable_level));
                    }
                }

                // Process each lvalue
                for &(lvalue_id, lv_level) in &lvalue_entries {
                    let node = state.ensure_identifier(lvalue_id);
                    node.level = join_levels(node.level, lv_level);
                    for &resolved in &resolved_operands {
                        if resolved != lvalue_id {
                            node.dependencies.insert(resolved);
                        }
                    }
                    for &scope_id in active_scopes {
                        node.scopes.insert(scope_id);
                    }
                }

                // Visit operands for scope association
                for &resolved in &resolved_operands {
                    let op_node = state.ensure_identifier(resolved);
                    for &scope_id in active_scopes {
                        op_node.scopes.insert(scope_id);
                    }
                }

                // Process embedded sequence instructions within compound reactive values.
                //
                // Port of TypeScript `computeMemoizationInputs('SequenceExpression')` which calls
                // `this.visitValueForMemoization(instr.id, instr.value, instr.lvalue)` for each
                // instruction in the sequence. This ensures that allocating instructions embedded
                // inside a ternary's consequent/alternate (e.g. `[-1, 1]` array literal inside
                // `t0 === undefined ? [-1, 1] : t0`) get their lvalue nodes properly classified
                // as Memoized. Without this, the array would be classified as Never and the
                // ternary result would not get a reactive scope.
                collect_embedded_sequence_instructions(&instr.value, state, active_scopes, opts);

                // Handle LoadLocal definitions
                if let ReactiveValue::Instruction(iv) = &instr.value {
                    if let InstructionValue::LoadLocal(v) = iv.as_ref() {
                        if let Some(lvalue) = &instr.lvalue {
                            state.definitions.insert(
                                lvalue.identifier.declaration_id,
                                v.place.identifier.declaration_id,
                            );
                        }
                    }

                    // Handle hook arguments escaping.
                    // In the TS, hook arguments are marked as escaping UNLESS the
                    // hook's function signature has noAlias=true.
                    match iv.as_ref() {
                        InstructionValue::CallExpression(call) => {
                            if crate::hir::environment::get_hook_kind_for_type(
                                opts.env,
                                &call.callee.identifier.type_,
                            )
                            .is_some()
                                && !has_no_alias(opts.shapes, &call.callee)
                            {
                                for arg in &call.args {
                                    let place = match arg {
                                        crate::hir::CallArg::Spread(s) => &s.place,
                                        crate::hir::CallArg::Place(p) => p,
                                    };
                                    let resolved = state.resolve(place.identifier.declaration_id);
                                    state.escaping_values.insert(resolved);
                                }
                            }
                        }
                        InstructionValue::MethodCall(call) => {
                            if crate::hir::environment::get_hook_kind_for_type(
                                opts.env,
                                &call.property.identifier.type_,
                            )
                            .is_some()
                                && !has_no_alias(opts.shapes, &call.property)
                            {
                                for arg in &call.args {
                                    let place = match arg {
                                        crate::hir::CallArg::Spread(s) => &s.place,
                                        crate::hir::CallArg::Place(p) => p,
                                    };
                                    let resolved = state.resolve(place.identifier.declaration_id);
                                    state.escaping_values.insert(resolved);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            ReactiveStatement::Terminal(term) => {
                // Return values escape
                if let ReactiveTerminal::Return(ret) = &term.terminal {
                    let resolved = state.resolve(ret.value.identifier.declaration_id);
                    state.escaping_values.insert(resolved);

                    // Associate return value with active scopes
                    let node = state.ensure_identifier(resolved);
                    for &scope_id in active_scopes {
                        node.scopes.insert(scope_id);
                    }
                }
                collect_in_terminal(&term.terminal, state, active_scopes, opts);
            }
            ReactiveStatement::Scope(scope) => {
                // Register scope node with its dependencies
                if !state.scopes.contains_key(&scope.scope.id) {
                    let dep_ids: Vec<DeclarationId> = scope
                        .scope
                        .dependencies
                        .iter()
                        .map(|d| state.resolve(d.identifier.declaration_id))
                        .collect();
                    state
                        .scopes
                        .insert(scope.scope.id, ScopeNode { dependencies: dep_ids, seen: false });
                }

                // Process scope's reassignments
                for reassignment in &scope.scope.reassignments {
                    let resolved = state.resolve(reassignment.declaration_id);
                    let node = state.ensure_identifier(resolved);
                    for &scope_id in active_scopes {
                        node.scopes.insert(scope_id);
                    }
                    node.scopes.insert(scope.scope.id);
                }

                // Recurse into scope body with this scope added to active scopes
                let mut inner_scopes = active_scopes.to_vec();
                inner_scopes.push(scope.scope.id);
                collect_in_block(&scope.instructions, state, &inner_scopes, opts);
            }
            ReactiveStatement::PrunedScope(scope) => {
                collect_in_block(&scope.instructions, state, active_scopes, opts);
            }
        }
    }
}

fn collect_in_terminal(
    terminal: &ReactiveTerminal,
    state: &mut State,
    active_scopes: &[ScopeId],
    opts: &PruneOptions<'_>,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            collect_in_block(&t.consequent, state, active_scopes, opts);
            if let Some(alt) = &t.alternate {
                collect_in_block(alt, state, active_scopes, opts);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    collect_in_block(block, state, active_scopes, opts);
                }
            }
        }
        ReactiveTerminal::While(t) => collect_in_block(&t.r#loop, state, active_scopes, opts),
        ReactiveTerminal::DoWhile(t) => collect_in_block(&t.r#loop, state, active_scopes, opts),
        ReactiveTerminal::For(t) => collect_in_block(&t.r#loop, state, active_scopes, opts),
        ReactiveTerminal::ForOf(t) => collect_in_block(&t.r#loop, state, active_scopes, opts),
        ReactiveTerminal::ForIn(t) => collect_in_block(&t.r#loop, state, active_scopes, opts),
        ReactiveTerminal::Label(t) => collect_in_block(&t.block, state, active_scopes, opts),
        ReactiveTerminal::Try(t) => {
            collect_in_block(&t.block, state, active_scopes, opts);
            collect_in_block(&t.handler, state, active_scopes, opts);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

/// Process instructions embedded within compound reactive values (Sequence, Ternary, Logical).
///
/// Port of the TypeScript `computeMemoizationInputs('SequenceExpression')` behavior, which
/// calls `this.visitValueForMemoization(instr.id, instr.value, instr.lvalue)` for each
/// instruction in the sequence. This ensures that allocating instructions embedded inside
/// a ternary/logical/sequence get their lvalue nodes classified with the proper level
/// (e.g., an `ArrayExpression` inside a ternary's consequent gets level `Memoized`).
///
/// Without this, embedded instructions only appear as operand dependencies (rvalues) but
/// their lvalue classification is never set, defaulting to `Never`. This causes the ternary
/// result to fail the "has memoized dependency" check and get pruned from reactive scopes.
fn collect_embedded_sequence_instructions(
    value: &ReactiveValue,
    state: &mut State,
    active_scopes: &[ScopeId],
    opts: &PruneOptions<'_>,
) {
    match value {
        ReactiveValue::Sequence(seq) => {
            // For each embedded instruction in the sequence, process it as if it were
            // a top-level instruction. This matches the TS behavior for 'SequenceExpression'.
            for embedded_instr in &seq.instructions {
                let level = classify_reactive_value(&embedded_instr.value, opts);
                let mut operand_ids = Vec::new();
                collect_operands(
                    &embedded_instr.value,
                    &mut operand_ids,
                    opts.shapes,
                    opts.force_memoize_primitives,
                );
                let resolved_operands: Vec<DeclarationId> =
                    operand_ids.iter().map(|&id| state.resolve(id)).collect();
                for &resolved in &resolved_operands {
                    state.ensure_identifier(resolved);
                }
                // Collect all lvalue entries for this embedded instruction,
                // matching the full lvalue logic in `collect_in_block`.
                let mut lvalue_entries: Vec<(DeclarationId, MemoizationLevel)> = Vec::new();
                if let Some(lvalue) = &embedded_instr.lvalue {
                    let lvalue_id = state.resolve(lvalue.identifier.declaration_id);
                    lvalue_entries.push((lvalue_id, level));
                }
                // Also process inner targets for StoreLocal, DeclareLocal, etc.
                // This mirrors `collect_in_block`'s inner-target handling (TS lines 558-573).
                // Without this, `StoreLocal Reassign x = $array` inside a ternary sequence
                // would only register the outer lvalue, leaving x's declaration_id out of
                // the graph and causing the scope to be incorrectly pruned.
                if let ReactiveValue::Instruction(iv) = &embedded_instr.value {
                    match iv.as_ref() {
                        InstructionValue::StoreLocal(v) => {
                            let target_id =
                                state.resolve(v.lvalue.place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Conditional));
                        }
                        InstructionValue::StoreContext(v) => {
                            let target_id =
                                state.resolve(v.lvalue_place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Memoized));
                        }
                        InstructionValue::DeclareLocal(v) => {
                            let target_id =
                                state.resolve(v.lvalue.place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Unmemoized));
                        }
                        InstructionValue::DeclareContext(v) => {
                            let target_id =
                                state.resolve(v.lvalue_place.identifier.declaration_id);
                            lvalue_entries.push((target_id, MemoizationLevel::Memoized));
                        }
                        InstructionValue::PrefixUpdate(v) => {
                            let inner_id =
                                state.resolve(v.lvalue.identifier.declaration_id);
                            lvalue_entries.push((inner_id, MemoizationLevel::Conditional));
                        }
                        InstructionValue::PostfixUpdate(v) => {
                            let inner_id =
                                state.resolve(v.lvalue.identifier.declaration_id);
                            lvalue_entries.push((inner_id, MemoizationLevel::Conditional));
                        }
                        _ => {}
                    }
                }
                // Apply all lvalue entries
                for &(lvalue_id, lv_level) in &lvalue_entries {
                    let node = state.ensure_identifier(lvalue_id);
                    node.level = join_levels(node.level, lv_level);
                    for &resolved in &resolved_operands {
                        if resolved != lvalue_id {
                            node.dependencies.insert(resolved);
                        }
                    }
                    for &scope_id in active_scopes {
                        node.scopes.insert(scope_id);
                    }
                }
                // Handle LoadLocal definitions inside sequences
                if let ReactiveValue::Instruction(iv) = &embedded_instr.value {
                    if let InstructionValue::LoadLocal(v) = iv.as_ref() {
                        if let Some(lvalue) = &embedded_instr.lvalue {
                            state.definitions.insert(
                                lvalue.identifier.declaration_id,
                                v.place.identifier.declaration_id,
                            );
                        }
                    }
                }
                // Recurse into nested sequences/ternaries/logicals within embedded instructions
                collect_embedded_sequence_instructions(
                    &embedded_instr.value,
                    state,
                    active_scopes,
                    opts,
                );
            }
            // Recurse into the sequence's final value
            collect_embedded_sequence_instructions(&seq.value, state, active_scopes, opts);
        }
        ReactiveValue::Ternary(ternary) => {
            // Recurse into consequent and alternate to process any embedded sequences
            collect_embedded_sequence_instructions(&ternary.consequent, state, active_scopes, opts);
            collect_embedded_sequence_instructions(&ternary.alternate, state, active_scopes, opts);
        }
        ReactiveValue::Logical(logical) => {
            collect_embedded_sequence_instructions(&logical.left, state, active_scopes, opts);
            collect_embedded_sequence_instructions(&logical.right, state, active_scopes, opts);
        }
        ReactiveValue::OptionalCall(opt) => {
            collect_embedded_sequence_instructions(&opt.value, state, active_scopes, opts);
        }
        ReactiveValue::Instruction(_) => {
            // Leaf — no embedded sequences to process
        }
    }
}

// =====================================================================================
// Phase 2: Compute memoized identifiers via DFS
// =====================================================================================

fn compute_memoized_identifiers(state: &mut State) -> FxHashSet<DeclarationId> {
    let mut memoized = FxHashSet::default();
    let escaping: Vec<DeclarationId> = state.escaping_values.iter().copied().collect();

    for value in escaping {
        visit_identifier(value, false, state, &mut memoized);
    }

    memoized
}

fn visit_identifier(
    id: DeclarationId,
    force_memoize: bool,
    state: &mut State,
    memoized: &mut FxHashSet<DeclarationId>,
) -> bool {
    let Some(node) = state.identifiers.get_mut(&id) else {
        return false;
    };
    if node.seen {
        return node.memoized;
    }
    node.seen = true;
    node.memoized = false;

    // Collect dependencies and scopes before recursive calls
    let deps: Vec<DeclarationId> = node.dependencies.iter().copied().collect();
    let level = node.level;
    let scope_ids: Vec<ScopeId> = node.scopes.iter().copied().collect();

    // Visit dependencies
    let mut has_memoized_dependency = false;
    for dep in deps {
        let is_dep_memoized = visit_identifier(dep, false, state, memoized);
        has_memoized_dependency |= is_dep_memoized;
    }

    // Determine if this identifier should be memoized
    let should_memoize = match level {
        MemoizationLevel::Memoized => true,
        MemoizationLevel::Conditional => has_memoized_dependency || force_memoize,
        MemoizationLevel::Unmemoized => force_memoize,
        MemoizationLevel::Never => false,
    };

    if should_memoize {
        let node = state.identifiers.get_mut(&id).unwrap();
        node.memoized = true;
        memoized.insert(id);

        // Force memoize scope dependencies
        for scope_id in scope_ids {
            force_memoize_scope_dependencies(scope_id, state, memoized);
        }
    }

    should_memoize
}

fn force_memoize_scope_dependencies(
    scope_id: ScopeId,
    state: &mut State,
    memoized: &mut FxHashSet<DeclarationId>,
) {
    let Some(node) = state.scopes.get_mut(&scope_id) else {
        return;
    };
    if node.seen {
        return;
    }
    node.seen = true;

    let deps: Vec<DeclarationId> = node.dependencies.clone();
    for dep in deps {
        visit_identifier(dep, true, state, memoized);
    }
}

// =====================================================================================
// Phase 3: Prune scopes
// =====================================================================================

fn prune_in_block(
    block: &mut ReactiveBlock,
    memoized: &FxHashSet<DeclarationId>,
    pruned_scopes: &mut FxHashSet<ScopeId>,
    reassignments: &mut FxHashMap<crate::hir::DeclarationId, Vec<crate::hir::Identifier>>,
) {
    let mut i = 0;
    while i < block.len() {
        match &mut block[i] {
            ReactiveStatement::Scope(scope) => {
                prune_in_block(&mut scope.instructions, memoized, pruned_scopes, reassignments);

                // Keep scopes with early returns (matches TS behavior)
                if scope.scope.early_return_value.is_some() {
                    i += 1;
                    continue;
                }

                // Keep scopes with no outputs (they may be needed for early returns later)
                if scope.scope.declarations.is_empty() && scope.scope.reassignments.is_empty() {
                    i += 1;
                    continue;
                }

                // Check if any declarations or reassignments are in the memoized set
                let has_memoized_output = scope
                    .scope
                    .declarations
                    .values()
                    .any(|decl| memoized.contains(&decl.identifier.declaration_id))
                    || scope
                        .scope
                        .reassignments
                        .iter()
                        .any(|ident| memoized.contains(&ident.declaration_id));

                if !has_memoized_output {
                    // Record pruned scope ID for FinishMemoize handling
                    pruned_scopes.insert(scope.scope.id);
                    // Scope doesn't need memoization — flatten it
                    let instructions = std::mem::take(&mut scope.instructions);
                    block.splice(i..=i, instructions);
                    continue;
                }
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_in_block(&mut scope.instructions, memoized, pruned_scopes, reassignments);
            }
            ReactiveStatement::Terminal(term) => {
                prune_in_terminal(&mut term.terminal, memoized, pruned_scopes, reassignments);
            }
            ReactiveStatement::Instruction(instr_stmt) => {
                // Track reassignments and set FinishMemoize.pruned, matching TS
                // PruneScopesTransform.transformInstruction
                let value = &instr_stmt.instruction.value;
                if let ReactiveValue::Instruction(iv) = value {
                    match iv.as_ref() {
                        InstructionValue::StoreLocal(v) => {
                            if v.lvalue.kind == crate::hir::InstructionKind::Reassign {
                                let entry = reassignments
                                    .entry(v.lvalue.place.identifier.declaration_id)
                                    .or_default();
                                entry.push(v.value.identifier.clone());
                            }
                        }
                        InstructionValue::LoadLocal(v) => {
                            if v.place.identifier.scope.is_some()
                                && let Some(lval) = &instr_stmt.instruction.lvalue
                                && lval.identifier.scope.is_none()
                            {
                                let entry = reassignments
                                    .entry(lval.identifier.declaration_id)
                                    .or_default();
                                entry.push(v.place.identifier.clone());
                            }
                        }
                        _ => {}
                    }
                }

                // Check FinishMemoize: set pruned=true if all decls' scopes were pruned
                if let ReactiveValue::Instruction(iv) = &instr_stmt.instruction.value {
                    if let InstructionValue::FinishMemoize(fm) = iv.as_ref() {
                        let decls: Vec<crate::hir::Identifier> =
                            if fm.decl.identifier.scope.is_none() {
                                reassignments.get(&fm.decl.identifier.declaration_id).map_or_else(
                                    || vec![fm.decl.identifier.clone()],
                                    |ids| ids.clone(),
                                )
                            } else {
                                vec![fm.decl.identifier.clone()]
                            };

                        let all_pruned = decls.iter().all(|decl| {
                            decl.scope.is_none()
                                || decl
                                    .scope
                                    .as_ref()
                                    .is_some_and(|s| pruned_scopes.contains(&s.id))
                        });

                        if all_pruned {
                            // Set pruned=true on the FinishMemoize instruction
                            // We need to get a mutable reference to the InstructionValue
                            if let ReactiveValue::Instruction(iv) =
                                &mut instr_stmt.instruction.value
                            {
                                if let InstructionValue::FinishMemoize(fm) = iv.as_mut() {
                                    fm.pruned = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
}

fn prune_in_terminal(
    terminal: &mut ReactiveTerminal,
    memoized: &FxHashSet<DeclarationId>,
    pruned_scopes: &mut FxHashSet<ScopeId>,
    reassignments: &mut FxHashMap<DeclarationId, Vec<crate::hir::Identifier>>,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            prune_in_block(&mut t.consequent, memoized, pruned_scopes, reassignments);
            if let Some(alt) = &mut t.alternate {
                prune_in_block(alt, memoized, pruned_scopes, reassignments);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    prune_in_block(block, memoized, pruned_scopes, reassignments);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            prune_in_block(&mut t.r#loop, memoized, pruned_scopes, reassignments);
        }
        ReactiveTerminal::DoWhile(t) => {
            prune_in_block(&mut t.r#loop, memoized, pruned_scopes, reassignments);
        }
        ReactiveTerminal::For(t) => {
            prune_in_block(&mut t.r#loop, memoized, pruned_scopes, reassignments);
        }
        ReactiveTerminal::ForOf(t) => {
            prune_in_block(&mut t.r#loop, memoized, pruned_scopes, reassignments);
        }
        ReactiveTerminal::ForIn(t) => {
            prune_in_block(&mut t.r#loop, memoized, pruned_scopes, reassignments);
        }
        ReactiveTerminal::Label(t) => {
            prune_in_block(&mut t.block, memoized, pruned_scopes, reassignments);
        }
        ReactiveTerminal::Try(t) => {
            prune_in_block(&mut t.block, memoized, pruned_scopes, reassignments);
            prune_in_block(&mut t.handler, memoized, pruned_scopes, reassignments);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}
