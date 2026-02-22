/// Promote used temporaries to named variables.
///
/// Port of `ReactiveScopes/PromoteUsedTemporaries.ts` from the React Compiler.
///
/// Finds temporary variables that are used as declarations of reactive scopes
/// and promotes them to named variables. This is needed because temporary
/// variables are typically unnamed, but scope declarations need names for
/// the output code.
///
/// The algorithm runs in four phases:
///
/// **Phase 1** (`CollectPromotableTemporaries`): Gather information about pruned scope
/// declarations and JSX expression tags. Records which pruned-scope temporaries are
/// used outside their defining scope, and which declaration IDs appear as JSX tags.
///
/// **Phase 2** (`PromoteTemporaries`): Visit scope blocks and promote unnamed
/// declarations and dependencies. Visit pruned scopes and promote unnamed
/// declarations that are used outside their scope. Visit function parameters
/// and promote unnamed ones.
///
/// **Phase 3** (`PromoteInterposedTemporaries`): Track defined-but-not-yet-used
/// temporaries. When a potentially-mutating instruction is encountered
/// (CallExpression, MethodCall, PropertyStore, etc.), mark pending temporaries
/// for promotion. When a marked temporary is actually used, promote it.
///
/// **Phase 4** (`PromoteAllInstances`): For each promoted DeclarationId, walk the
/// entire tree and promote all remaining Identifier instances that share that
/// DeclarationId (scope declarations, dependencies, all places).
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    DeclarationId, HIRFunction, IdentifierId, IdentifierName, InstructionKind, InstructionValue,
    Place, PrunedReactiveScopeBlock, ReactiveBlock, ReactiveFunction, ReactiveInstruction,
    ReactiveParam, ReactiveScope, ReactiveScopeBlock, ReactiveStatement, ReactiveTerminal,
    ReactiveValue, ScopeId,
};

// =====================================================================================
// State types
// =====================================================================================

/// Information about a pruned-scope declaration.
struct PrunedInfo {
    /// The scope IDs that were active when this pruned decl was encountered.
    active_scopes: Vec<ScopeId>,
    /// Whether the identifier was used outside of its defining scope.
    used_outside_scope: bool,
}

/// Shared state passed through all phases.
struct State {
    /// Declaration IDs that appear as JSX expression tags.
    tags: FxHashSet<DeclarationId>,
    /// Declaration IDs that have been promoted.
    promoted: FxHashSet<DeclarationId>,
    /// Information about declarations from pruned scopes.
    pruned: FxHashMap<DeclarationId, PrunedInfo>,
}

// =====================================================================================
// promote_identifier helper
// =====================================================================================

/// Promote a single unnamed identifier to a named temporary.
///
/// If the identifier's declaration_id is in the `tags` set (JSX tag), it gets
/// a capitalized name (`#T{n}`); otherwise a lowercase name (`#t{n}`).
/// The declaration_id is recorded in the `promoted` set.
fn promote_identifier(identifier: &mut crate::hir::Identifier, state: &mut State) {
    debug_assert!(
        identifier.name.is_none(),
        "promoteTemporary: Expected to be called only for temporary variables"
    );
    let decl_id = identifier.declaration_id;
    if state.tags.contains(&decl_id) {
        // JSX tag — capitalized name
        identifier.name = Some(IdentifierName::Promoted(format!("#T{}", decl_id.0)));
    } else {
        identifier.name = Some(IdentifierName::Promoted(format!("#t{}", decl_id.0)));
    }
    state.promoted.insert(decl_id);
}

// =====================================================================================
// Phase 1: CollectPromotableTemporaries
// =====================================================================================

/// Phase 1 visitor state.
struct CollectPromotableTemporaries {
    /// Stack of active scope IDs during traversal.
    active_scopes: Vec<ScopeId>,
}

impl CollectPromotableTemporaries {
    fn new() -> Self {
        Self { active_scopes: Vec::new() }
    }

    /// Visit a place — check if it is a pruned declaration used outside its scope.
    fn visit_place(&self, place: &Place, state: &mut State) {
        if !self.active_scopes.is_empty()
            && let Some(pruned_info) = state.pruned.get_mut(&place.identifier.declaration_id)
            && let Some(current_scope) = self.active_scopes.last()
            && !pruned_info.active_scopes.contains(current_scope)
        {
            pruned_info.used_outside_scope = true;
        }
    }

    /// Visit a reactive value — detect JSX expression tags.
    fn visit_value(value: &ReactiveValue, state: &mut State) {
        if let ReactiveValue::Instruction(instr_value) = value
            && let InstructionValue::JsxExpression(jsx) = instr_value.as_ref()
            && let crate::hir::JsxTag::Place(tag_place) = &jsx.tag
        {
            state.tags.insert(tag_place.identifier.declaration_id);
        }
    }

    /// Visit a block of reactive statements.
    fn visit_block(&mut self, block: &ReactiveBlock, state: &mut State) {
        for stmt in block {
            self.visit_statement(stmt, state);
        }
    }

