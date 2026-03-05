/// Propagate scope dependencies through the HIR.
///
/// Port of `HIR/PropagateScopeDependenciesHIR.ts` from the React Compiler.
///
/// This pass computes dependencies for each reactive scope — which values from
/// outside the scope are read inside it. The algorithm has 3 phases:
///
/// Phase 1: Build sidemaps
///   - `collect_temporaries_sidemap` maps temporary identifiers to their source
///     property chains (e.g. `$1 -> props.a.b`)
///   - `collect_optional_chain_dependencies` tracks optional chaining patterns
///   - `collect_hoistable_property_loads` identifies hoistable property loads
///
/// Phase 2: Collect dependencies per scope
///   - Walk the CFG tracking which scope is active
///   - For each instruction operand from outside the current scope, resolve
///     through temporaries and record as a dependency
///
/// Phase 3: Derive minimal dependency set
///   - Use `ReactiveScopeDependencyTree` to deduplicate and minimize
///   - e.g. if both `a.b` and `a.b.c` are used, only `a.b` is kept
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::SourceLocation;
use crate::hir::types::{ObjectType, PropertyLiteral, Type};

use super::hir_types::{
    BasicBlock, BlockId, DeclarationId, DependencyPath, DependencyPathEntry, HIRFunction,
    Identifier, IdentifierId, Instruction, InstructionId, InstructionKind, InstructionValue, Place,
    ReactiveScope, ReactiveScopeDependency, ScopeId, Terminal,
};
use super::visitors::{
    each_instruction_operand, each_instruction_value_operand, each_pattern_operand,
    each_terminal_operand, each_terminal_successor,
};

// =====================================================================================
// ScopeBlockTraversal — tracks entering/exiting reactive scopes during CFG traversal
// =====================================================================================

/// Information about a scope block boundary.
#[derive(Debug, Clone)]
enum ScopeBlockInfo {
    Begin { scope: ReactiveScope, pruned: bool },
    End { scope: ReactiveScope, pruned: bool },
}

/// Tracks entering/exiting reactive scopes during CFG traversal.
///
/// Port of `ScopeBlockTraversal` from `HIR/visitors.ts`.
struct ScopeBlockTraversal {
    /// Live stack of active scopes.
    active_scopes: Vec<ScopeId>,
    /// Maps block IDs to their scope boundary info.
    block_infos: FxHashMap<BlockId, ScopeBlockInfo>,
}

impl ScopeBlockTraversal {
    fn new() -> Self {
        Self { active_scopes: Vec::new(), block_infos: FxHashMap::default() }
    }

    /// Record scope boundaries from a block's terminal.
    fn record_scopes(&mut self, block: &BasicBlock) {
        let block_info = self.block_infos.get(&block.id);
        match block_info {
            Some(ScopeBlockInfo::Begin { scope, .. }) => {
                self.active_scopes.push(scope.id);
            }
            Some(ScopeBlockInfo::End { .. }) => {
                self.active_scopes.pop();
            }
            None => {}
        }

        match &block.terminal {
            Terminal::Scope(t) => {
                self.block_infos.insert(
                    t.block,
                    ScopeBlockInfo::Begin { scope: t.scope.clone(), pruned: false },
                );
                self.block_infos.insert(
                    t.fallthrough,
                    ScopeBlockInfo::End { scope: t.scope.clone(), pruned: false },
                );
            }
            Terminal::PrunedScope(t) => {
                self.block_infos.insert(
                    t.block,
                    ScopeBlockInfo::Begin { scope: t.scope.clone(), pruned: true },
                );
                self.block_infos.insert(
                    t.fallthrough,
                    ScopeBlockInfo::End { scope: t.scope.clone(), pruned: true },
                );
            }
            _ => {}
        }
    }

    /// Returns true if the given scope is currently active.
    fn is_scope_active(&self, scope_id: ScopeId) -> bool {
        self.active_scopes.contains(&scope_id)
    }

    /// The current, innermost active scope.
    fn current_scope(&self) -> Option<ScopeId> {
        self.active_scopes.last().copied()
    }
}

// =====================================================================================
// Immutable Stack — persistent stack for tracking scope nesting
// =====================================================================================

/// An immutable persistent stack. Each push creates a new "frame" that shares
/// the tail with its parent, matching the TS `Stack<T>`.
#[derive(Debug, Clone)]
enum Stack<T: Clone> {
    Empty,
    Node { value: T, next: Box<Stack<T>> },
}

impl<T: Clone> Stack<T> {
    fn empty() -> Self {
        Stack::Empty
    }

    fn push(&self, value: T) -> Self {
        Stack::Node { value, next: Box::new(self.clone()) }
    }

    fn pop(&self) -> Self {
        match self {
            Stack::Empty => Stack::Empty,
            Stack::Node { next, .. } => *next.clone(),
        }
    }

    fn value(&self) -> Option<&T> {
        match self {
            Stack::Empty => None,
            Stack::Node { value, .. } => Some(value),
        }
    }

    fn find(&self, f: &impl Fn(&T) -> bool) -> bool {
        match self {
            Stack::Empty => false,
            Stack::Node { value, next } => {
                if f(value) {
                    true
                } else {
                    next.find(f)
                }
            }
        }
    }

    fn each(&self, f: &mut impl FnMut(&T)) {
        match self {
            Stack::Empty => {}
            Stack::Node { value, next } => {
                f(value);
                next.each(f);
            }
        }
    }
}

// =====================================================================================
// ReactiveScopeDependencyTree — tree for deduplicating property-chain dependencies
// =====================================================================================

/// Access type for a property in the dependency tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyAccessType {
    OptionalAccess,
    UnconditionalAccess,
    OptionalDependency,
    UnconditionalDependency,
}

impl PropertyAccessType {
    fn is_optional(self) -> bool {
        matches!(self, PropertyAccessType::OptionalAccess | PropertyAccessType::OptionalDependency)
    }

    fn is_dependency(self) -> bool {
        matches!(
            self,
            PropertyAccessType::OptionalDependency | PropertyAccessType::UnconditionalDependency
        )
    }

    fn merge(self, other: PropertyAccessType) -> PropertyAccessType {
        let result_is_unconditional = !(self.is_optional() && other.is_optional());
        let result_is_dependency = self.is_dependency() || other.is_dependency();

        if result_is_unconditional {
            if result_is_dependency {
                PropertyAccessType::UnconditionalDependency
            } else {
                PropertyAccessType::UnconditionalAccess
            }
        } else if result_is_dependency {
            PropertyAccessType::OptionalDependency
        } else {
            PropertyAccessType::OptionalAccess
        }
    }
}

/// Access type for hoistable objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoistableAccessType {
    Optional,
    NonNull,
}

/// A node in the hoistable tree.
struct HoistableNode {
    properties: FxHashMap<PropertyLiteral, HoistableNode>,
    access_type: HoistableAccessType,
}

/// A node in the dependency tree.
struct DependencyNode {
    properties: FxHashMap<PropertyLiteral, DependencyNode>,
    access_type: PropertyAccessType,
    loc: SourceLocation,
}

/// Tree structure for deduplicating property-chain dependencies.
///
/// Port of `ReactiveScopeDependencyTreeHIR` from `DeriveMinimalDependenciesHIR.ts`.
struct ReactiveScopeDependencyTree {
    hoistable_objects: FxHashMap<IdentifierId, (HoistableNode, bool)>,
    deps: FxHashMap<IdentifierId, (DependencyNode, bool)>,
    /// Maps root IdentifierId to its full Identifier for reconstructing dependencies.
    root_identifiers: FxHashMap<IdentifierId, Identifier>,
}

