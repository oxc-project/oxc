//! Port of `ReactiveScopes/PruneInitializationDependencies.ts`.
//!
//! This pass is built on the observation by `@jbrown215` that the argument
//! expression passed to `useState` and `useRef` is evaluated only on the first
//! render. On subsequent renders the argument is still evaluated but its result
//! is ignored. We exploit this to drop dependencies that only feed into the
//! arguments of these "create-only" hooks: such dependencies don't actually
//! affect re-renders, so a reactive scope whose only consumer is the initial
//! argument can be re-driven by the cache-sentinel path instead of being
//! invalidated whenever the dependency changes.
//!
//! This pass is conservative — it runs only when
//! `enableChangeDetectionForDebugging` is set, matching the TS reference. The
//! upstream port (TS `PruneInitializationDependencies.ts`) calls out that the
//! pass "isn't yet stress-tested so it's not enabled by default."
//!
//! ## Algorithm
//!
//! 1. **Alias collection** — Walk the reactive function and build a DisjointSet
//!    over identifiers that are aliased through `StoreLocal`/`LoadLocal` and
//!    `StoreContext`/`LoadContext`, plus a per-root path table for
//!    `PropertyLoad`/`PropertyStore` chains. This lets us translate a scope
//!    dependency `<root>.path` back to the temporary identifier that consumers
//!    actually reference.
//!
//! 2. **Reverse traversal** — Walk the reactive function in reverse program
//!    order. Each identifier is tagged with one of `Update`, `Create`, or
//!    `Unknown`:
//!    - terminals propagate `Update` into their operands (any control-flow
//!      sink means the value is consumed on every render);
//!    - calls to `useState` / `useRef` propagate `Create` into their argument
//!      operands (only the callee/receiver stay `Update`);
//!    - other hooks propagate `Update` into their operands;
//!    - regular instructions propagate the join of their lvalues' tags into
//!      their operands.
//!
//! 3. **Dependency pruning** — When leaving a scope, look up each dependency
//!    via the alias/path table. If the corresponding identifier is `Create`
//!    only, drop the dep from the scope: that input only mattered for the
//!    initial useState/useRef argument, so the scope can be re-emitted via
//!    the cache sentinel instead.

use rustc_hash::FxHashMap;

use crate::hir::{
    CallArg, Identifier, IdentifierId, InstructionValue, Place, ReactiveBlock, ReactiveFunction,
    ReactiveInstruction, ReactiveScopeBlock, ReactiveStatement, ReactiveTerminal, ReactiveValue,
    environment::get_hook_kind_for_type,
    types::{PropertyLiteral, Type},
    visitors::each_instruction_value_lvalue,
};
use crate::utils::disjoint_set::DisjointSet;

/// Whether an identifier is consumed on every render (`Update`), only on the
/// initial render through `useState`/`useRef` arguments (`Create`), or has not
/// been observed yet (`Unknown`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreateUpdate {
    Create,
    Update,
    Unknown,
}

impl CreateUpdate {
    fn join(self, other: Self) -> Self {
        match (self, other) {
            (CreateUpdate::Update, _) | (_, CreateUpdate::Update) => CreateUpdate::Update,
            (CreateUpdate::Create, _) | (_, CreateUpdate::Create) => CreateUpdate::Create,
            _ => CreateUpdate::Unknown,
        }
    }
}

/// Run the pass. The environment is needed to resolve hook signatures on
/// function-typed callees (matching the upstream `getHookKind(env, callee)`
/// hook detector).
pub fn prune_initialization_dependencies(
    func: &mut ReactiveFunction,
    env: &crate::hir::environment::Environment,
) {
    let (aliases, paths) = collect_aliases(&func.body);
    let mut visitor = Visitor { map: FxHashMap::default(), aliases, paths, env };
    visitor.visit_block(&mut func.body, CreateUpdate::Update);
}

// =====================================================================================
// Alias / path collection (first pass)
// =====================================================================================