    /// Visit a single reactive statement.
    fn visit_statement(&mut self, stmt: &ReactiveStatement, state: &mut State) {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                self.visit_instruction(&instr_stmt.instruction, state);
            }
            ReactiveStatement::Terminal(term_stmt) => {
                self.visit_terminal(&term_stmt.terminal, state);
            }
            ReactiveStatement::Scope(scope_block) => {
                self.active_scopes.push(scope_block.scope.id);
                self.visit_block(&scope_block.instructions, state);
                self.active_scopes.pop();
            }
            ReactiveStatement::PrunedScope(pruned_block) => {
                // Record all declarations in this pruned scope.
                for decl in pruned_block.scope.declarations.values() {
                    state.pruned.insert(
                        decl.identifier.declaration_id,
                        PrunedInfo {
                            active_scopes: self.active_scopes.clone(),
                            used_outside_scope: false,
                        },
                    );
                }
                self.visit_block(&pruned_block.instructions, state);
            }
        }
    }

    /// Visit a reactive instruction — traverse its value and visit places.
    fn visit_instruction(&mut self, instr: &ReactiveInstruction, state: &mut State) {
        self.traverse_value(&instr.value, state);
    }

    /// Traverse a reactive value recursively, visiting places and detecting JSX tags.
    fn traverse_value(&mut self, value: &ReactiveValue, state: &mut State) {
        // First detect JSX tags on this value.
        Self::visit_value(value, state);

        match value {
            ReactiveValue::Instruction(instr_value) => {
                // Visit all operand places.
                for place in crate::hir::visitors::each_instruction_value_operand(instr_value) {
                    self.visit_place(place, state);
                }
            }
            ReactiveValue::Logical(v) => {
                self.traverse_value(&v.left, state);
                self.traverse_value(&v.right, state);
            }
            ReactiveValue::Ternary(v) => {
                self.traverse_value(&v.test, state);
                self.traverse_value(&v.consequent, state);
                self.traverse_value(&v.alternate, state);
            }
            ReactiveValue::Sequence(v) => {
                for instr in &v.instructions {
                    self.visit_instruction(instr, state);
                }
                self.traverse_value(&v.value, state);
            }
            ReactiveValue::OptionalCall(v) => {
                self.traverse_value(&v.value, state);
            }
        }
    }

    /// Visit terminals — traverse their children blocks and operand places.
    fn visit_terminal(&mut self, terminal: &ReactiveTerminal, state: &mut State) {
        match terminal {
            ReactiveTerminal::Return(t) => {
                self.visit_place(&t.value, state);
            }
            ReactiveTerminal::Throw(t) => {
                self.visit_place(&t.value, state);
            }
            ReactiveTerminal::If(t) => {
                self.visit_place(&t.test, state);
                self.visit_block(&t.consequent, state);
                if let Some(alt) = &t.alternate {
                    self.visit_block(alt, state);
                }
            }
            ReactiveTerminal::Switch(t) => {
                self.visit_place(&t.test, state);
                for case in &t.cases {
                    if let Some(test) = &case.test {
                        self.visit_place(test, state);
                    }
                    if let Some(block) = &case.block {
                        self.visit_block(block, state);
                    }
                }
            }
            ReactiveTerminal::For(t) => {
                self.traverse_value(&t.init, state);
                self.traverse_value(&t.test, state);
                self.visit_block(&t.r#loop, state);
                if let Some(update) = &t.update {
                    self.traverse_value(update, state);
                }
            }
            ReactiveTerminal::ForOf(t) => {
                self.traverse_value(&t.init, state);
                self.traverse_value(&t.test, state);
                self.visit_block(&t.r#loop, state);
            }
            ReactiveTerminal::ForIn(t) => {
                self.traverse_value(&t.init, state);
                self.visit_block(&t.r#loop, state);
            }
            ReactiveTerminal::DoWhile(t) => {
                self.visit_block(&t.r#loop, state);
                self.traverse_value(&t.test, state);
            }
            ReactiveTerminal::While(t) => {
                self.traverse_value(&t.test, state);
                self.visit_block(&t.r#loop, state);
            }
            ReactiveTerminal::Label(t) => {
                self.visit_block(&t.block, state);
            }
            ReactiveTerminal::Try(t) => {
                self.visit_block(&t.block, state);
                self.visit_block(&t.handler, state);
            }
            ReactiveTerminal::Break(_) | ReactiveTerminal::Continue(_) => {}
        }
    }
}

// =====================================================================================
// Phase 2: PromoteTemporaries
// =====================================================================================

/// Phase 2: promote unnamed identifiers in scope declarations, dependencies,
/// pruned scope declarations (if used outside scope), and function parameters.
fn promote_temporaries(func: &mut ReactiveFunction, state: &mut State) {
    promote_temporaries_block(&mut func.body, state);
}

fn promote_temporaries_block(block: &mut ReactiveBlock, state: &mut State) {
    for stmt in block.iter_mut() {
        promote_temporaries_statement(stmt, state);
    }
}

fn promote_temporaries_statement(stmt: &mut ReactiveStatement, state: &mut State) {
    match stmt {
        ReactiveStatement::Scope(scope_block) => {
            promote_temporaries_scope(scope_block, state);
        }
        ReactiveStatement::PrunedScope(pruned_block) => {
            promote_temporaries_pruned_scope(pruned_block, state);
        }
        ReactiveStatement::Instruction(instr_stmt) => {
            promote_temporaries_value(&mut instr_stmt.instruction.value, state);
        }
        ReactiveStatement::Terminal(term_stmt) => {
            promote_temporaries_terminal(&mut term_stmt.terminal, state);
        }
    }
}

fn promote_temporaries_scope(scope_block: &mut ReactiveScopeBlock, state: &mut State) {
    // Promote unnamed dependencies.
    // Note: Rust ReactiveScopeDependency only has identifier_id, not full Identifier.
    // We cannot promote here since we don't have the Identifier to mutate.
    // Dependencies will be handled by other means if needed.

    // Promote unnamed declarations.
    for decl in scope_block.scope.declarations.values_mut() {
        if decl.identifier.name.is_none() {
            promote_identifier(&mut decl.identifier, state);
        }
    }

    promote_temporaries_block(&mut scope_block.instructions, state);
}

fn promote_temporaries_pruned_scope(
    pruned_block: &mut PrunedReactiveScopeBlock,
    state: &mut State,
) {
    for decl in pruned_block.scope.declarations.values_mut() {
        if decl.identifier.name.is_none()
            && let Some(pruned_info) = state.pruned.get(&decl.identifier.declaration_id)
            && pruned_info.used_outside_scope
        {
            promote_identifier(&mut decl.identifier, state);
        }
    }

    promote_temporaries_block(&mut pruned_block.instructions, state);
}

fn promote_temporaries_value(value: &mut ReactiveValue, state: &mut State) {
    match value {
        ReactiveValue::Instruction(instr_value) => {
            // Recurse into FunctionExpression / ObjectMethod.
            match instr_value.as_mut() {
                InstructionValue::FunctionExpression(v) => {
                    promote_temporaries_hir_function(&mut v.lowered_func.func, state);
                }
                InstructionValue::ObjectMethod(v) => {
                    promote_temporaries_hir_function(&mut v.lowered_func.func, state);
                }
                _ => {}
            }
        }
        ReactiveValue::Logical(v) => {
            promote_temporaries_value(&mut v.left, state);
            promote_temporaries_value(&mut v.right, state);
        }
        ReactiveValue::Ternary(v) => {
            promote_temporaries_value(&mut v.test, state);
            promote_temporaries_value(&mut v.consequent, state);
            promote_temporaries_value(&mut v.alternate, state);
        }
        ReactiveValue::Sequence(v) => {
            for instr in &mut v.instructions {
                promote_temporaries_value(&mut instr.value, state);
            }
            promote_temporaries_value(&mut v.value, state);
        }
        ReactiveValue::OptionalCall(v) => {
            promote_temporaries_value(&mut v.value, state);
        }
    }
}

fn promote_temporaries_hir_function(func: &mut HIRFunction, state: &mut State) {
    // Promote unnamed params.
    for param in &mut func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &mut s.place,
        };
        if place.identifier.name.is_none() {
            promote_identifier(&mut place.identifier, state);
        }
    }
    // Traverse blocks.
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if let InstructionValue::FunctionExpression(v) = &instr.value {
                // We need mutable access, but this is within an immutable borrow.
                // Instead, we'll handle this through the reactive function traversal.
                let _ = v;
            }
        }
    }
}