impl ReactiveScopeDependencyTree {
    fn new(hoistable_objects: impl IntoIterator<Item = ReactiveScopeDependency>) -> Self {
        let mut hoistable_map: FxHashMap<IdentifierId, (HoistableNode, bool)> =
            FxHashMap::default();

        for dep in hoistable_objects {
            let root_access = if !dep.path.is_empty() && dep.path[0].optional {
                HoistableAccessType::Optional
            } else {
                HoistableAccessType::NonNull
            };

            let (root_node, _) = hoistable_map.entry(dep.identifier.id).or_insert_with(|| {
                (
                    HoistableNode { properties: FxHashMap::default(), access_type: root_access },
                    dep.reactive,
                )
            });

            let mut curr_node = root_node as *mut HoistableNode;
            for (i, entry) in dep.path.iter().enumerate() {
                let access_type = if i + 1 < dep.path.len() && dep.path[i + 1].optional {
                    HoistableAccessType::Optional
                } else {
                    HoistableAccessType::NonNull
                };

                // SAFETY: we only access through mutable pointer, no aliasing
                let node = unsafe { &mut *curr_node };
                let next = node.properties.entry(entry.property.clone()).or_insert_with(|| {
                    HoistableNode { properties: FxHashMap::default(), access_type }
                });
                curr_node = std::ptr::from_mut::<HoistableNode>(next);
            }
        }

        Self {
            hoistable_objects: hoistable_map,
            deps: FxHashMap::default(),
            root_identifiers: FxHashMap::default(),
        }
    }

    fn add_dependency(&mut self, dep: &ReactiveScopeDependency) {
        let identifier_id = dep.identifier.id;
        let reactive = dep.reactive;
        let loc = dep.loc;
        self.root_identifiers.entry(identifier_id).or_insert_with(|| dep.identifier.clone());

        let (dep_root, _) = self.deps.entry(identifier_id).or_insert_with(|| {
            (
                DependencyNode {
                    properties: FxHashMap::default(),
                    access_type: PropertyAccessType::UnconditionalAccess,
                    loc,
                },
                reactive,
            )
        });

        let hoistable_root = self.hoistable_objects.get(&identifier_id).map(|(n, _)| n);

        let mut dep_cursor = dep_root as *mut DependencyNode;
        let mut hoistable_cursor: Option<*const HoistableNode> =
            hoistable_root.map(std::ptr::from_ref::<HoistableNode>);

        for entry in &dep.path {
            let next_hoistable_cursor: Option<*const HoistableNode>;
            let access_type: PropertyAccessType;

            if entry.optional {
                if let Some(h_cursor) = hoistable_cursor {
                    // SAFETY: `h_cursor` points into `self.hoistable_objects` which is not mutated here.
                    let h_node = unsafe { &*h_cursor };
                    next_hoistable_cursor = h_node
                        .properties
                        .get(&entry.property)
                        .map(std::ptr::from_ref::<HoistableNode>);
                } else {
                    next_hoistable_cursor = None;
                }

                if let Some(h_cursor) = hoistable_cursor {
                    // SAFETY: `h_cursor` points into `self.hoistable_objects` which is not mutated here.
                    let h_node = unsafe { &*h_cursor };
                    if h_node.access_type == HoistableAccessType::NonNull {
                        access_type = PropertyAccessType::UnconditionalAccess;
                    } else {
                        access_type = PropertyAccessType::OptionalAccess;
                    }
                } else {
                    access_type = PropertyAccessType::OptionalAccess;
                }
            } else if let Some(h_cursor) = hoistable_cursor {
                // SAFETY: `h_cursor` points into `self.hoistable_objects` which is not mutated here.
                let h_node = unsafe { &*h_cursor };
                if h_node.access_type == HoistableAccessType::NonNull {
                    next_hoistable_cursor = h_node
                        .properties
                        .get(&entry.property)
                        .map(std::ptr::from_ref::<HoistableNode>);
                    access_type = PropertyAccessType::UnconditionalAccess;
                } else {
                    // Break to truncate the dependency
                    break;
                }
            } else {
                // Break to truncate the dependency
                break;
            }

            // Make or merge the property node
            // SAFETY: `dep_cursor` points into `self.deps` sub-nodes. Each iteration accesses
            // disjoint nodes, so no aliasing occurs.
            let dep_node = unsafe { &mut *dep_cursor };
            let child = dep_node.properties.entry(entry.property.clone()).or_insert_with(|| {
                DependencyNode { properties: FxHashMap::default(), access_type, loc: entry.loc }
            });
            child.access_type = child.access_type.merge(access_type);

            dep_cursor = std::ptr::from_mut::<DependencyNode>(child);
            hoistable_cursor = next_hoistable_cursor;
        }

        // Mark the final node as a dependency
        // SAFETY: `dep_cursor` is the final node pointer, with no other mutable references alive.
        let dep_node = unsafe { &mut *dep_cursor };
        dep_node.access_type = dep_node.access_type.merge(PropertyAccessType::OptionalDependency);
    }

    fn derive_minimal_dependencies(&self) -> Vec<ReactiveScopeDependency> {
        let mut results = Vec::new();
        for (&root_id, (root_node, reactive)) in &self.deps {
            if let Some(root_ident) = self.root_identifiers.get(&root_id) {
                collect_minimal_dependencies_in_subtree(
                    root_node,
                    *reactive,
                    root_ident,
                    &[],
                    &mut results,
                );
            }
        }
        results
    }
}

fn collect_minimal_dependencies_in_subtree(
    node: &DependencyNode,
    reactive: bool,
    root_identifier: &Identifier,
    path: &[DependencyPathEntry],
    results: &mut Vec<ReactiveScopeDependency>,
) {
    if node.access_type.is_dependency() {
        results.push(ReactiveScopeDependency {
            identifier: root_identifier.clone(),
            reactive,
            path: path.to_vec(),
            loc: node.loc,
        });
    } else {
        for (child_name, child_node) in &node.properties {
            let mut new_path = path.to_vec();
            new_path.push(DependencyPathEntry {
                property: child_name.clone(),
                optional: child_node.access_type.is_optional(),
                loc: child_node.loc,
            });
            collect_minimal_dependencies_in_subtree(
                child_node,
                reactive,
                root_identifier,
                &new_path,
                results,
            );
        }
    }
}

// =====================================================================================
// Helper: identifier type checks
// =====================================================================================

fn is_ref_value_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == "BuiltInRefValue"
    )
}

fn is_use_ref_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == "BuiltInUseRefId"
    )
}

fn is_object_method_type(identifier: &Identifier) -> bool {
    identifier.type_ == Type::ObjectMethod
}

// =====================================================================================
// Helper: getProperty — resolve a property load to its dependency chain
// =====================================================================================

fn get_property(
    object: &Place,
    property_name: &PropertyLiteral,
    optional: bool,
    loc: SourceLocation,
    temporaries: &FxHashMap<IdentifierId, TempReactiveScopeDependency>,
) -> TempReactiveScopeDependency {
    let resolved = temporaries.get(&object.identifier.id);

    match resolved {
        None => TempReactiveScopeDependency {
            identifier: object.identifier.clone(),
            reactive: object.reactive,
            path: vec![DependencyPathEntry { property: property_name.clone(), optional, loc }],
            loc,
        },
        Some(resolved_dep) => {
            let mut path = resolved_dep.path.clone();
            path.push(DependencyPathEntry { property: property_name.clone(), optional, loc });
            TempReactiveScopeDependency {
                identifier: resolved_dep.identifier.clone(),
                reactive: resolved_dep.reactive,
                path,
                loc,
            }
        }
    }
}

