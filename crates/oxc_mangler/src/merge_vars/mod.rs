//! Let `var`s and parameters of one function share a mangled name when their lifetimes never
//! overlap, like UglifyJS's `merge_vars`.
//!
//! For every function (and the top level, when it is mangled), [`flow::Walker`] builds a
//! control-flow graph of the reads and writes of the candidate variables, [`liveness`] turns it
//! into live ranges, and variables whose ranges are disjoint are grouped so that each group can
//! take one slot.

mod flow;
mod liveness;

use oxc_allocator::{Allocator, ArenaVec, BitSet};
use oxc_ast::{
    AstKind,
    ast::{BindingPattern, FormalParameters, Program},
};
use oxc_semantic::{AstNodes, Scoping};
use oxc_str::Ident;
use oxc_syntax::{
    node::NodeId,
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use crate::Constraints;
use flow::{FlowBuffers, Root, Walker};
use liveness::{Liveness, Range};

const NO_LOCAL: u32 = u32::MAX;

/// How many of the earliest groups a variable is tested against before it starts a new group.
const MAX_GROUP_SCAN: usize = 32;

/// Variables whose declarations became redundant because they now share a name with another
/// variable of the same function.
///
/// Returned by [`crate::Mangler::build_with_semantic_and_merged_vars`].
#[derive(Debug, Default)]
pub struct MergedVars {
    redundant: Vec<SymbolId>,
}

impl MergedVars {
    /// `true` if no variables were merged.
    pub fn is_empty(&self) -> bool {
        self.redundant.is_empty()
    }

    /// Symbols, in ascending order, whose `var` declarations are redundant.
    ///
    /// Each one shares its mangled name with a parameter or `var` of the same function that keeps
    /// its declaration, so a `var` declaring only redundant symbols can be dropped or turned into
    /// an assignment.
    pub fn redundant_symbols(&self) -> &[SymbolId] {
        &self.redundant
    }
}

/// The result of merging: which symbol stands in for each merged symbol during slot assignment.
pub struct Merges<'t> {
    /// `representatives[symbol]` is the smallest symbol of its group. `None` if nothing merged.
    representatives: Option<ArenaVec<'t, SymbolId>>,
    merged_vars: MergedVars,
}

impl Merges<'_> {
    pub fn none() -> Self {
        Self { representatives: None, merged_vars: MergedVars::default() }
    }

    pub fn is_empty(&self) -> bool {
        self.representatives.is_none()
    }

    #[inline]
    pub fn representative(&self, symbol_id: SymbolId) -> SymbolId {
        self.representatives
            .as_ref()
            .map_or(symbol_id, |representatives| representatives[symbol_id.index()])
    }

    pub fn into_merged_vars(self) -> MergedVars {
        self.merged_vars
    }
}

/// A candidate variable of the scope being analyzed, indexed by its local index.
#[derive(Clone, Copy)]
struct Local {
    symbol_id: SymbolId,
    is_param: bool,
    /// Seen in a position the walker does not model, so it must keep its own slot.
    revoked: bool,
    /// Has a `var x;` declarator, which the minifier can drop when the symbol is redundant.
    bare: bool,
    references: u32,
    declarations: u32,
}