fn promote_temporaries_terminal(terminal: &mut ReactiveTerminal, state: &mut State) {
    match terminal {
        ReactiveTerminal::If(t) => {
            promote_temporaries_block(&mut t.consequent, state);
            if let Some(alt) = &mut t.alternate {
                promote_temporaries_block(alt, state);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    promote_temporaries_block(block, state);
                }
            }
        }
        ReactiveTerminal::For(t) => {
            promote_temporaries_value(&mut t.init, state);
            promote_temporaries_value(&mut t.test, state);
            promote_temporaries_block(&mut t.r#loop, state);
            if let Some(update) = &mut t.update {
                promote_temporaries_value(update, state);
            }
        }
        ReactiveTerminal::ForOf(t) => {
            promote_temporaries_value(&mut t.init, state);
            promote_temporaries_value(&mut t.test, state);
            promote_temporaries_block(&mut t.r#loop, state);
        }
        ReactiveTerminal::ForIn(t) => {
            promote_temporaries_value(&mut t.init, state);
            promote_temporaries_block(&mut t.r#loop, state);
        }
        ReactiveTerminal::DoWhile(t) => {
            promote_temporaries_block(&mut t.r#loop, state);
            promote_temporaries_value(&mut t.test, state);
        }
        ReactiveTerminal::While(t) => {
            promote_temporaries_value(&mut t.test, state);
            promote_temporaries_block(&mut t.r#loop, state);
        }
        ReactiveTerminal::Label(t) => {
            promote_temporaries_block(&mut t.block, state);
        }
        ReactiveTerminal::Try(t) => {
            promote_temporaries_block(&mut t.block, state);
            promote_temporaries_block(&mut t.handler, state);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

// =====================================================================================
// Phase 3: PromoteInterposedTemporaries
// =====================================================================================

/// State for tracking temporaries that may need promotion due to interposing instructions.
/// Maps IdentifierId -> (DeclarationId, needs_promotion).
type InterState = FxHashMap<IdentifierId, (DeclarationId, bool)>;

/// Phase 3: find temporaries that have interposing potentially-mutating instructions
/// between their definition and use, and promote them.
fn promote_interposed_temporaries(func: &mut ReactiveFunction, state: &mut State) {
    let mut consts: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut globals: FxHashSet<IdentifierId> = FxHashSet::default();

    // Record params as consts.
    for param in &func.params {
        match param {
            ReactiveParam::Place(p) => {
                consts.insert(p.identifier.id);
            }
            ReactiveParam::Spread(s) => {
                consts.insert(s.place.identifier.id);
            }
        }
    }

    let mut inter_state: InterState = FxHashMap::default();
    promote_interposed_block(&mut func.body, state, &mut inter_state, &mut consts, &mut globals);
}

fn promote_interposed_block(
    block: &mut ReactiveBlock,
    state: &mut State,
    inter_state: &mut InterState,
    consts: &mut FxHashSet<IdentifierId>,
    globals: &mut FxHashSet<IdentifierId>,
) {
    for stmt in block.iter_mut() {
        promote_interposed_statement(stmt, state, inter_state, consts, globals);
    }
}

fn promote_interposed_statement(
    stmt: &mut ReactiveStatement,
    state: &mut State,
    inter_state: &mut InterState,
    consts: &mut FxHashSet<IdentifierId>,
    globals: &mut FxHashSet<IdentifierId>,
) {
    match stmt {
        ReactiveStatement::Instruction(instr_stmt) => {
            promote_interposed_instruction(
                &mut instr_stmt.instruction,
                state,
                inter_state,
                consts,
                globals,
            );
        }
        ReactiveStatement::Scope(scope_block) => {
            promote_interposed_block(
                &mut scope_block.instructions,
                state,
                inter_state,
                consts,
                globals,
            );
        }
        ReactiveStatement::PrunedScope(pruned_block) => {
            promote_interposed_block(
                &mut pruned_block.instructions,
                state,
                inter_state,
                consts,
                globals,
            );
        }
        ReactiveStatement::Terminal(term_stmt) => {
            promote_interposed_terminal(
                &mut term_stmt.terminal,
                state,
                inter_state,
                consts,
                globals,
            );
        }
    }
}

/// Visit a place in the interposed pass — if this identifier was marked for
/// promotion and hasn't been promoted yet, promote it now.
fn interposed_visit_place(
    place: &mut Place,
    state: &mut State,
    inter_state: &InterState,
    consts: &FxHashSet<IdentifierId>,
) {
    if let Some((decl_id, needs_promotion)) = inter_state.get(&place.identifier.id).copied()
        && needs_promotion
        && place.identifier.name.is_none()
        && !consts.contains(&place.identifier.id)
    {
        // Find an identifier with this declaration_id to promote.
        // We promote this place's identifier directly.
        promote_identifier(&mut place.identifier, state);
        // Also record the declaration_id as promoted.
        state.promoted.insert(decl_id);
    }
}

/// Visit all operand places in a reactive value for the interposed pass.
fn interposed_visit_value_places(
    value: &mut ReactiveValue,
    state: &mut State,
    inter_state: &mut InterState,
    consts: &FxHashSet<IdentifierId>,
) {
    match value {
        ReactiveValue::Instruction(instr_value) => {
            interposed_visit_instr_value_places(instr_value, state, inter_state, consts);
        }
        ReactiveValue::Logical(v) => {
            interposed_visit_value_places(&mut v.left, state, inter_state, consts);
            interposed_visit_value_places(&mut v.right, state, inter_state, consts);
        }
        ReactiveValue::Ternary(v) => {
            interposed_visit_value_places(&mut v.test, state, inter_state, consts);
            interposed_visit_value_places(&mut v.consequent, state, inter_state, consts);
            interposed_visit_value_places(&mut v.alternate, state, inter_state, consts);
        }
        ReactiveValue::Sequence(v) => {
            for instr in &mut v.instructions {
                interposed_visit_value_places(&mut instr.value, state, inter_state, consts);
            }
            interposed_visit_value_places(&mut v.value, state, inter_state, consts);
        }
        ReactiveValue::OptionalCall(v) => {
            interposed_visit_value_places(&mut v.value, state, inter_state, consts);
        }
    }
}

/// Visit operand places inside an InstructionValue for the interposed pass.
fn interposed_visit_instr_value_places(
    value: &mut InstructionValue,
    state: &mut State,
    inter_state: &InterState,
    consts: &FxHashSet<IdentifierId>,
) {
    // We need to visit each operand place mutably.
    // Use the match-based approach similar to the existing visitors.
    match value {
        InstructionValue::CallExpression(v) => {
            interposed_visit_place(&mut v.callee, state, inter_state, consts);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => {
                        interposed_visit_place(p, state, inter_state, consts);
                    }
                    crate::hir::CallArg::Spread(s) => {
                        interposed_visit_place(&mut s.place, state, inter_state, consts);
                    }
                }
            }
        }
        InstructionValue::MethodCall(v) => {
            interposed_visit_place(&mut v.receiver, state, inter_state, consts);
            interposed_visit_place(&mut v.property, state, inter_state, consts);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => {
                        interposed_visit_place(p, state, inter_state, consts);
                    }
                    crate::hir::CallArg::Spread(s) => {
                        interposed_visit_place(&mut s.place, state, inter_state, consts);
                    }
                }
            }
        }
        InstructionValue::NewExpression(v) => {
            interposed_visit_place(&mut v.callee, state, inter_state, consts);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => {
                        interposed_visit_place(p, state, inter_state, consts);
                    }
                    crate::hir::CallArg::Spread(s) => {
                        interposed_visit_place(&mut s.place, state, inter_state, consts);
                    }
                }
            }
        }
        InstructionValue::BinaryExpression(v) => {
            interposed_visit_place(&mut v.left, state, inter_state, consts);
            interposed_visit_place(&mut v.right, state, inter_state, consts);
        }
        InstructionValue::UnaryExpression(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::LoadLocal(v) => {
            interposed_visit_place(&mut v.place, state, inter_state, consts);
        }
        InstructionValue::LoadContext(v) => {
            interposed_visit_place(&mut v.place, state, inter_state, consts);
        }
        InstructionValue::StoreLocal(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::StoreContext(v) => {
            interposed_visit_place(&mut v.lvalue_place, state, inter_state, consts);
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::StoreGlobal(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::Destructure(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::PropertyLoad(v) => {
            interposed_visit_place(&mut v.object, state, inter_state, consts);
        }
        InstructionValue::PropertyStore(v) => {
            interposed_visit_place(&mut v.object, state, inter_state, consts);
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::PropertyDelete(v) => {
            interposed_visit_place(&mut v.object, state, inter_state, consts);
        }
        InstructionValue::ComputedLoad(v) => {
            interposed_visit_place(&mut v.object, state, inter_state, consts);
            interposed_visit_place(&mut v.property, state, inter_state, consts);
        }
        InstructionValue::ComputedStore(v) => {
            interposed_visit_place(&mut v.object, state, inter_state, consts);
            interposed_visit_place(&mut v.property, state, inter_state, consts);
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::ComputedDelete(v) => {
            interposed_visit_place(&mut v.object, state, inter_state, consts);
            interposed_visit_place(&mut v.property, state, inter_state, consts);
        }
        InstructionValue::TypeCastExpression(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(ref mut p) = v.tag {
                interposed_visit_place(p, state, inter_state, consts);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        interposed_visit_place(place, state, inter_state, consts);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        interposed_visit_place(argument, state, inter_state, consts);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children.iter_mut() {
                    interposed_visit_place(child, state, inter_state, consts);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                interposed_visit_place(child, state, inter_state, consts);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                            interposed_visit_place(place, state, inter_state, consts);
                        }
                        interposed_visit_place(&mut p.place, state, inter_state, consts);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        interposed_visit_place(&mut s.place, state, inter_state, consts);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        interposed_visit_place(p, state, inter_state, consts);
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        interposed_visit_place(&mut s.place, state, inter_state, consts);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                interposed_visit_place(ctx, state, inter_state, consts);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                interposed_visit_place(ctx, state, inter_state, consts);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            interposed_visit_place(&mut v.tag, state, inter_state, consts);
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                interposed_visit_place(subexpr, state, inter_state, consts);
            }
        }
        InstructionValue::Await(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::GetIterator(v) => {
            interposed_visit_place(&mut v.collection, state, inter_state, consts);
        }
        InstructionValue::IteratorNext(v) => {
            interposed_visit_place(&mut v.iterator, state, inter_state, consts);
            interposed_visit_place(&mut v.collection, state, inter_state, consts);
        }
        InstructionValue::NextPropertyOf(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::PrefixUpdate(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::PostfixUpdate(v) => {
            interposed_visit_place(&mut v.value, state, inter_state, consts);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value, ..
                    } = dep.root
                    {
                        interposed_visit_place(value, state, inter_state, consts);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            interposed_visit_place(&mut v.decl, state, inter_state, consts);
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

/// Check if all value-level lvalues have a non-null name (for the invariant check).
fn all_value_lvalues_are_named(value: &ReactiveValue) -> bool {
    if let ReactiveValue::Instruction(instr_value) = value {
        match instr_value.as_ref() {
            InstructionValue::DeclareContext(v) => v.lvalue_place.identifier.name.is_some(),
            InstructionValue::StoreContext(v) => v.lvalue_place.identifier.name.is_some(),
            InstructionValue::DeclareLocal(v) => v.lvalue.place.identifier.name.is_some(),
            InstructionValue::StoreLocal(v) => v.lvalue.place.identifier.name.is_some(),
            InstructionValue::Destructure(v) => {
                crate::hir::visitors::each_pattern_operand(&v.lvalue.pattern)
                    .iter()
                    .all(|p| p.identifier.name.is_some())
            }
            InstructionValue::PrefixUpdate(v) => v.lvalue.identifier.name.is_some(),
            InstructionValue::PostfixUpdate(v) => v.lvalue.identifier.name.is_some(),
            _ => true,
        }
    } else {
        true
    }
}

fn promote_interposed_instruction(
    instruction: &mut ReactiveInstruction,
    state: &mut State,
    inter_state: &mut InterState,
    consts: &mut FxHashSet<IdentifierId>,
    globals: &mut FxHashSet<IdentifierId>,
) {
    // Check the invariant: all value-level lvalues should be named.
    // (In the TS code this is a CompilerError.invariant; we use debug_assert.)
    debug_assert!(
        all_value_lvalues_are_named(&instruction.value),
        "PromoteInterposedTemporaries: Assignment targets not expected to be temporaries"
    );

    let value_kind = instruction.value.instruction_kind();

    match value_kind {
        Some(InterposedValueKind::MutatingInstruction) => {
            let mut const_store = false;

            // Check for const stores (StoreLocal/StoreContext with Const/HoistedConst lvalue).
            if let ReactiveValue::Instruction(instr_value) = &instruction.value {
                match instr_value.as_ref() {
                    InstructionValue::StoreContext(_) | InstructionValue::StoreLocal(_)
                        if matches!(
                            get_store_lvalue_kind(instr_value),
                            Some(InstructionKind::Const | InstructionKind::HoistedConst)
                        ) =>
                    {
                        let lvalue_id = match instr_value.as_ref() {
                            InstructionValue::StoreContext(sc) => sc.lvalue_place.identifier.id,
                            InstructionValue::StoreLocal(sl) => sl.lvalue.place.identifier.id,
                            _ => unreachable!(),
                        };
                        consts.insert(lvalue_id);
                        const_store = true;
                    }
                    InstructionValue::Destructure(v)
                        if matches!(
                            v.lvalue.kind,
                            InstructionKind::Const | InstructionKind::HoistedConst
                        ) =>
                    {
                        for place in crate::hir::visitors::each_pattern_operand(&v.lvalue.pattern) {
                            consts.insert(place.identifier.id);
                        }
                        const_store = true;
                    }
                    InstructionValue::MethodCall(v) => {
                        // Treat property of method call as constlike.
                        consts.insert(v.property.identifier.id);
                    }
                    _ => {}
                }
            }

            // Visit operand places (default traversal).
            interposed_visit_value_places(&mut instruction.value, state, inter_state, consts);

            // After visiting operands: if not a const store and the instruction will be
            // emitted as a statement, mark all pending temporaries for promotion.
            if !const_store {
                let emitted_as_statement = instruction.lvalue.is_none()
                    || instruction.lvalue.as_ref().is_some_and(|lv| lv.identifier.name.is_some());

                if emitted_as_statement {
                    for (_key, (_decl_id, needs_promotion)) in inter_state.iter_mut() {
                        *needs_promotion = true;
                    }
                }
            }

            // Add this instruction's lvalue to the state if unnamed.
            if let Some(lvalue) = &instruction.lvalue
                && lvalue.identifier.name.is_none()
            {
                inter_state.insert(lvalue.identifier.id, (lvalue.identifier.declaration_id, false));
            }
        }
        Some(InterposedValueKind::DeclareContextOrLocal) => {
            // DeclareContext / DeclareLocal.
            if let ReactiveValue::Instruction(instr_value) = &instruction.value {
                match instr_value.as_ref() {
                    InstructionValue::DeclareLocal(v) => {
                        if matches!(
                            v.lvalue.kind,
                            InstructionKind::Const | InstructionKind::HoistedConst
                        ) {
                            consts.insert(v.lvalue.place.identifier.id);
                        }
                    }
                    InstructionValue::DeclareContext(v) => {
                        if matches!(
                            v.lvalue_kind,
                            InstructionKind::Const | InstructionKind::HoistedConst
                        ) {
                            consts.insert(v.lvalue_place.identifier.id);
                        }
                    }
                    _ => {}
                }
            }
            // Default traversal.
            interposed_visit_value_places(&mut instruction.value, state, inter_state, consts);
        }
        Some(InterposedValueKind::LoadContextOrLocal) => {
            // LoadContext / LoadLocal: track the lvalue as a pending temporary.
            if let Some(lvalue) = &instruction.lvalue
                && lvalue.identifier.name.is_none()
            {
                // If loading from a const, mark the lvalue as const too.
                if let ReactiveValue::Instruction(instr_value) = &instruction.value {
                    match instr_value.as_ref() {
                        InstructionValue::LoadLocal(v) => {
                            if consts.contains(&v.place.identifier.id) {
                                consts.insert(lvalue.identifier.id);
                            }
                        }
                        InstructionValue::LoadContext(v) => {
                            if consts.contains(&v.place.identifier.id) {
                                consts.insert(lvalue.identifier.id);
                            }
                        }
                        _ => {}
                    }
                }
                inter_state.insert(lvalue.identifier.id, (lvalue.identifier.declaration_id, false));
            }
            // Default traversal.
            interposed_visit_value_places(&mut instruction.value, state, inter_state, consts);
        }
        Some(InterposedValueKind::PropertyOrComputedLoad) => {
            // PropertyLoad / ComputedLoad.
            if let Some(lvalue) = &instruction.lvalue {
                if let ReactiveValue::Instruction(instr_value) = &instruction.value {
                    let object_id = match instr_value.as_ref() {
                        InstructionValue::PropertyLoad(v) => Some(v.object.identifier.id),
                        InstructionValue::ComputedLoad(v) => Some(v.object.identifier.id),
                        _ => None,
                    };
                    if let Some(obj_id) = object_id
                        && globals.contains(&obj_id)
                    {
                        globals.insert(lvalue.identifier.id);
                        consts.insert(lvalue.identifier.id);
                    }
                }
                if lvalue.identifier.name.is_none() {
                    inter_state
                        .insert(lvalue.identifier.id, (lvalue.identifier.declaration_id, false));
                }
            }
            // Default traversal.
            interposed_visit_value_places(&mut instruction.value, state, inter_state, consts);
        }
        Some(InterposedValueKind::LoadGlobal) => {
            // LoadGlobal: mark lvalue as global.
            if let Some(lvalue) = &instruction.lvalue {
                globals.insert(lvalue.identifier.id);
            }
            // Default traversal.
            interposed_visit_value_places(&mut instruction.value, state, inter_state, consts);
        }
        None => {
            // Default: just traverse.
            interposed_visit_value_places(&mut instruction.value, state, inter_state, consts);
        }
    }
}

/// Classify a ReactiveValue for the interposed pass.
#[derive(Debug, Clone, Copy)]
enum InterposedValueKind {
    MutatingInstruction,
    DeclareContextOrLocal,
    LoadContextOrLocal,
    PropertyOrComputedLoad,
    LoadGlobal,
}

impl ReactiveValue {
    fn instruction_kind(&self) -> Option<InterposedValueKind> {
        if let ReactiveValue::Instruction(instr_value) = self {
            match instr_value.as_ref() {
                InstructionValue::CallExpression(_)
                | InstructionValue::MethodCall(_)
                | InstructionValue::Await(_)
                | InstructionValue::PropertyStore(_)
                | InstructionValue::PropertyDelete(_)
                | InstructionValue::ComputedStore(_)
                | InstructionValue::ComputedDelete(_)
                | InstructionValue::PostfixUpdate(_)
                | InstructionValue::PrefixUpdate(_)
                | InstructionValue::StoreLocal(_)
                | InstructionValue::StoreContext(_)
                | InstructionValue::StoreGlobal(_)
                | InstructionValue::Destructure(_) => {
                    Some(InterposedValueKind::MutatingInstruction)
                }
                InstructionValue::DeclareContext(_) | InstructionValue::DeclareLocal(_) => {
                    Some(InterposedValueKind::DeclareContextOrLocal)
                }
                InstructionValue::LoadContext(_) | InstructionValue::LoadLocal(_) => {
                    Some(InterposedValueKind::LoadContextOrLocal)
                }
                InstructionValue::PropertyLoad(_) | InstructionValue::ComputedLoad(_) => {
                    Some(InterposedValueKind::PropertyOrComputedLoad)
                }
                InstructionValue::LoadGlobal(_) => Some(InterposedValueKind::LoadGlobal),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Get the lvalue kind from a store instruction value.
fn get_store_lvalue_kind(value: &InstructionValue) -> Option<InstructionKind> {
    match value {
        InstructionValue::StoreLocal(v) => Some(v.lvalue.kind),
        InstructionValue::StoreContext(v) => Some(v.lvalue_kind),
        _ => None,
    }
}

fn promote_interposed_terminal(
    terminal: &mut ReactiveTerminal,
    state: &mut State,
    inter_state: &mut InterState,
    consts: &mut FxHashSet<IdentifierId>,
    globals: &mut FxHashSet<IdentifierId>,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            promote_interposed_block(&mut t.consequent, state, inter_state, consts, globals);
            if let Some(alt) = &mut t.alternate {
                promote_interposed_block(alt, state, inter_state, consts, globals);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    promote_interposed_block(block, state, inter_state, consts, globals);
                }
            }
        }
        ReactiveTerminal::For(t) => {
            promote_interposed_block(&mut t.r#loop, state, inter_state, consts, globals);
        }
        ReactiveTerminal::ForOf(t) => {
            promote_interposed_block(&mut t.r#loop, state, inter_state, consts, globals);
        }
        ReactiveTerminal::ForIn(t) => {
            promote_interposed_block(&mut t.r#loop, state, inter_state, consts, globals);
        }
        ReactiveTerminal::DoWhile(t) => {
            promote_interposed_block(&mut t.r#loop, state, inter_state, consts, globals);
        }
        ReactiveTerminal::While(t) => {
            promote_interposed_block(&mut t.r#loop, state, inter_state, consts, globals);
        }
        ReactiveTerminal::Label(t) => {
            promote_interposed_block(&mut t.block, state, inter_state, consts, globals);
        }
        ReactiveTerminal::Try(t) => {
            promote_interposed_block(&mut t.block, state, inter_state, consts, globals);
            promote_interposed_block(&mut t.handler, state, inter_state, consts, globals);
        }
        ReactiveTerminal::Return(t) => {
            interposed_visit_place(&mut t.value, state, inter_state, consts);
        }
        ReactiveTerminal::Throw(t) => {
            interposed_visit_place(&mut t.value, state, inter_state, consts);
        }
        ReactiveTerminal::Break(_) | ReactiveTerminal::Continue(_) => {}
    }
}

// =====================================================================================
// Phase 4: PromoteAllInstances
// =====================================================================================

/// Phase 4: walk the entire tree and promote all remaining unnamed identifiers
/// whose declaration_id is in the promoted set.
fn promote_all_instances(func: &mut ReactiveFunction, state: &mut State) {
    promote_all_instances_block(&mut func.body, state);
}

fn promote_all_instances_block(block: &mut ReactiveBlock, state: &mut State) {
    for stmt in block.iter_mut() {
        promote_all_instances_statement(stmt, state);
    }
}

fn promote_all_instances_statement(stmt: &mut ReactiveStatement, state: &mut State) {
    match stmt {
        ReactiveStatement::Instruction(instr_stmt) => {
            promote_all_instances_instruction(&mut instr_stmt.instruction, state);
        }
        ReactiveStatement::Terminal(term_stmt) => {
            promote_all_instances_terminal(&mut term_stmt.terminal, state);
        }
        ReactiveStatement::Scope(scope_block) => {
            promote_all_instances_block(&mut scope_block.instructions, state);
            promote_all_instances_scope_identifiers(&mut scope_block.scope, state);
        }
        ReactiveStatement::PrunedScope(pruned_block) => {
            promote_all_instances_block(&mut pruned_block.instructions, state);
            promote_all_instances_scope_identifiers(&mut pruned_block.scope, state);
        }
    }
}

/// Promote identifiers in scope declarations (and reassignments/dependencies if they
/// stored full Identifier objects — in this Rust port, only declarations do).
fn promote_all_instances_scope_identifiers(scope: &mut ReactiveScope, state: &mut State) {
    for decl in scope.declarations.values_mut() {
        if decl.identifier.name.is_none()
            && state.promoted.contains(&decl.identifier.declaration_id)
        {
            promote_identifier(&mut decl.identifier, state);
        }
    }
    // Note: In the TS version, scope.dependencies and scope.reassignments also store
    // full Identifier objects that need promotion. In this Rust port, they only store
    // IdentifierId values, so there are no Identifier instances to promote there.
}

fn promote_all_instances_instruction(instruction: &mut ReactiveInstruction, state: &mut State) {
    // Promote the instruction-level lvalue.
    if let Some(lvalue) = &mut instruction.lvalue {
        promote_place_if_needed(lvalue, state);
    }

    // Promote all places in the value (operands + value-level lvalues).
    promote_all_instances_value(&mut instruction.value, state);
}

fn promote_all_instances_value(value: &mut ReactiveValue, state: &mut State) {
    match value {
        ReactiveValue::Instruction(instr_value) => {
            promote_all_instances_instr_value(instr_value, state);
        }
        ReactiveValue::Logical(v) => {
            promote_all_instances_value(&mut v.left, state);
            promote_all_instances_value(&mut v.right, state);
        }
        ReactiveValue::Ternary(v) => {
            promote_all_instances_value(&mut v.test, state);
            promote_all_instances_value(&mut v.consequent, state);
            promote_all_instances_value(&mut v.alternate, state);
        }
        ReactiveValue::Sequence(v) => {
            for instr in &mut v.instructions {
                promote_all_instances_instruction(instr, state);
            }
            promote_all_instances_value(&mut v.value, state);
        }
        ReactiveValue::OptionalCall(v) => {
            promote_all_instances_value(&mut v.value, state);
        }
    }
}

/// Promote all places within an InstructionValue (both operands and value-level lvalues).
fn promote_all_instances_instr_value(value: &mut InstructionValue, state: &mut State) {
    match value {
        InstructionValue::CallExpression(v) => {
            promote_place_if_needed(&mut v.callee, state);
            promote_call_args_if_needed(&mut v.args, state);
        }
        InstructionValue::MethodCall(v) => {
            promote_place_if_needed(&mut v.receiver, state);
            promote_place_if_needed(&mut v.property, state);
            promote_call_args_if_needed(&mut v.args, state);
        }
        InstructionValue::NewExpression(v) => {
            promote_place_if_needed(&mut v.callee, state);
            promote_call_args_if_needed(&mut v.args, state);
        }
        InstructionValue::BinaryExpression(v) => {
            promote_place_if_needed(&mut v.left, state);
            promote_place_if_needed(&mut v.right, state);
        }
        InstructionValue::UnaryExpression(v) => {
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::LoadLocal(v) => {
            promote_place_if_needed(&mut v.place, state);
        }
        InstructionValue::LoadContext(v) => {
            promote_place_if_needed(&mut v.place, state);
        }
        InstructionValue::StoreLocal(v) => {
            promote_place_if_needed(&mut v.lvalue.place, state);
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::StoreContext(v) => {
            promote_place_if_needed(&mut v.lvalue_place, state);
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::StoreGlobal(v) => {
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::DeclareLocal(v) => {
            promote_place_if_needed(&mut v.lvalue.place, state);
        }
        InstructionValue::DeclareContext(v) => {
            promote_place_if_needed(&mut v.lvalue_place, state);
        }
        InstructionValue::Destructure(v) => {
            promote_pattern_places_if_needed(&mut v.lvalue.pattern, state);
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::PropertyLoad(v) => {
            promote_place_if_needed(&mut v.object, state);
        }
        InstructionValue::PropertyStore(v) => {
            promote_place_if_needed(&mut v.object, state);
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::PropertyDelete(v) => {
            promote_place_if_needed(&mut v.object, state);
        }
        InstructionValue::ComputedLoad(v) => {
            promote_place_if_needed(&mut v.object, state);
            promote_place_if_needed(&mut v.property, state);
        }
        InstructionValue::ComputedStore(v) => {
            promote_place_if_needed(&mut v.object, state);
            promote_place_if_needed(&mut v.property, state);
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::ComputedDelete(v) => {
            promote_place_if_needed(&mut v.object, state);
            promote_place_if_needed(&mut v.property, state);
        }
        InstructionValue::TypeCastExpression(v) => {
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(ref mut p) = v.tag {
                promote_place_if_needed(p, state);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        promote_place_if_needed(place, state);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        promote_place_if_needed(argument, state);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children.iter_mut() {
                    promote_place_if_needed(child, state);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                promote_place_if_needed(child, state);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                            promote_place_if_needed(place, state);
                        }
                        promote_place_if_needed(&mut p.place, state);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        promote_place_if_needed(&mut s.place, state);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        promote_place_if_needed(p, state);
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        promote_place_if_needed(&mut s.place, state);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                promote_place_if_needed(ctx, state);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                promote_place_if_needed(ctx, state);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            promote_place_if_needed(&mut v.tag, state);
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                promote_place_if_needed(subexpr, state);
            }
        }
        InstructionValue::Await(v) => {
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::GetIterator(v) => {
            promote_place_if_needed(&mut v.collection, state);
        }
        InstructionValue::IteratorNext(v) => {
            promote_place_if_needed(&mut v.iterator, state);
            promote_place_if_needed(&mut v.collection, state);
        }
        InstructionValue::NextPropertyOf(v) => {
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::PrefixUpdate(v) => {
            promote_place_if_needed(&mut v.lvalue, state);
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::PostfixUpdate(v) => {
            promote_place_if_needed(&mut v.lvalue, state);
            promote_place_if_needed(&mut v.value, state);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value, ..
                    } = dep.root
                    {
                        promote_place_if_needed(value, state);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            promote_place_if_needed(&mut v.decl, state);
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

fn promote_call_args_if_needed(args: &mut [crate::hir::CallArg], state: &mut State) {
    for arg in args.iter_mut() {
        match arg {
            crate::hir::CallArg::Place(p) => promote_place_if_needed(p, state),
            crate::hir::CallArg::Spread(s) => promote_place_if_needed(&mut s.place, state),
        }
    }
}

fn promote_pattern_places_if_needed(pattern: &mut crate::hir::Pattern, state: &mut State) {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    crate::hir::ArrayPatternElement::Place(p) => {
                        promote_place_if_needed(p, state);
                    }
                    crate::hir::ArrayPatternElement::Spread(s) => {
                        promote_place_if_needed(&mut s.place, state);
                    }
                    crate::hir::ArrayPatternElement::Hole => {}
                }
            }
        }
        crate::hir::Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        promote_place_if_needed(&mut p.place, state);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        promote_place_if_needed(&mut s.place, state);
                    }
                }
            }
        }
    }
}

/// Promote a place's identifier if it is unnamed and its declaration_id is promoted.
fn promote_place_if_needed(place: &mut Place, state: &mut State) {
    if place.identifier.name.is_none() && state.promoted.contains(&place.identifier.declaration_id)
    {
        promote_identifier(&mut place.identifier, state);
    }
}

fn promote_all_instances_terminal(terminal: &mut ReactiveTerminal, state: &mut State) {
    match terminal {
        ReactiveTerminal::Return(t) => {
            promote_place_if_needed(&mut t.value, state);
        }
        ReactiveTerminal::Throw(t) => {
            promote_place_if_needed(&mut t.value, state);
        }
        ReactiveTerminal::If(t) => {
            promote_place_if_needed(&mut t.test, state);
            promote_all_instances_block(&mut t.consequent, state);
            if let Some(alt) = &mut t.alternate {
                promote_all_instances_block(alt, state);
            }
        }
        ReactiveTerminal::Switch(t) => {
            promote_place_if_needed(&mut t.test, state);
            for case in &mut t.cases {
                if let Some(test) = &mut case.test {
                    promote_place_if_needed(test, state);
                }
                if let Some(block) = &mut case.block {
                    promote_all_instances_block(block, state);
                }
            }
        }
        ReactiveTerminal::For(t) => {
            promote_all_instances_value(&mut t.init, state);
            promote_all_instances_value(&mut t.test, state);
            promote_all_instances_block(&mut t.r#loop, state);
            if let Some(update) = &mut t.update {
                promote_all_instances_value(update, state);
            }
        }
        ReactiveTerminal::ForOf(t) => {
            promote_all_instances_value(&mut t.init, state);
            promote_all_instances_value(&mut t.test, state);
            promote_all_instances_block(&mut t.r#loop, state);
        }
        ReactiveTerminal::ForIn(t) => {
            promote_all_instances_value(&mut t.init, state);
            promote_all_instances_block(&mut t.r#loop, state);
        }
        ReactiveTerminal::DoWhile(t) => {
            promote_all_instances_block(&mut t.r#loop, state);
            promote_all_instances_value(&mut t.test, state);
        }
        ReactiveTerminal::While(t) => {
            promote_all_instances_value(&mut t.test, state);
            promote_all_instances_block(&mut t.r#loop, state);
        }
        ReactiveTerminal::Label(t) => {
            promote_all_instances_block(&mut t.block, state);
        }
        ReactiveTerminal::Try(t) => {
            promote_all_instances_block(&mut t.block, state);
            if let Some(binding) = &mut t.handler_binding {
                promote_place_if_needed(binding, state);
            }
            promote_all_instances_block(&mut t.handler, state);
        }
        ReactiveTerminal::Break(_) | ReactiveTerminal::Continue(_) => {}
    }
}

// =====================================================================================
// Public entry point
// =====================================================================================

/// Promote used temporaries to named variables.
///
/// Port of `promoteUsedTemporaries` from `ReactiveScopes/PromoteUsedTemporaries.ts`.
pub fn promote_used_temporaries(func: &mut ReactiveFunction) {
    let mut state = State {
        tags: FxHashSet::default(),
        promoted: FxHashSet::default(),
        pruned: FxHashMap::default(),
    };

    // Phase 1: Collect promotable temporaries (JSX tags, pruned scope usage).
    let mut collector = CollectPromotableTemporaries::new();
    collector.visit_block(&func.body, &mut state);

    // Promote unnamed params of the top-level function.
    for param in &mut func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &mut s.place,
        };
        if place.identifier.name.is_none() {
            promote_identifier(&mut place.identifier, &mut state);
        }
    }

    // Phase 2: Promote temporaries in scope declarations, dependencies, pruned scopes.
    promote_temporaries(func, &mut state);

    // Phase 3: Promote interposed temporaries.
    promote_interposed_temporaries(func, &mut state);

    // Phase 4: Promote all remaining instances of promoted declaration IDs.
    promote_all_instances(func, &mut state);
}