// =====================================================================================
// Temporaries sidemap types — uses full Identifier for richer tracking
// =====================================================================================

/// Internal dependency representation that keeps the full Identifier (not just ID).
/// This is needed during the collection phase to check type info and scope ranges.
#[derive(Debug, Clone)]
struct TempReactiveScopeDependency {
    identifier: Identifier,
    reactive: bool,
    path: DependencyPath,
    loc: SourceLocation,
}

impl TempReactiveScopeDependency {
    /// Convert to the public ReactiveScopeDependency.
    fn to_scope_dependency(&self) -> ReactiveScopeDependency {
        ReactiveScopeDependency {
            identifier: self.identifier.clone(),
            reactive: self.reactive,
            path: self.path.clone(),
            loc: self.loc,
        }
    }
}

// =====================================================================================
// Phase 1a: findTemporariesUsedOutsideDeclaringScope
// =====================================================================================

fn find_temporaries_used_outside_declaring_scope(func: &HIRFunction) -> FxHashSet<DeclarationId> {
    let mut declarations: FxHashMap<DeclarationId, ScopeId> = FxHashMap::default();
    let mut pruned_scopes: FxHashSet<ScopeId> = FxHashSet::default();
    let mut scope_traversal = ScopeBlockTraversal::new();
    let mut used_outside_declaring_scope: FxHashSet<DeclarationId> = FxHashSet::default();

    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();

    for &block_id in &block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else { continue };
        scope_traversal.record_scopes(block);

        let scope_start_info = scope_traversal.block_infos.get(&block_id);
        if let Some(ScopeBlockInfo::Begin { scope, pruned: true, .. }) = scope_start_info {
            pruned_scopes.insert(scope.id);
        }

        // Handle places: check if used outside declaring scope
        let handle_place =
            |place: &Place,
             declarations: &FxHashMap<DeclarationId, ScopeId>,
             pruned_scopes: &FxHashSet<ScopeId>,
             scope_traversal: &ScopeBlockTraversal,
             used_outside: &mut FxHashSet<DeclarationId>| {
                if let Some(&declaring_scope) = declarations.get(&place.identifier.declaration_id)
                    && !scope_traversal.is_scope_active(declaring_scope)
                    && !pruned_scopes.contains(&declaring_scope)
                {
                    used_outside.insert(place.identifier.declaration_id);
                }
            };

        // Handle instruction: record declarations
        let handle_instruction =
            |instr: &Instruction,
             declarations: &mut FxHashMap<DeclarationId, ScopeId>,
             scope_traversal: &ScopeBlockTraversal,
             pruned_scopes: &FxHashSet<ScopeId>| {
                let scope = scope_traversal.current_scope();
                let Some(scope) = scope else { return };
                if pruned_scopes.contains(&scope) {
                    return;
                }
                match &instr.value {
                    InstructionValue::LoadLocal(_)
                    | InstructionValue::LoadContext(_)
                    | InstructionValue::PropertyLoad(_) => {
                        declarations.insert(instr.lvalue.identifier.declaration_id, scope);
                    }
                    _ => {}
                }
            };

        for instr in &block.instructions {
            for place in each_instruction_operand(instr) {
                handle_place(
                    place,
                    &declarations,
                    &pruned_scopes,
                    &scope_traversal,
                    &mut used_outside_declaring_scope,
                );
            }
            handle_instruction(instr, &mut declarations, &scope_traversal, &pruned_scopes);
        }

        for place in each_terminal_operand(&block.terminal) {
            handle_place(
                place,
                &declarations,
                &pruned_scopes,
                &scope_traversal,
                &mut used_outside_declaring_scope,
            );
        }
    }

    used_outside_declaring_scope
}

// =====================================================================================
// Phase 1b: collectTemporariesSidemap
// =====================================================================================

fn collect_temporaries_sidemap(
    func: &HIRFunction,
    used_outside_declaring_scope: &FxHashSet<DeclarationId>,
) -> FxHashMap<IdentifierId, TempReactiveScopeDependency> {
    let mut temporaries = FxHashMap::default();
    collect_temporaries_sidemap_impl(func, used_outside_declaring_scope, &mut temporaries, None);
    temporaries
}

fn collect_temporaries_sidemap_impl(
    func: &HIRFunction,
    used_outside_declaring_scope: &FxHashSet<DeclarationId>,
    temporaries: &mut FxHashMap<IdentifierId, TempReactiveScopeDependency>,
    inner_fn_context: Option<InstructionId>,
) {
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let orig_instr_id = instr.id;
            let instr_id =
                if let Some(outer_id) = inner_fn_context { outer_id } else { orig_instr_id };

            let used_outside =
                used_outside_declaring_scope.contains(&instr.lvalue.identifier.declaration_id);

            match &instr.value {
                InstructionValue::PropertyLoad(v) if !used_outside => {
                    if inner_fn_context.is_none()
                        || temporaries.contains_key(&v.object.identifier.id)
                    {
                        let property =
                            get_property(&v.object, &v.property, false, v.loc, temporaries);
                        temporaries.insert(instr.lvalue.identifier.id, property);
                    }
                }
                InstructionValue::LoadLocal(v)
                    if instr.lvalue.identifier.name.is_none()
                        && v.place.identifier.name.is_some()
                        && !used_outside =>
                {
                    if inner_fn_context.is_none()
                        || func.context.iter().any(|ctx| ctx.identifier.id == v.place.identifier.id)
                    {
                        temporaries.insert(
                            instr.lvalue.identifier.id,
                            TempReactiveScopeDependency {
                                identifier: v.place.identifier.clone(),
                                reactive: v.place.reactive,
                                path: vec![],
                                loc: v.loc,
                            },
                        );
                    }
                }
                InstructionValue::LoadContext(v)
                    if instr.lvalue.identifier.name.is_none() && !used_outside =>
                {
                    if v.place.identifier.name.is_some()
                        && (inner_fn_context.is_none()
                            || func
                                .context
                                .iter()
                                .any(|ctx| ctx.identifier.id == v.place.identifier.id))
                    {
                        temporaries.insert(
                            instr.lvalue.identifier.id,
                            TempReactiveScopeDependency {
                                identifier: v.place.identifier.clone(),
                                reactive: v.place.reactive,
                                path: vec![],
                                loc: v.loc,
                            },
                        );
                    }
                }
                InstructionValue::FunctionExpression(v) => {
                    collect_temporaries_sidemap_impl(
                        &v.lowered_func.func,
                        used_outside_declaring_scope,
                        temporaries,
                        Some(inner_fn_context.unwrap_or(instr_id)),
                    );
                }
                InstructionValue::ObjectMethod(v) => {
                    collect_temporaries_sidemap_impl(
                        &v.lowered_func.func,
                        used_outside_declaring_scope,
                        temporaries,
                        Some(inner_fn_context.unwrap_or(instr_id)),
                    );
                }
                _ => {}
            }
        }
    }
}

// =====================================================================================
// Phase 2: DependencyCollectionContext + collectDependencies
// =====================================================================================