/// Compute which candidate variables share a slot.
pub fn compute<'t>(
    allocator: &'t Allocator,
    scoping: &Scoping,
    ast_nodes: &AstNodes<'_>,
    program: &Program<'_>,
    constraints: &Constraints,
) -> Merges<'t> {
    let mut merges = Merges::none();
    // The transformer strips TypeScript before minification, so TS-only syntax is not modeled.
    if program.source_type.is_typescript() {
        return merges;
    }
    let mut analyzer: Option<Analyzer<'_, 't>> = None;
    let mut arguments_scopes: Option<BitSet<'t>> = None;
    let root_scope_id = scoping.root_scope_id();

    for (scope_id, bindings) in scoping.iter_bindings() {
        if bindings.len() < 2 {
            continue;
        }
        let flags = scoping.scope_flags(scope_id);
        if flags.contains_direct_eval() {
            continue;
        }
        if scope_id == root_scope_id {
            if !constraints.top_level {
                continue;
            }
        } else if !flags.is_function() && !flags.is_class_static_block() {
            continue;
        }
        if !bindings.values().any(|&symbol_id| is_var(scoping, ast_nodes, symbol_id)) {
            continue;
        }
        let root = match ast_nodes.kind(scoping.get_node_id(scope_id)) {
            AstKind::Program(program) => Root::Program(program),
            AstKind::Function(function) => match &function.body {
                Some(body) => Root::Function(&function.params, body),
                None => continue,
            },
            AstKind::ArrowFunctionExpression(arrow) => Root::Arrow(arrow),
            AstKind::StaticBlock(block) => Root::StaticBlock(block),
            _ => continue,
        };

        let params_mergeable = root.params().is_some_and(is_simple_parameter_list)
            && (flags.is_strict_mode()
                || flags.is_arrow()
                || !uses_arguments(
                    arguments_scopes
                        .get_or_insert_with(|| collect_arguments_scopes(allocator, scoping)),
                    scoping,
                    scope_id,
                ));

        let analyzer = analyzer.get_or_insert_with(|| Analyzer::new(allocator, scoping, ast_nodes));
        analyzer.locals.clear();
        analyzer.reference_nodes.clear();
        let mut has_var = false;
        for &symbol_id in bindings.values() {
            if let Some(is_param) = candidate_kind(
                scoping,
                ast_nodes,
                constraints,
                &analyzer.homes,
                scope_id,
                symbol_id,
                params_mergeable,
                &mut analyzer.reference_nodes,
            ) {
                has_var |= !is_param;
                analyzer.locals.push(Local {
                    symbol_id,
                    is_param,
                    revoked: false,
                    bare: false,
                    references: 0,
                    declarations: 0,
                });
            }
        }
        if analyzer.locals.len() < 2 || !has_var {
            continue;
        }
        analyzer.analyze(scoping, ast_nodes, scope_id, root, &mut merges);
    }

    merges.merged_vars.redundant.sort_unstable();
    merges
}

/// Reusable buffers for analyzing one scope at a time.
struct Analyzer<'a, 't> {
    allocator: &'t Allocator,
    /// `local_index[symbol]` is the symbol's index in `locals`, or [`NO_LOCAL`].
    local_index: ArenaVec<'t, u32>,
    locals: ArenaVec<'t, Local>,
    /// `homes[scope]` is the nearest var scope enclosing or equal to `scope`, or `None` if a
    /// `with` comes first.
    homes: ArenaVec<'t, Option<ScopeId>>,
    /// Nodes of the references to `locals`.
    reference_nodes: ArenaVec<'t, NodeId>,
    /// AST nodes that contain a reference to a candidate of their scope. Each reference stays
    /// in its scope, below any other function or static block, so the nodes marked for different
    /// scopes are disjoint and never need unmarking.
    marked: BitSet<'t>,
    flow: FlowBuffers<'a, 't>,
    liveness: Liveness<'t>,
    order: ArenaVec<'t, u32>,
    groups: ArenaVec<'t, Group>,
    next_member: ArenaVec<'t, u32>,
    /// Min-heap of `(end, group)`. Entries whose end no longer matches the group are stale.
    heap: ArenaVec<'t, (u32, u32)>,
}

#[derive(Clone, Copy)]
struct Group {
    /// Last position covered by any member.
    end: u32,
    /// First member; the rest are linked through `next_member`.
    head: u32,
    len: u32,
}

impl<'a, 't> Analyzer<'a, 't> {
    fn new(allocator: &'t Allocator, scoping: &Scoping, ast_nodes: &AstNodes<'_>) -> Self {
        Self {
            allocator,
            local_index: ArenaVec::from_iter_in(
                std::iter::repeat_n(NO_LOCAL, scoping.symbols_len()),
                &allocator,
            ),
            locals: ArenaVec::new_in(&allocator),
            homes: collect_homes(allocator, scoping),
            reference_nodes: ArenaVec::new_in(&allocator),
            marked: BitSet::new_in(ast_nodes.len(), allocator),
            flow: FlowBuffers::new_in(allocator),
            liveness: Liveness::new_in(allocator),
            order: ArenaVec::new_in(&allocator),
            groups: ArenaVec::new_in(&allocator),
            next_member: ArenaVec::new_in(&allocator),
            heap: ArenaVec::new_in(&allocator),
        }
    }

    fn analyze(
        &mut self,
        scoping: &Scoping,
        ast_nodes: &AstNodes<'_>,
        scope_id: ScopeId,
        root: Root<'a>,
        merges: &mut Merges<'t>,
    ) {
        self.locals.sort_unstable_by_key(|local| local.symbol_id);
        for (index, local) in self.locals.iter().enumerate() {
            #[expect(clippy::cast_possible_truncation, reason = "bounded by the symbol count")]
            let index = index as u32;
            self.local_index[local.symbol_id.index()] = index;
        }
        let scope_node_id = scoping.get_node_id(scope_id);
        for &reference_node in &self.reference_nodes {
            let mut node_id = reference_node;
            while node_id != scope_node_id && !self.marked.has_bit(node_id.index()) {
                self.marked.set_bit(node_id.index());
                node_id = ast_nodes.parent_id(node_id);
            }
        }

        let analyzed =
            Walker::new(scoping, &self.local_index, &self.marked, &mut self.locals, &mut self.flow)
                .walk(scope_id, root)
                && self.revoke_unvisited(scoping)
                && self.liveness.compute(&self.flow, self.locals.len());
        if analyzed {
            self.group();
            self.emit(scoping, merges);
        }

        for local in &self.locals {
            self.local_index[local.symbol_id.index()] = NO_LOCAL;
        }
    }