fn collect_aliases(
    body: &ReactiveBlock,
) -> (DisjointSet<IdentifierId>, FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>>)
{
    let mut aliases: DisjointSet<IdentifierId> = DisjointSet::new();
    let mut raw_paths: FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>> =
        FxHashMap::default();
    collect_aliases_in_block(body, &mut aliases, &mut raw_paths);

    // Normalize path keys/values through the alias relation, matching the
    // upstream `getAliases` post-processing step. `find` mutates the disjoint
    // set (path compression), so we collect into a fresh map here.
    let mut paths: FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>> =
        FxHashMap::default();
    for (key, inner) in raw_paths {
        let root_key = aliases.find(&key).unwrap_or(key);
        for (prop, value) in inner {
            let root_value = aliases.find(&value).unwrap_or(value);
            paths.entry(root_key).or_default().insert(prop, root_value);
        }
    }

    (aliases, paths)
}

fn collect_aliases_in_block(
    block: &ReactiveBlock,
    aliases: &mut DisjointSet<IdentifierId>,
    paths: &mut FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>>,
) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                collect_aliases_in_instruction(&instr_stmt.instruction, aliases, paths);
            }
            ReactiveStatement::Terminal(term_stmt) => {
                collect_aliases_in_terminal(&term_stmt.terminal, aliases, paths);
            }
            ReactiveStatement::Scope(scope) => {
                collect_aliases_in_block(&scope.instructions, aliases, paths);
            }
            ReactiveStatement::PrunedScope(scope) => {
                collect_aliases_in_block(&scope.instructions, aliases, paths);
            }
        }
    }
}

fn collect_aliases_in_instruction(
    instr: &ReactiveInstruction,
    aliases: &mut DisjointSet<IdentifierId>,
    paths: &mut FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>>,
) {
    collect_aliases_in_value(&instr.value, instr.lvalue.as_ref(), aliases, paths);
}

fn collect_aliases_in_value(
    value: &ReactiveValue,
    lvalue: Option<&Place>,
    aliases: &mut DisjointSet<IdentifierId>,
    paths: &mut FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>>,
) {
    match value {
        ReactiveValue::Instruction(boxed) => match boxed.as_ref() {
            InstructionValue::StoreLocal(v) => {
                aliases.union(&[v.lvalue.place.identifier.id, v.value.identifier.id]);
            }
            InstructionValue::StoreContext(v) => {
                aliases.union(&[v.lvalue_place.identifier.id, v.value.identifier.id]);
            }
            InstructionValue::LoadLocal(v) => {
                if let Some(lv) = lvalue {
                    aliases.union(&[lv.identifier.id, v.place.identifier.id]);
                }
            }
            InstructionValue::LoadContext(v) => {
                if let Some(lv) = lvalue {
                    aliases.union(&[lv.identifier.id, v.place.identifier.id]);
                }
            }
            InstructionValue::PropertyLoad(v) => {
                if let Some(lv) = lvalue {
                    paths
                        .entry(v.object.identifier.id)
                        .or_default()
                        .insert(v.property.clone(), lv.identifier.id);
                }
            }
            InstructionValue::PropertyStore(v) => {
                paths
                    .entry(v.object.identifier.id)
                    .or_default()
                    .insert(v.property.clone(), v.value.identifier.id);
            }
            _ => {}
        },
        ReactiveValue::Logical(v) => {
            collect_aliases_in_value(&v.left, None, aliases, paths);
            collect_aliases_in_value(&v.right, None, aliases, paths);
        }
        ReactiveValue::Ternary(v) => {
            collect_aliases_in_value(&v.test, None, aliases, paths);
            collect_aliases_in_value(&v.consequent, None, aliases, paths);
            collect_aliases_in_value(&v.alternate, None, aliases, paths);
        }
        ReactiveValue::Sequence(v) => {
            for instr in &v.instructions {
                collect_aliases_in_instruction(instr, aliases, paths);
            }
            collect_aliases_in_value(&v.value, None, aliases, paths);
        }
        ReactiveValue::OptionalCall(v) => {
            collect_aliases_in_value(&v.value, None, aliases, paths);
        }
    }
}