/// Declaration tracking for dependency collection.
#[derive(Debug, Clone)]
struct Decl {
    id: InstructionId,
    scope: Stack<ReactiveScope>,
}

/// Context for collecting dependencies during CFG traversal.
///
/// Port of `DependencyCollectionContext` from the TS reference.
struct DependencyCollectionContext<'a> {
    /// Maps declaration IDs to their first declaration info.
    declarations: FxHashMap<DeclarationId, Decl>,
    /// Maps identifier IDs to their latest reassignment info.
    reassignments: FxHashMap<IdentifierId, Decl>,

    /// Stack of active reactive scopes.
    scopes: Stack<ReactiveScope>,
    /// Stack of dependency arrays — one per active scope.
    dependencies: Stack<Vec<TempReactiveScopeDependency>>,
    /// Completed scope -> deps map.
    deps: Vec<(ReactiveScope, Vec<TempReactiveScopeDependency>)>,

    /// Temporaries sidemap.
    temporaries: &'a FxHashMap<IdentifierId, TempReactiveScopeDependency>,
    /// Instructions processed as part of optional chains (skip them).
    ///
    /// Owned rather than borrowed so it can be swapped when entering inner
    /// functions. InstructionIds overlap between inner and outer functions in the
    /// Rust HIR (unlike the TS reference which uses object references as Set
    /// keys), so each function level needs its own set to avoid false positives.
    processed_instrs_in_optional: FxHashSet<InstructionId>,

    /// Tracks whether we are inside an inner function.
    inner_fn_context: Option<InstructionId>,

    /// Collected scope declaration mutations to apply after the main traversal.
    /// Each entry is (scope_id, identifier_id, identifier, declaring_scope).
    scope_declaration_mutations: Vec<(ScopeId, IdentifierId, Identifier, ReactiveScope)>,

    /// Shadow tracker to prevent duplicate declaration mutations.
    /// The TS reference mutates `scope.declarations` in-place so subsequent checks
    /// within the same traversal see the updated state. In Rust we defer mutations,
    /// so we need this set to avoid queueing the same (scope, declaration) twice.
    pending_scope_declarations: FxHashSet<(ScopeId, DeclarationId)>,

    /// Collected scope reassignment mutations to apply after the main traversal.
    /// Each entry is (scope_id, identifier).
    scope_reassignment_mutations: Vec<(ScopeId, Identifier)>,
}

impl<'a> DependencyCollectionContext<'a> {
    fn new(
        temporaries: &'a FxHashMap<IdentifierId, TempReactiveScopeDependency>,
        processed_instrs_in_optional: FxHashSet<InstructionId>,
    ) -> Self {
        Self {
            declarations: FxHashMap::default(),
            reassignments: FxHashMap::default(),
            scopes: Stack::empty(),
            dependencies: Stack::empty(),
            deps: Vec::new(),
            temporaries,
            processed_instrs_in_optional,
            inner_fn_context: None,
            scope_declaration_mutations: Vec::new(),
            pending_scope_declarations: FxHashSet::default(),
            scope_reassignment_mutations: Vec::new(),
        }
    }

    fn enter_scope(&mut self, scope: ReactiveScope) {
        self.dependencies = self.dependencies.push(Vec::new());
        self.scopes = self.scopes.push(scope);
    }

    fn exit_scope(&mut self, scope: ReactiveScope, pruned: bool) {
        let scoped_dependencies = match self.dependencies.value() {
            Some(deps) => deps.clone(),
            None => return,
        };

        self.scopes = self.scopes.pop();
        self.dependencies = self.dependencies.pop();

        // Propagate dependencies upward: child scopes may have dependencies on
        // values created within the outer scope, which cannot be dependencies
        // of the outer scope.
        let parent_deps = self.dependencies.value().cloned();
        if let Some(mut parent) = parent_deps {
            for dep in &scoped_dependencies {
                if self.check_valid_dependency(dep) {
                    parent.push(dep.clone());
                }
            }
            self.dependencies = self.dependencies.pop().push(parent);
        }

        if !pruned {
            self.deps.push((scope, scoped_dependencies));
        }
    }

    fn declare(&mut self, identifier: &Identifier, decl: Decl) {
        if self.inner_fn_context.is_some() {
            return;
        }
        self.declarations.entry(identifier.declaration_id).or_insert_with(|| decl.clone());
        self.reassignments.insert(identifier.id, decl);
    }

    fn has_declared(&self, identifier: &Identifier) -> bool {
        self.declarations.contains_key(&identifier.declaration_id)
    }

    fn check_valid_dependency(&self, maybe_dep: &TempReactiveScopeDependency) -> bool {
        // ref value is not a valid dep
        if is_ref_value_type(&maybe_dep.identifier) {
            return false;
        }

        // object methods are not deps
        if is_object_method_type(&maybe_dep.identifier) {
            return false;
        }

        let current_declaration = self
            .reassignments
            .get(&maybe_dep.identifier.id)
            .or_else(|| self.declarations.get(&maybe_dep.identifier.declaration_id));

        let current_scope = self.scopes.value();

        match (current_scope, current_declaration) {
            (Some(scope), Some(decl)) => decl.id < scope.range.start,
            (None, _) | (_, None) => false,
        }
    }

    fn is_scope_active_in_stack(&self, scope: &ReactiveScope) -> bool {
        self.scopes.find(&|s| s.id == scope.id)
    }

    /// Declare a phi result in `reassignments` so that `check_valid_dependency` can
    /// correctly determine whether the phi result should be a scope dependency.
    ///
    /// In SSA form, phi results have a unique `IdentifierId` that is never stored in
    /// `reassignments` (since phi nodes are not lowered as regular instructions).
    /// Without this declaration, `check_valid_dependency` falls back to
    /// `declarations[declaration_id]`, which finds the variable's first declaration
    /// (typically a `DeclareLocal` or `let x = 0` before the scope). This causes phi
    /// results to always appear as external deps, even when ALL operands were assigned
    /// inside the scope.
    ///
    /// The fix: declare the phi result with the MINIMUM instruction id among all
    /// operand decls. This means:
    /// - If any operand was assigned before the current scope, the phi inherits that
    ///   pre-scope id → still a valid dep (correctly mirrors TS behavior where the
    ///   pre-scope value flows into the phi).
    /// - If ALL operands were assigned inside the current scope, the phi gets an
    ///   inside-scope id → NOT a valid dep (correctly mirrors TS where `#reassignments`
    ///   returns the inside-scope assignment for the same identifier).
    fn declare_phi_result(&mut self, phi: &crate::hir::hir_types::Phi) {
        if self.inner_fn_context.is_some() {
            return;
        }
        // Compute the minimum instruction id across all operand decls.
        let mut min_decl: Option<Decl> = None;
        for operand_place in phi.operands.values() {
            let operand_decl = self
                .reassignments
                .get(&operand_place.identifier.id)
                .or_else(|| self.declarations.get(&operand_place.identifier.declaration_id));
            if let Some(decl) = operand_decl {
                min_decl = Some(match min_decl {
                    None => decl.clone(),
                    Some(ref existing) if decl.id < existing.id => decl.clone(),
                    Some(existing) => existing,
                });
            }
        }
        // Declare the phi result in reassignments with the computed min decl.
        // If no operand decl was found, skip (phi result stays undeclared).
        if let Some(decl) = min_decl {
            self.reassignments.insert(phi.place.identifier.id, decl);
        }
    }