    /// Revoke candidates with a reference or declaration the walker did not reach, e.g. one in a
    /// class body. Returns `false` if fewer than two candidates, or no `var`, remain.
    fn revoke_unvisited(&mut self, scoping: &Scoping) -> bool {
        let mut remaining = 0;
        let mut has_var = false;
        for local in &mut self.locals {
            let references = scoping.get_resolved_reference_ids(local.symbol_id).len();
            // The list of redeclarations includes the first declaration.
            let declarations = scoping.symbol_redeclarations(local.symbol_id).len().max(1);
            if local.references as usize != references
                || local.declarations as usize != declarations
            {
                local.revoked = true;
            }
            if !local.revoked {
                remaining += 1;
                has_var |= !local.is_param;
            }
        }
        remaining >= 2 && has_var
    }

    /// Greedily partition the candidates into groups with pairwise disjoint live ranges.
    fn group(&mut self) {
        let Self { locals, liveness, order, groups, next_member, heap, .. } = self;
        order.clear();
        order.extend(
            (0..locals.len())
                .filter(|&index| !locals[index].revoked)
                .map(|index| u32::try_from(index).unwrap_or(NO_LOCAL)),
        );
        order.sort_unstable_by_key(|&local| {
            let first = liveness.ranges(local).first().map_or(u32::MAX, |range| range.start);
            (first, locals[local as usize].symbol_id)
        });
        groups.clear();
        heap.clear();
        next_member.clear();
        next_member.resize(locals.len(), NO_LOCAL);

        for &local in order.iter() {
            let ranges = liveness.ranges(local);
            let target = if locals[local as usize].is_param {
                None
            } else if let (Some(first), Some(last)) = (ranges.first(), ranges.last()) {
                find_group(groups, heap, next_member, liveness, ranges, first.start)
                    .map(|group| (group, last.end))
            } else {
                // Never read or written, so it fits in any group.
                (!groups.is_empty()).then_some((0, 0))
            };
            if let Some((group_index, end)) = target {
                let group = &mut groups[group_index as usize];
                next_member[local as usize] = group.head;
                group.head = local;
                group.len += 1;
                if end > group.end {
                    group.end = end;
                    heap_push(heap, (end, group_index));
                }
            } else {
                let end = ranges.last().map_or(0, |range| range.end);
                let group_index = u32::try_from(groups.len()).unwrap_or(NO_LOCAL);
                groups.push(Group { end, head: local, len: 1 });
                heap_push(heap, (end, group_index));
            }
        }
    }

    /// Record the groups with more than one member.
    fn emit(&self, scoping: &Scoping, merges: &mut Merges<'t>) {
        for group in &self.groups {
            if group.len < 2 {
                continue;
            }
            let mut representative = self.locals[group.head as usize].symbol_id;
            let mut keeper: Option<Local> = None;
            let mut member = group.head;
            while member != NO_LOCAL {
                let local = self.locals[member as usize];
                representative = representative.min(local.symbol_id);
                keeper = Some(match keeper {
                    None => local,
                    Some(keeper) => {
                        if keeper_rank(local) < keeper_rank(keeper) {
                            local
                        } else {
                            keeper
                        }
                    }
                });
                member = self.next_member[member as usize];
            }
            let keeper = keeper.map(|keeper| keeper.symbol_id);
            debug_assert!(self.members_are_disjoint(group));

            let representatives = merges.representatives.get_or_insert_with(|| {
                ArenaVec::from_iter_in(
                    (0..scoping.symbols_len()).map(SymbolId::from_usize),
                    &self.allocator,
                )
            });
            let mut member = group.head;
            while member != NO_LOCAL {
                let symbol_id = self.locals[member as usize].symbol_id;
                representatives[symbol_id.index()] = representative;
                if Some(symbol_id) != keeper {
                    merges.merged_vars.redundant.push(symbol_id);
                }
                member = self.next_member[member as usize];
            }
        }
    }