fn collect_aliases_in_terminal(
    terminal: &ReactiveTerminal,
    aliases: &mut DisjointSet<IdentifierId>,
    paths: &mut FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>>,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            collect_aliases_in_block(&t.consequent, aliases, paths);
            if let Some(alt) = &t.alternate {
                collect_aliases_in_block(alt, aliases, paths);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    collect_aliases_in_block(block, aliases, paths);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            collect_aliases_in_value(&t.test, None, aliases, paths);
            collect_aliases_in_block(&t.r#loop, aliases, paths);
        }
        ReactiveTerminal::DoWhile(t) => {
            collect_aliases_in_block(&t.r#loop, aliases, paths);
            collect_aliases_in_value(&t.test, None, aliases, paths);
        }
        ReactiveTerminal::For(t) => {
            collect_aliases_in_value(&t.init, None, aliases, paths);
            collect_aliases_in_value(&t.test, None, aliases, paths);
            collect_aliases_in_block(&t.r#loop, aliases, paths);
            if let Some(update) = &t.update {
                collect_aliases_in_value(update, None, aliases, paths);
            }
        }
        ReactiveTerminal::ForOf(t) => {
            collect_aliases_in_value(&t.init, None, aliases, paths);
            collect_aliases_in_value(&t.test, None, aliases, paths);
            collect_aliases_in_block(&t.r#loop, aliases, paths);
        }
        ReactiveTerminal::ForIn(t) => {
            collect_aliases_in_value(&t.init, None, aliases, paths);
            collect_aliases_in_block(&t.r#loop, aliases, paths);
        }
        ReactiveTerminal::Label(t) => collect_aliases_in_block(&t.block, aliases, paths),
        ReactiveTerminal::Try(t) => {
            collect_aliases_in_block(&t.block, aliases, paths);
            collect_aliases_in_block(&t.handler, aliases, paths);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

// =====================================================================================
// Reverse traversal (second pass)
// =====================================================================================

struct Visitor<'env> {
    map: FxHashMap<IdentifierId, CreateUpdate>,
    aliases: DisjointSet<IdentifierId>,
    paths: FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>>,
    env: &'env crate::hir::environment::Environment,
}

impl Visitor<'_> {
    fn lookup(&self, id: IdentifierId) -> CreateUpdate {
        self.map.get(&id).copied().unwrap_or(CreateUpdate::Unknown)
    }

    fn mark_place(&mut self, place: &Place, state: CreateUpdate) {
        let id = place.identifier.id;
        let combined = state.join(self.lookup(id));
        self.map.insert(id, combined);
    }

    fn visit_block(&mut self, block: &mut ReactiveBlock, state: CreateUpdate) {
        // The TS reference walks the block in reverse so that consumers (which
        // appear later in program order) are tagged before producers.
        for stmt in block.iter_mut().rev() {
            self.visit_statement(stmt, state);
        }
    }

    fn visit_statement(&mut self, stmt: &mut ReactiveStatement, state: CreateUpdate) {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                self.visit_instruction(&instr_stmt.instruction, state);
            }
            ReactiveStatement::Terminal(term_stmt) => {
                self.visit_terminal(&mut term_stmt.terminal, state);
            }
            ReactiveStatement::Scope(scope) => {
                self.visit_scope(scope, state);
            }
            ReactiveStatement::PrunedScope(scope) => {
                self.visit_block(&mut scope.instructions, state);
            }
        }
    }

    fn visit_scope(&mut self, scope_block: &mut ReactiveScopeBlock, _outer: CreateUpdate) {
        // Start with the join of the scope's declaration / reassignment tags
        // (mirrors the TS `visitScope` reduction over `declarations` +
        // `reassignments`).
        let mut state = CreateUpdate::Unknown;
        for &decl_id in scope_block.scope.declarations.keys() {
            state = state.join(self.lookup(decl_id));
        }
        for reassign in &scope_block.scope.reassignments {
            state = state.join(self.lookup(reassign.id));
        }

        self.visit_block(&mut scope_block.instructions, state);

        // After the scope body has been traversed, drop dependencies whose
        // resolved identifier is `Create` only.
        // Snapshot the dependencies so we can borrow `self.aliases` mutably
        // inside `resolve_dep_state` (which performs path compression).
        let deps: Vec<_> = scope_block.scope.dependencies.iter().cloned().collect();
        let mut to_keep = rustc_hash::FxHashSet::default();
        for dep in deps {
            let resolved = resolve_dep_state(
                &self.map,
                &mut self.aliases,
                &self.paths,
                dep.identifier.id,
                &dep.path,
            );
            if !matches!(resolved, Some(CreateUpdate::Create)) {
                to_keep.insert(dep);
            }
        }
        scope_block.scope.dependencies = to_keep;
    }

    fn visit_instruction(&mut self, instr: &ReactiveInstruction, outer: CreateUpdate) {
        // Aggregate the state of this instruction's lvalues. For ReactiveInstruction
        // that's the outer `lvalue` plus any inner-value lvalues (StoreLocal etc.).
        let mut state = CreateUpdate::Unknown;
        if let Some(lv) = &instr.lvalue {
            state = state.join(self.lookup(lv.identifier.id));
        }
        if let ReactiveValue::Instruction(boxed) = &instr.value {
            for lvalue in each_instruction_value_lvalue(boxed) {
                state = state.join(self.lookup(lvalue.identifier.id));
            }
        }
        // Conservative fallback: if no lvalue is observed, defer to the outer
        // state (mirrors the TS join over an empty lvalue set, which produces
        // `Unknown`, and then the implicit `Update` propagation from sequence
        // / terminal contexts).
        if matches!(state, CreateUpdate::Unknown) {
            state = outer;
        }

        self.visit_value(&instr.value, instr.lvalue.as_ref(), state);
    }

    fn visit_value(&mut self, value: &ReactiveValue, lvalue: Option<&Place>, state: CreateUpdate) {
        match value {
            ReactiveValue::Instruction(boxed) => {
                self.visit_instruction_value(boxed, lvalue, state);
            }
            ReactiveValue::Logical(v) => {
                self.visit_value(&v.left, None, state);
                self.visit_value(&v.right, None, state);
            }
            ReactiveValue::Ternary(v) => {
                self.visit_value(&v.test, None, state);
                self.visit_value(&v.consequent, None, state);
                self.visit_value(&v.alternate, None, state);
            }
            ReactiveValue::OptionalCall(v) => {
                self.visit_value(&v.value, None, state);
            }
            ReactiveValue::Sequence(seq) => {
                // Reverse-order matches the TS reverse traversal over a block.
                self.visit_value(&seq.value, None, state);
                for instr in seq.instructions.iter().rev() {
                    self.visit_instruction(instr, state);
                }
            }
        }
    }

    fn visit_instruction_value(
        &mut self,
        value: &InstructionValue,
        lvalue: Option<&Place>,
        state: CreateUpdate,
    ) {
        let is_create_only = lvalue.is_some_and(|lv| is_create_only_lvalue(&lv.identifier));
        match value {
            InstructionValue::CallExpression(call) => {
                if is_create_only {
                    // useState/useRef: callee stays at the surrounding `state`
                    // (so the hook itself isn't tagged Create), and arguments
                    // propagate `Create` — those are the values whose updates
                    // we want to drop.
                    self.mark_place(&call.callee, state);
                    for arg in &call.args {
                        self.mark_call_arg(arg, CreateUpdate::Create);
                    }
                } else {
                    let inner = if self.is_hook_call(value) { CreateUpdate::Update } else { state };
                    self.mark_place(&call.callee, inner);
                    for arg in &call.args {
                        self.mark_call_arg(arg, inner);
                    }
                }
            }
            InstructionValue::MethodCall(method) => {
                if is_create_only {
                    self.mark_place(&method.property, state);
                    self.mark_place(&method.receiver, state);
                    for arg in &method.args {
                        self.mark_call_arg(arg, CreateUpdate::Create);
                    }
                } else {
                    let inner = if self.is_hook_call(value) { CreateUpdate::Update } else { state };
                    self.mark_place(&method.property, inner);
                    self.mark_place(&method.receiver, inner);
                    for arg in &method.args {
                        self.mark_call_arg(arg, inner);
                    }
                }
            }
            _ => {
                for place in crate::hir::visitors::each_instruction_value_operand(value) {
                    self.mark_place(place, state);
                }
            }
        }
    }

    fn mark_call_arg(&mut self, arg: &CallArg, state: CreateUpdate) {
        match arg {
            CallArg::Place(p) => self.mark_place(p, state),
            CallArg::Spread(s) => self.mark_place(&s.place, state),
        }
    }

    fn visit_terminal(&mut self, terminal: &mut ReactiveTerminal, outer: CreateUpdate) {
        match terminal {
            ReactiveTerminal::Break(_) | ReactiveTerminal::Continue(_) => {}
            ReactiveTerminal::Return(t) => {
                self.mark_place(&t.value, CreateUpdate::Update);
            }
            ReactiveTerminal::Throw(t) => {
                self.mark_place(&t.value, CreateUpdate::Update);
            }
            ReactiveTerminal::For(t) => {
                self.visit_value(&t.init, None, CreateUpdate::Update);
                self.visit_value(&t.test, None, CreateUpdate::Update);
                self.visit_block(&mut t.r#loop, outer);
                if let Some(update) = &t.update {
                    self.visit_value(update, None, CreateUpdate::Update);
                }
            }
            ReactiveTerminal::ForOf(t) => {
                self.visit_value(&t.init, None, CreateUpdate::Update);
                self.visit_value(&t.test, None, CreateUpdate::Update);
                self.visit_block(&mut t.r#loop, outer);
            }
            ReactiveTerminal::ForIn(t) => {
                self.visit_value(&t.init, None, CreateUpdate::Update);
                self.visit_block(&mut t.r#loop, outer);
            }
            ReactiveTerminal::DoWhile(t) => {
                self.visit_block(&mut t.r#loop, outer);
                self.visit_value(&t.test, None, CreateUpdate::Update);
            }
            ReactiveTerminal::While(t) => {
                self.visit_value(&t.test, None, CreateUpdate::Update);
                self.visit_block(&mut t.r#loop, outer);
            }
            ReactiveTerminal::If(t) => {
                self.mark_place(&t.test, CreateUpdate::Update);
                self.visit_block(&mut t.consequent, outer);
                if let Some(alt) = &mut t.alternate {
                    self.visit_block(alt, outer);
                }
            }
            ReactiveTerminal::Switch(t) => {
                self.mark_place(&t.test, CreateUpdate::Update);
                for case in &mut t.cases {
                    if let Some(test) = &case.test {
                        self.mark_place(test, CreateUpdate::Update);
                    }
                    if let Some(block) = case.block.as_mut() {
                        self.visit_block(block, outer);
                    }
                }
            }
            ReactiveTerminal::Label(t) => {
                self.visit_block(&mut t.block, outer);
            }
            ReactiveTerminal::Try(t) => {
                self.visit_block(&mut t.block, outer);
                self.visit_block(&mut t.handler, outer);
            }
        }
    }

    fn is_hook_call(&self, value: &InstructionValue) -> bool {
        match value {
            InstructionValue::CallExpression(call) => {
                get_hook_kind_for_type(self.env, &call.callee.identifier.type_).is_some()
            }
            InstructionValue::MethodCall(method) => {
                get_hook_kind_for_type(self.env, &method.property.identifier.type_).is_some()
            }
            _ => false,
        }
    }
}

/// Resolve the abstract state of a scope dependency `(root, path)` through the
/// alias / path index. Returns `None` if the chain can't be followed all the
/// way (mirroring the TS `target &&=` short-circuit).
fn resolve_dep_state(
    map: &FxHashMap<IdentifierId, CreateUpdate>,
    aliases: &mut DisjointSet<IdentifierId>,
    paths: &FxHashMap<IdentifierId, FxHashMap<PropertyLiteral, IdentifierId>>,
    root_id: IdentifierId,
    path: &[crate::hir::DependencyPathEntry],
) -> Option<CreateUpdate> {
    let mut target = aliases.find(&root_id).unwrap_or(root_id);
    for entry in path {
        let inner = paths.get(&target)?;
        let next = *inner.get(&entry.property)?;
        target = aliases.find(&next).unwrap_or(next);
    }
    Some(map.get(&target).copied().unwrap_or(CreateUpdate::Unknown))
}

/// Whether the instruction's lvalue carries the return-value type of a
/// create-only hook (`useState` or `useRef`). The upstream pass spells these
/// checks as `isUseStateType` / `isUseRefType`, both of which inspect the
/// `Object` shape on the identifier.
fn is_create_only_lvalue(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(o)
            if matches!(
                o.shape_id.as_deref(),
                Some(
                    crate::hir::object_shape::BUILT_IN_USE_STATE_ID
                        | crate::hir::object_shape::BUILT_IN_USE_REF_ID,
                ),
            )
    )
}