    fn visit_operand(&mut self, place: &Place) {
        let dep = match self.temporaries.get(&place.identifier.id) {
            Some(resolved) => resolved.clone(),
            None => TempReactiveScopeDependency {
                identifier: place.identifier.clone(),
                reactive: place.reactive,
                path: vec![],
                loc: place.loc,
            },
        };
        self.visit_dependency(dep);
    }

    fn visit_property(
        &mut self,
        object: &Place,
        property: &PropertyLiteral,
        optional: bool,
        loc: SourceLocation,
    ) {
        let next_dep = get_property(object, property, optional, loc, self.temporaries);
        self.visit_dependency(next_dep);
    }

    fn visit_dependency(&mut self, maybe_dep: TempReactiveScopeDependency) {
        // Record declarations for scopes that the original declaration was in.
        // In the TS version, this mutates scope.declarations directly. In Rust,
        // we collect mutations in a side-table and apply them in a second pass.
        let original_declaration =
            self.declarations.get(&maybe_dep.identifier.declaration_id).cloned();
        if let Some(ref original_decl) = original_declaration
            && let Some(declaring_scope) = original_decl.scope.value()
        {
            let scopes_to_update: Vec<ReactiveScope> = {
                let mut result = Vec::new();
                original_decl.scope.each(&mut |scope| {
                    result.push(scope.clone());
                });
                result
            };

            for scope in &scopes_to_update {
                let is_active = self.is_scope_active_in_stack(scope);
                let has_decl = scope.declarations.values().any(|decl| {
                    decl.identifier.declaration_id == maybe_dep.identifier.declaration_id
                }) || self
                    .pending_scope_declarations
                    .contains(&(scope.id, maybe_dep.identifier.declaration_id));
                if !is_active && !has_decl {
                    self.pending_scope_declarations
                        .insert((scope.id, maybe_dep.identifier.declaration_id));
                    self.scope_declaration_mutations.push((
                        scope.id,
                        maybe_dep.identifier.id,
                        maybe_dep.identifier.clone(),
                        declaring_scope.clone(),
                    ));
                }
            }
        }

        // Handle ref.current access
        let mut dep = maybe_dep;
        if is_use_ref_type(&dep.identifier)
            && let Some(first) = dep.path.first()
            && first.property == PropertyLiteral::String("current".to_string())
        {
            dep = TempReactiveScopeDependency {
                identifier: dep.identifier.clone(),
                reactive: dep.reactive,
                path: vec![],
                loc: dep.loc,
            };
        }

        let is_valid = self.check_valid_dependency(&dep);
        if is_valid && let Some(current_deps) = self.dependencies.value() {
            let mut new_deps = current_deps.clone();
            new_deps.push(dep);
            self.dependencies = self.dependencies.pop().push(new_deps);
        }
    }

    fn visit_reassignment(&mut self, place: &Place) {
        let current_scope = self.scopes.value().cloned();
        if let Some(ref scope) = current_scope
            && !scope
                .reassignments
                .iter()
                .any(|r| r.declaration_id == place.identifier.declaration_id)
        {
            let is_valid = self.check_valid_dependency(&TempReactiveScopeDependency {
                identifier: place.identifier.clone(),
                reactive: place.reactive,
                path: vec![],
                loc: place.loc,
            });
            if is_valid {
                // Collect scope reassignment mutation for later application
                self.scope_reassignment_mutations.push((scope.id, place.identifier.clone()));
            }
        }
    }

    fn enter_inner_fn(&mut self, instr_id: InstructionId) {
        if self.inner_fn_context.is_none() {
            self.inner_fn_context = Some(instr_id);
        }
    }

    fn exit_inner_fn(&mut self, was_none: bool) {
        if was_none {
            self.inner_fn_context = None;
        }
    }

    /// Check if an instruction should be deferred (already processed in optional chain
    /// or is a temporary in the sidemap).
    fn is_deferred_instruction(&self, instr: &Instruction) -> bool {
        self.processed_instrs_in_optional.contains(&instr.id)
            || self.temporaries.contains_key(&instr.lvalue.identifier.id)
    }

    /// Check if a terminal should be deferred.
    fn is_deferred_terminal(&self, terminal: &Terminal) -> bool {
        self.processed_instrs_in_optional.contains(&terminal.id())
    }
}

// =====================================================================================
// handleInstruction — process a single instruction for dependency collection
// =====================================================================================

fn handle_instruction(instr: &Instruction, context: &mut DependencyCollectionContext) {
    let id = instr.id;

    context.declare(&instr.lvalue.identifier, Decl { id, scope: context.scopes.clone() });

    if context.is_deferred_instruction(instr) {
        return;
    }

    match &instr.value {
        InstructionValue::PropertyLoad(v) => {
            context.visit_property(&v.object, &v.property, false, v.loc);
        }
        InstructionValue::StoreLocal(v) => {
            context.visit_operand(&v.value);
            if v.lvalue.kind == InstructionKind::Reassign {
                context.visit_reassignment(&v.lvalue.place);
            }
            context.declare(&v.lvalue.place.identifier, Decl { id, scope: context.scopes.clone() });
        }
        InstructionValue::DeclareLocal(v) => {
            // Only declare non-hoisted lvalues
            if v.lvalue.kind.convert_hoisted().is_none() {
                context.declare(
                    &v.lvalue.place.identifier,
                    Decl { id, scope: context.scopes.clone() },
                );
            }
        }
        InstructionValue::DeclareContext(v) => {
            if v.lvalue_kind.convert_hoisted().is_none() {
                // In the TS reference, DeclareContext's mutable range starts at the
                // instruction itself, so the scope (from InferReactiveScopeVariables)
                // is already active when declare() is called. In Rust, DeclareContext's
                // mutable range starts later (at the first StoreContext), so the scope
                // may not yet be active. To match TS behavior, when the current scope
                // stack is empty but the identifier has an assigned scope, use that
                // scope so visitDependency can later add it as a scope declaration.
                let decl_scope = if context.scopes.value().is_none() {
                    if let Some(ref var_scope) = v.lvalue_place.identifier.scope {
                        Stack::empty().push(var_scope.as_ref().clone())
                    } else {
                        context.scopes.clone()
                    }
                } else {
                    context.scopes.clone()
                };
                context.declare(&v.lvalue_place.identifier, Decl { id, scope: decl_scope });
            }
        }
        InstructionValue::Destructure(v) => {
            context.visit_operand(&v.value);
            for place in each_pattern_operand(&v.lvalue.pattern) {
                if v.lvalue.kind == InstructionKind::Reassign {
                    context.visit_reassignment(place);
                }
                context.declare(&place.identifier, Decl { id, scope: context.scopes.clone() });
            }
        }
        InstructionValue::StoreContext(v) => {
            if !context.has_declared(&v.lvalue_place.identifier)
                || v.lvalue_kind != InstructionKind::Reassign
            {
                // Same scope compensation as DeclareContext: when the current scope
                // stack is empty but the identifier has an assigned scope, use that
                // scope to match TS behavior.
                let decl_scope = if context.scopes.value().is_none() {
                    if let Some(ref var_scope) = v.lvalue_place.identifier.scope {
                        Stack::empty().push(var_scope.as_ref().clone())
                    } else {
                        context.scopes.clone()
                    }
                } else {
                    context.scopes.clone()
                };
                context.declare(&v.lvalue_place.identifier, Decl { id, scope: decl_scope });
            }

            for operand in each_instruction_value_operand(&instr.value) {
                context.visit_operand(operand);
            }
        }
        InstructionValue::StartMemoize(_) => {
            // StartMemoize contains manually-specified dependency information from the user's
            // useMemo/useCallback call. Its root Places should NOT be visited as inferred scope
            // dependencies — doing so would add coarser-grained deps (e.g. `props`) instead of
            // the user's intended finer-grained ones (e.g. `props.value`). Scope dependency
            // inference works independently on the actual instructions inside the scope body.
        }
        InstructionValue::FinishMemoize(fm) => {
            // FinishMemoize.decl must be visited so that the declaring scope can register it
            // as an output declaration. This mirrors TypeScript's eachInstructionValueOperand
            // which yields `instrValue.decl` for FinishMemoize. Without this visit, the scope
            // that owns the memoized value would have empty declarations, causing
            // PruneNonEscapingScopes to keep the scope (empty-decls path) without adding it
            // to `pruned_scopes`, and then FinishMemoize.pruned would never be set to true.
            // Note: check_valid_dependency returns false when there is no active scope, so
            // the decl will NOT be added as a scope dependency — only as a declaration.
            context.visit_operand(&fm.decl);
        }
        InstructionValue::LoadContext(v) => {
            context.visit_operand(&v.place);
        }
        _ => {
            for operand in each_instruction_value_operand(&instr.value) {
                context.visit_operand(operand);
            }
        }
    }
}