    fn members_are_disjoint(&self, group: &Group) -> bool {
        let mut first = group.head;
        while first != NO_LOCAL {
            let mut second = self.next_member[first as usize];
            while second != NO_LOCAL {
                if ranges_intersect(self.liveness.ranges(first), self.liveness.ranges(second)) {
                    return false;
                }
                second = self.next_member[second as usize];
            }
            first = self.next_member[first as usize];
        }
        true
    }
}

/// Lower is a better keeper: the parameter if any, then a `var` without a droppable `var x;`,
/// then the smallest symbol.
fn keeper_rank(local: Local) -> (bool, bool, SymbolId) {
    (!local.is_param, local.bare, local.symbol_id)
}

fn find_group(
    groups: &[Group],
    heap: &mut ArenaVec<'_, (u32, u32)>,
    next_member: &[u32],
    liveness: &Liveness<'_>,
    ranges: &[Range],
    start: u32,
) -> Option<u32> {
    while let Some(&(end, group)) = heap.first() {
        if groups[group as usize].end != end {
            heap_pop(heap);
            continue;
        }
        if end < start {
            return Some(group);
        }
        break;
    }
    (0..groups.len().min(MAX_GROUP_SCAN))
        .find(|&group| {
            let mut member = groups[group].head;
            while member != NO_LOCAL {
                if ranges_intersect(ranges, liveness.ranges(member)) {
                    return false;
                }
                member = next_member[member as usize];
            }
            true
        })
        .map(|group| u32::try_from(group).unwrap_or(NO_LOCAL))
}

fn ranges_intersect(left: &[Range], right: &[Range]) -> bool {
    let (mut left_index, mut right_index) = (0, 0);
    while let (Some(left_range), Some(right_range)) = (left.get(left_index), right.get(right_index))
    {
        if left_range.end < right_range.start {
            left_index += 1;
        } else if right_range.end < left_range.start {
            right_index += 1;
        } else {
            return true;
        }
    }
    false
}

fn heap_push(heap: &mut ArenaVec<'_, (u32, u32)>, entry: (u32, u32)) {
    heap.push(entry);
    let mut child = heap.len() - 1;
    while child > 0 {
        let parent = (child - 1) / 2;
        if heap[parent] <= heap[child] {
            break;
        }
        heap.swap(parent, child);
        child = parent;
    }
}

fn heap_pop(heap: &mut ArenaVec<'_, (u32, u32)>) {
    let Some(last) = heap.pop() else { return };
    if heap.is_empty() {
        return;
    }
    heap[0] = last;
    let mut parent = 0;
    loop {
        let left = parent * 2 + 1;
        if left >= heap.len() {
            break;
        }
        let right = left + 1;
        let child = if right < heap.len() && heap[right] < heap[left] { right } else { left };
        if heap[parent] <= heap[child] {
            break;
        }
        heap.swap(parent, child);
        parent = child;
    }
}

/// Whether `symbol_id`, bound in `scope_id`, may be merged: `Some(true)` for a parameter,
/// `Some(false)` for a `var`.
fn candidate_kind(
    scoping: &Scoping,
    ast_nodes: &AstNodes<'_>,
    constraints: &Constraints,
    homes: &[Option<ScopeId>],
    scope_id: ScopeId,
    symbol_id: SymbolId,
    params_mergeable: bool,
    reference_nodes: &mut ArenaVec<'_, NodeId>,
) -> Option<bool> {
    if scoping.symbol_flags(symbol_id) != SymbolFlags::FunctionScopedVariable
        || !constraints.is_mangle_candidate(symbol_id, scoping)
    {
        return None;
    }
    let declaration = scoping.symbol_declaration(symbol_id);
    let is_param = match ast_nodes.kind(declaration) {
        AstKind::VariableDeclarator(_) => false,
        AstKind::FormalParameter(_) if params_mergeable => true,
        _ => return None,
    };
    let name = scoping.symbol_ident(symbol_id);
    let reaches =
        |from: ScopeId| from == scope_id || reaches_scope(scoping, scope_id, symbol_id, name, from);

    let redeclarations = scoping.symbol_redeclarations(symbol_id);
    if redeclarations.is_empty() {
        if !reaches(ast_nodes.get_node(declaration).scope_id()) {
            return None;
        }
    } else {
        let mut params = 0;
        for redeclaration in redeclarations {
            match ast_nodes.kind(redeclaration.declaration) {
                AstKind::VariableDeclarator(_) => {}
                AstKind::FormalParameter(_) => params += 1,
                _ => return None,
            }
            if !reaches(ast_nodes.get_node(redeclaration.declaration).scope_id()) {
                return None;
            }
        }
        // Duplicate parameters: `function f(a, a) {}`.
        if params > 1 {
            return None;
        }
    }
    let reference_nodes_len = reference_nodes.len();
    for reference in scoping.get_resolved_references(symbol_id) {
        if homes[reference.scope_id().index()] != Some(scope_id) {
            reference_nodes.truncate(reference_nodes_len);
            return None;
        }
        reference_nodes.push(reference.node_id());
    }
    Some(is_param)
}

fn is_var(scoping: &Scoping, ast_nodes: &AstNodes<'_>, symbol_id: SymbolId) -> bool {
    scoping.symbol_flags(symbol_id) == SymbolFlags::FunctionScopedVariable
        && matches!(
            ast_nodes.kind(scoping.symbol_declaration(symbol_id)),
            AstKind::VariableDeclarator(_)
        )
}

/// Whether `from` reaches `scope_id` without leaving the function, entering `with`, or passing a
/// scope that binds `name` to another symbol.
fn reaches_scope(
    scoping: &Scoping,
    scope_id: ScopeId,
    symbol_id: SymbolId,
    name: Ident<'_>,
    from: ScopeId,
) -> bool {
    let mut current = from;
    while current != scope_id {
        if scoping.get_binding(current, name).is_some_and(|binding| binding != symbol_id) {
            return false;
        }
        let Some(parent) = scoping.scope_parent_id(current) else { return false };
        current = parent;
    }
    stays_in_scope(scoping, scope_id, from)
}

fn collect_homes<'t>(allocator: &'t Allocator, scoping: &Scoping) -> ArenaVec<'t, Option<ScopeId>> {
    let mut homes = ArenaVec::with_capacity_in(scoping.scopes_len(), &allocator);
    for index in 0..scoping.scopes_len() {
        let scope_id = ScopeId::from_usize(index);
        let flags = scoping.scope_flags(scope_id);
        let home = if flags.is_var() {
            Some(scope_id)
        } else if flags.contains(ScopeFlags::With) {
            None
        } else {
            scoping.scope_parent_id(scope_id).and_then(|parent| {
                if parent.index() < index {
                    homes[parent.index()]
                } else {
                    find_home(scoping, parent)
                }
            })
        };
        homes.push(home);
    }
    homes
}

fn find_home(scoping: &Scoping, from: ScopeId) -> Option<ScopeId> {
    for scope_id in scoping.scope_ancestors(from) {
        let flags = scoping.scope_flags(scope_id);
        if flags.is_var() {
            return Some(scope_id);
        }
        if flags.contains(ScopeFlags::With) {
            break;
        }
    }
    None
}

/// Whether `from` reaches `scope_id` without leaving the function or entering `with`.
///
/// Enough for a resolved reference, which no other binding of its name can shadow.
fn stays_in_scope(scoping: &Scoping, scope_id: ScopeId, from: ScopeId) -> bool {
    let mut current = from;
    while current != scope_id {
        let flags = scoping.scope_flags(current);
        if flags.is_var() || flags.contains(ScopeFlags::With) {
            return false;
        }
        let Some(parent) = scoping.scope_parent_id(current) else { return false };
        current = parent;
    }
    true
}

fn is_simple_parameter_list(params: &FormalParameters<'_>) -> bool {
    params.rest.is_none()
        && params.items.iter().all(|param| {
            param.initializer.is_none()
                && matches!(param.pattern, BindingPattern::BindingIdentifier(_))
        })
}

/// In sloppy mode, `arguments[i]` aliases parameter `i`, so parameters of a function that uses
/// `arguments` must keep their own names.
fn uses_arguments(arguments_scopes: &BitSet<'_>, scoping: &Scoping, scope_id: ScopeId) -> bool {
    arguments_scopes.has_bit(scope_id.index())
        || scoping.get_binding(scope_id, Ident::from("arguments")).is_some()
}

/// The non-arrow functions whose own `arguments` object is referenced.
fn collect_arguments_scopes<'t>(allocator: &'t Allocator, scoping: &Scoping) -> BitSet<'t> {
    let mut scopes = BitSet::new_in(scoping.scopes_len(), allocator);
    if let Some(reference_ids) = scoping.root_unresolved_references().get(&Ident::from("arguments"))
    {
        for &reference_id in reference_ids {
            let reference_scope_id = scoping.get_reference(reference_id).scope_id();
            if let Some(function_scope_id) =
                scoping.scope_ancestors(reference_scope_id).find(|&id| {
                    let flags = scoping.scope_flags(id);
                    flags.is_function() && !flags.is_arrow()
                })
            {
                scopes.set_bit(function_scope_id.index());
            }
        }
    }
    scopes
}