// =====================================================================================
// collectDependencies — main CFG traversal
// =====================================================================================

/// Result of dependency collection including collected scope mutations.
struct CollectDependenciesResult {
    deps: Vec<(ReactiveScope, Vec<TempReactiveScopeDependency>)>,
    /// Scope declaration mutations: (scope_id, identifier_id, identifier, declaring_scope).
    scope_declaration_mutations: Vec<(ScopeId, IdentifierId, Identifier, ReactiveScope)>,
    /// Scope reassignment mutations: (scope_id, identifier).
    scope_reassignment_mutations: Vec<(ScopeId, Identifier)>,
}

fn collect_dependencies(
    func: &HIRFunction,
    temporaries: &FxHashMap<IdentifierId, TempReactiveScopeDependency>,
    processed_instrs_in_optional: FxHashSet<InstructionId>,
    intermediate_optional_results: &FxHashSet<IdentifierId>,
) -> Result<CollectDependenciesResult, crate::compiler_error::CompilerError> {
    fn handle_function(
        func: &HIRFunction,
        context: &mut DependencyCollectionContext,
        scope_traversal: &mut ScopeBlockTraversal,
        temporaries: &FxHashMap<IdentifierId, TempReactiveScopeDependency>,
        intermediate_optional_results: &FxHashSet<IdentifierId>,
    ) -> Result<(), crate::compiler_error::CompilerError> {
        let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
        for &block_id in &block_ids {
            let Some(block) = func.body.blocks.get(&block_id) else { continue };
            scope_traversal.record_scopes(block);

            let scope_block_info = scope_traversal.block_infos.get(&block_id).cloned();
            match &scope_block_info {
                Some(ScopeBlockInfo::Begin { scope, .. }) => {
                    context.enter_scope(scope.clone());
                }
                Some(ScopeBlockInfo::End { scope, pruned }) => {
                    context.exit_scope(scope.clone(), *pruned);
                }
                None => {}
            }

            // Record referenced optional chains in phis.
            // Skip phi operands that are intermediate results of nested optional chains
            // (i.e. they are consumed by an outer optional chain and should not be
            // tracked as deps themselves).
            for phi in &block.phis {
                for operand_place in phi.operands.values() {
                    // Skip intermediate optional chain results — they represent partial
                    // optional chain results (e.g. `a?.b` in `a?.b.c`) that are consumed
                    // by an outer optional chain.  The outer chain's phi already carries
                    // the full dep (e.g. `a?.b.c`), so adding `a?.b` here would introduce
                    // a spurious coarser dep.
                    if intermediate_optional_results.contains(&operand_place.identifier.id) {
                        continue;
                    }
                    if let Some(maybe_optional_chain) =
                        temporaries.get(&operand_place.identifier.id)
                    {
                        context.visit_dependency(maybe_optional_chain.clone());
                    }
                }
                // Declare the phi result in `reassignments` so that uses of the phi result
                // (which has a unique IdentifierId in SSA form) can be correctly evaluated
                // by `check_valid_dependency`. The phi result inherits the minimum instruction
                // id from its operands: if all operands were assigned inside the current scope,
                // the phi result is considered inside-scope (not a dep); if any operand was
                // assigned before the scope, the phi result can still be a dep.
                context.declare_phi_result(phi);
            }

            for instr in &block.instructions {
                match &instr.value {
                    InstructionValue::FunctionExpression(_) | InstructionValue::ObjectMethod(_) => {
                        context.declare(
                            &instr.lvalue.identifier,
                            Decl { id: instr.id, scope: context.scopes.clone() },
                        );

                        let inner_func = match &instr.value {
                            InstructionValue::FunctionExpression(v) => &v.lowered_func.func,
                            InstructionValue::ObjectMethod(v) => &v.lowered_func.func,
                            _ => continue,
                        };

                        // Swap processed_instrs_in_optional with the inner function's
                        // own set. InstructionIds overlap between inner and outer
                        // functions in the Rust HIR (the TS reference uses object
                        // references as Set keys which are naturally unique per
                        // function), so each function level needs its own set to
                        // avoid false-positive deferrals.
                        let inner_opt_chain =
                            super::collect_optional_chain_dependencies::collect_optional_chain_sidemap(inner_func);
                        let saved_processed = std::mem::replace(
                            &mut context.processed_instrs_in_optional,
                            inner_opt_chain.processed_instrs_in_optional,
                        );

                        let was_none = context.inner_fn_context.is_none();
                        context.enter_inner_fn(instr.id);
                        handle_function(
                            inner_func,
                            context,
                            scope_traversal,
                            temporaries,
                            intermediate_optional_results,
                        )?;
                        context.exit_inner_fn(was_none);

                        // Restore the outer function's processed instructions.
                        context.processed_instrs_in_optional = saved_processed;
                    }
                    _ => {
                        handle_instruction(instr, context);
                    }
                }
            }

            if !context.is_deferred_terminal(&block.terminal) {
                for place in each_terminal_operand(&block.terminal) {
                    context.visit_operand(place);
                }
            }
        }
        Ok(())
    }

    let mut context = DependencyCollectionContext::new(temporaries, processed_instrs_in_optional);

    // Declare params
    for param in &func.params {
        match param {
            super::hir_types::ReactiveParam::Place(p) => {
                context.declare(
                    &p.identifier,
                    Decl { id: InstructionId::ZERO, scope: Stack::empty() },
                );
            }
            super::hir_types::ReactiveParam::Spread(s) => {
                context.declare(
                    &s.place.identifier,
                    Decl { id: InstructionId::ZERO, scope: Stack::empty() },
                );
            }
        }
    }

    let mut scope_traversal = ScopeBlockTraversal::new();
    handle_function(
        func,
        &mut context,
        &mut scope_traversal,
        temporaries,
        intermediate_optional_results,
    )?;
    Ok(CollectDependenciesResult {
        deps: context.deps,
        scope_declaration_mutations: context.scope_declaration_mutations,
        scope_reassignment_mutations: context.scope_reassignment_mutations,
    })
}

// =====================================================================================
// Main entry point
// =====================================================================================

/// Propagate scope dependencies through the HIR.
///
/// For each reactive scope, computes the set of external values (dependencies)
/// that are read inside the scope. The result is stored on each scope's
/// `dependencies` field.
///
/// # Errors
///
/// Returns a `CompilerError` if dependency propagation encounters an invariant violation.
pub fn propagate_scope_dependencies_hir(
    func: &mut HIRFunction,
) -> Result<(), crate::compiler_error::CompilerError> {
    let used_outside_declaring_scope = find_temporaries_used_outside_declaring_scope(func);

    let temporaries = collect_temporaries_sidemap(func, &used_outside_declaring_scope);

    // Collect optional chain sidemap
    let opt_chain =
        super::collect_optional_chain_dependencies::collect_optional_chain_sidemap(func);

    // Merge temporaries with optional chain temporaries.
    //
    // The TypeScript reference merges as:
    //   new Map([...temporaries, ...temporariesReadInOptional])
    // where later entries override earlier ones.  This means
    // `temporariesReadInOptional` takes priority over `temporaries` for any key
    // present in both maps.  This is important because
    // `collect_temporaries_sidemap_impl` may have processed a PropertyLoad from
    // an optional-chain result and recorded an unresolved entry (e.g. `{
    // identifier: opt_result1, path: [.value] }`), while
    // `temporariesReadInOptional` has the correct fully-resolved entry (e.g. `{
    // identifier: prop2, path: [?.inner, .value] }`).  By giving
    // `temporariesReadInOptional` priority we avoid the stale entry.
    let mut merged_temporaries = temporaries;
    for (id, dep) in &opt_chain.temporaries_read_in_optional {
        merged_temporaries.insert(
            *id,
            TempReactiveScopeDependency {
                identifier: dep.identifier.clone(),
                reactive: dep.reactive,
                path: dep.path.clone(),
                loc: dep.loc,
            },
        );
    }

    // Build a ReactiveScopeDependency map from merged_temporaries for hoistable analysis
    let temporaries_for_hoistable: FxHashMap<IdentifierId, ReactiveScopeDependency> =
        merged_temporaries.iter().map(|(&id, temp)| (id, temp.to_scope_dependency())).collect();

    // Collect hoistable property loads
    let hoistable = super::collect_hoistable_property_loads::collect_hoistable_property_loads(
        func,
        &temporaries_for_hoistable,
        &opt_chain.hoistable_objects,
    );

    // Key hoistable by scope ID, resolving PropertyPathNodes to full paths
    let mut hoistable_by_scope =
        super::collect_hoistable_property_loads::key_by_scope_id_with_registry(func, &hoistable);

    // Augment hoistable_by_scope with entries from opt_chain.hoistable_objects.
    //
    // In the TypeScript HIR, `prop2?.inner.value` is lowered as a single Optional
    // terminal whose test block contains both LoadLocal(prop2) AND PropertyLoad(prop2, "inner").
    // The PropertyLoad makes `prop2?.inner` non-null in the test block, which then
    // propagates backward to the scope body block via CFG analysis.
    //
    // In the Rust HIR, `prop2?.inner` is a SEPARATE inner Optional terminal (optional=true)
    // and `prop2?.inner.value` uses an OUTER Optional terminal (optional=false). The
    // hoistable dep `{prop2, path: [?.inner]}` is recorded on the OUTER Optional block,
    // which is inside the scope body. The backward propagation in
    // collect_hoistable_property_loads may fail to carry this info to the scope body
    // start block because intermediate blocks have multiple successors.
    //
    // To match TS behavior, we find the scope containing each hoistable_objects block
    // and add the dep to that scope's hoistable set.
    //
    // However, we must NOT augment hoistable objects that are inside ANOTHER optional
    // chain's conditional branch. For example, in `prop3?.fn(prop4?.inner.value)`, the
    // blocks for `prop4?.inner.value` are inside `prop3?`'s conditional consequent.
    // The hoistable dep `prop4?.inner` should NOT be added to the scope.
    //
    // To distinguish, we collect the set of blocks guarded by each Optional(optional=true)
    // chain's conditional branch, along with the root variable's declaration_id. A
    // hoistable dep is skipped if its block is guarded by a chain whose root differs
    // from the dep's root.
    if !opt_chain.hoistable_objects.is_empty() {
        // For each Optional(optional=true) block, find:
        //   1. The root variable's declaration_id (from the LoadLocal in the deepest test block)
        //   2. The Branch consequent block (start of the guarded region)
        //   3. The Optional fallthrough block (end of the guarded region)
        //
        // Then collect all blocks reachable from the consequent until the fallthrough
        // and map them to the root variable's declaration_id.
        //
        // A block can be guarded by multiple optional chains (nested optionals).
        let mut guarded_blocks: FxHashMap<BlockId, Vec<DeclarationId>> = FxHashMap::default();

        for block in func.body.blocks.values() {
            if let Terminal::Optional(opt) = &block.terminal {
                if !opt.optional {
                    continue;
                }

                // Trace the test chain to find the Branch terminal and the LoadLocal source.
                let mut current_test = opt.test;
                let branch_info: Option<(DeclarationId, BlockId)> = loop {
                    let Some(test_block) = func.body.blocks.get(&current_test) else {
                        break None;
                    };
                    match &test_block.terminal {
                        Terminal::Branch(branch) => {
                            // Find the root variable's declaration_id from the test block's
                            // LoadLocal/LoadContext instruction. The Branch tests a temporary
                            // created by LoadLocal, but we need the ORIGINAL variable.
                            let root_decl = if test_block.instructions.is_empty() {
                                None
                            } else {
                                match &test_block.instructions[0].value {
                                    InstructionValue::LoadLocal(v) => {
                                        Some(v.place.identifier.declaration_id)
                                    }
                                    InstructionValue::LoadContext(v) => {
                                        Some(v.place.identifier.declaration_id)
                                    }
                                    _ => None,
                                }
                            };
                            break root_decl.map(|decl| (decl, branch.consequent));
                        }
                        Terminal::Optional(inner_opt) => {
                            current_test = inner_opt.test;
                        }
                        _ => break None,
                    }
                };

                let Some((root_decl_id, consequent_block_id)) = branch_info else {
                    continue;
                };

                // Collect all blocks guarded by this Optional(optional=true) chain.
                //
                // There are two regions to consider:
                //
                // 1. Blocks reachable from the Branch consequent (inside the initial
                //    null check). This covers the property load for the base access.
                //
                // 2. Blocks reachable from the Optional's fallthrough block's Branch
                //    consequent. The fallthrough block is typically a phi + Branch that
                //    continues the chain: if the variable was non-null, it takes the
                //    consequent (which includes the rest of the chain like `.fn(...)` and
                //    any nested chains like `prop4?.inner.value`). If the variable was
                //    null, it takes the alternate (bail out).
                //
                // Both regions are guarded by the same variable's nullability.
                let fallthrough = opt.fallthrough;

                // Find the continuation consequent from the fallthrough block.
                let continuation_consequent =
                    func.body.blocks.get(&fallthrough).and_then(|fb| match &fb.terminal {
                        Terminal::Branch(branch) => Some(branch.consequent),
                        _ => None,
                    });

                // Collect the parent Optional's fallthrough (the enclosing non-optional
                // chain's fallthrough) as a stop boundary. Walk up from the current
                // Optional(optional=true) block through its predecessors to find the
                // nearest Optional(optional=false) parent and use ITS fallthrough.
                let parent_fallthrough = {
                    let mut pf: Option<BlockId> = None;
                    // Walk up predecessors: the Optional(optional=true) block's pred is
                    // an Optional(optional=false) block.
                    for &pred_id in &block.preds {
                        if let Some(pred_block) = func.body.blocks.get(&pred_id)
                            && let Terminal::Optional(parent_opt) = &pred_block.terminal
                            && !parent_opt.optional
                        {
                            pf = Some(parent_opt.fallthrough);
                        }
                    }
                    pf
                };

                // BFS from both the initial consequent AND the continuation consequent,
                // stopping at the parent Optional's fallthrough.
                let mut start_blocks = vec![consequent_block_id];
                if let Some(cont) = continuation_consequent {
                    start_blocks.push(cont);
                }

                let mut visit_queue: Vec<BlockId> = start_blocks;
                let mut visited: FxHashSet<BlockId> = FxHashSet::default();
                // Don't enter the fallthrough itself or the parent's fallthrough
                visited.insert(fallthrough);
                if let Some(pf) = parent_fallthrough {
                    visited.insert(pf);
                }

                while let Some(bid) = visit_queue.pop() {
                    if !visited.insert(bid) {
                        continue;
                    }
                    guarded_blocks.entry(bid).or_default().push(root_decl_id);

                    // Follow successors of this block.
                    if let Some(b) = func.body.blocks.get(&bid) {
                        for succ in each_terminal_successor(&b.terminal) {
                            visit_queue.push(succ);
                        }
                    }
                }
            }
        }

        // Build a list of (scope_id, scope_range) from scope terminals.
        let scope_ranges: Vec<(
            ScopeId,
            super::hir_types::InstructionId,
            super::hir_types::InstructionId,
        )> = func
            .body
            .blocks
            .values()
            .filter_map(|b| match &b.terminal {
                Terminal::Scope(t) => Some((t.scope.id, t.scope.range.start, t.scope.range.end)),
                Terminal::PrunedScope(t) => {
                    Some((t.scope.id, t.scope.range.start, t.scope.range.end))
                }
                _ => None,
            })
            .collect();

        for (opt_block_id, opt_dep) in &opt_chain.hoistable_objects {
            // Skip entries with empty paths. These come from the base case of
            // non-optional Optional terminals (e.g. the `node` in `node.fields?.[field]`)
            // and are already handled by the `hoistable_from_optionals` mechanism in
            // `collect_hoistable_property_loads`. Augmenting them directly into the
            // scope's hoistable set would bypass the CFG intersection logic, incorrectly
            // marking potentially-null identifiers (like `node = cond ? val : null`) as
            // unconditionally non-null at the scope level.
            if opt_dep.path.is_empty() {
                continue;
            }

            let dep_decl_id = opt_dep.identifier.declaration_id;

            // Check if this block is guarded by an optional chain for a DIFFERENT variable.
            // If so, the dep is nested inside another chain's conditional branch and should
            // NOT be augmented into the scope's hoistable set.
            let has_foreign_guard = guarded_blocks
                .get(opt_block_id)
                .is_some_and(|guards| guards.iter().any(|&guard_decl| guard_decl != dep_decl_id));

            if has_foreign_guard {
                continue;
            }

            let Some(opt_block) = func.body.blocks.get(opt_block_id) else { continue };

            let block_instr_id = if let Some(first_instr) = opt_block.instructions.first() {
                first_instr.id
            } else {
                opt_block.terminal.id()
            };

            // Find the innermost scope that contains this block.
            let mut best: Option<(ScopeId, u32)> = None;
            for &(scope_id, range_start, range_end) in &scope_ranges {
                if block_instr_id >= range_start && block_instr_id < range_end {
                    let range_len = range_end.0.saturating_sub(range_start.0);
                    if best.is_none_or(|(_, best_len)| range_len < best_len) {
                        best = Some((scope_id, range_len));
                    }
                }
            }

            if let Some((scope_id, _)) = best {
                let entry = hoistable_by_scope.entry(scope_id).or_default();
                let already_present = entry.iter().any(|existing| {
                    existing.identifier.declaration_id == opt_dep.identifier.declaration_id
                        && super::hir_types::are_equal_paths(&existing.path, &opt_dep.path)
                });
                if !already_present {
                    entry.push(opt_dep.clone());
                }
            }
        }
    }

    let processed_instrs = opt_chain.processed_instrs_in_optional;
    let intermediate_optional_results = opt_chain.intermediate_optional_results;

    // Collect dependencies
    let collect_result = collect_dependencies(
        func,
        &merged_temporaries,
        processed_instrs,
        &intermediate_optional_results,
    )?;

    // Phase 3: Derive minimal dependency set for each scope
    // We need to write dependencies back to the scopes in the terminals.
    let mut scope_deps_map: FxHashMap<ScopeId, Vec<TempReactiveScopeDependency>> =
        FxHashMap::default();
    for (scope, deps) in collect_result.deps {
        if deps.is_empty() {
            continue;
        }
        scope_deps_map.insert(scope.id, deps);
    }

    // Build minimal dependencies and write back to scope terminals.
    // Also apply collected scope declaration and reassignment mutations.
    for block in func.body.blocks.values_mut() {
        let scope_mut = match &mut block.terminal {
            Terminal::Scope(t) => Some(&mut t.scope),
            Terminal::PrunedScope(t) => Some(&mut t.scope),
            _ => None,
        };

        if let Some(scope) = scope_mut {
            // Apply collected scope declaration mutations (Task 5A fix)
            for (target_scope_id, ident_id, ident, declaring_scope) in
                &collect_result.scope_declaration_mutations
            {
                if scope.id == *target_scope_id && !scope.declarations.contains_key(ident_id) {
                    scope.declarations.insert(
                        *ident_id,
                        super::hir_types::ReactiveScopeDeclaration {
                            identifier: ident.clone(),
                            scope: declaring_scope.clone(),
                        },
                    );
                }
            }

            // Apply collected scope reassignment mutations (Task 5A fix)
            for (target_scope_id, ident) in &collect_result.scope_reassignment_mutations {
                if scope.id == *target_scope_id
                    && !scope.reassignments.iter().any(|r| r.declaration_id == ident.declaration_id)
                {
                    scope.reassignments.push(ident.clone());
                }
            }

            // Apply dependencies
            if let Some(deps) = scope_deps_map.get(&scope.id) {
                let hoistable_paths: Vec<ReactiveScopeDependency> =
                    hoistable_by_scope.get(&scope.id).cloned().unwrap_or_default();

                let mut tree = ReactiveScopeDependencyTree::new(hoistable_paths);
                for dep in deps {
                    tree.add_dependency(&dep.to_scope_dependency());
                }

                let candidates = tree.derive_minimal_dependencies();
                for candidate_dep in candidates {
                    let already_exists = scope.dependencies.iter().any(|existing| {
                        existing.identifier.declaration_id
                            == candidate_dep.identifier.declaration_id
                            && super::hir_types::are_equal_paths(
                                &existing.path,
                                &candidate_dep.path,
                            )
                    });
                    if !already_exists {
                        scope.dependencies.insert(candidate_dep);
                    }
                }
            }
        }
    }
    Ok(())
}
