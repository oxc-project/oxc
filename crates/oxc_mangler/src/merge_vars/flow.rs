//! A control-flow graph of the reads and writes of one scope's candidate variables.
//!
//! Blocks are laid out in evaluation order and every block, read and write takes the next
//! *position*, so a variable's live range is a set of position intervals. Loop bodies occupy a
//! contiguous run of blocks, so backward edges only ever target a loop header that records the
//! end of its loop.
//!
//! Whatever the walker does not model precisely is handled conservatively: candidates seen in an
//! unmodeled position are revoked, and constructs that cannot be modeled at all abort the scope.

use oxc_allocator::{Allocator, ArenaVec, BitSet};
use oxc_ast::{ast::*, match_member_expression};
use oxc_ast_visit::{VisitJs, walk_js};
use oxc_semantic::Scoping;
use oxc_str::Ident;
use oxc_syntax::scope::{ScopeFlags, ScopeId};

use super::{Local, NO_LOCAL, is_simple_parameter_list};

/// [`Edge::to`] is a label rather than a block.
pub(super) const EDGE_TO_LABEL: u8 = 1;
/// The edge is taken when a statement of a `try` block (or `catch` clause) throws.
pub(super) const EDGE_EXCEPTIONAL: u8 = 2;

/// A label that no edge targets yet.
const LABEL_UNUSED: u32 = u32::MAX;
/// A label that edges target but that is not placed at a block yet.
pub(super) const LABEL_PENDING: u32 = u32::MAX - 1;
/// The label of the block after the scope body, which `return` jumps to.
const EXIT_LABEL: u32 = 0;

/// The node whose body is analyzed.
#[derive(Clone, Copy)]
pub(super) enum Root<'a> {
    Program(&'a Program<'a>),
    Function(&'a FormalParameters<'a>, &'a FunctionBody<'a>),
    Arrow(&'a ArrowFunctionExpression<'a>),
    StaticBlock(&'a StaticBlock<'a>),
}

impl<'a> Root<'a> {
    pub(super) fn params(self) -> Option<&'a FormalParameters<'a>> {
        match self {
            Self::Function(params, _) => Some(params),
            Self::Arrow(arrow) => Some(&arrow.params),
            Self::Program(_) | Self::StaticBlock(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Block {
    pub start: u32,
    pub end: u32,
    /// Index of the first event of this block. A block's events end where the next block's begin.
    pub first_event: u32,
    /// For a loop header, the end of the last block of the loop. `0` for other blocks.
    pub loop_end: u32,
}

#[derive(Clone, Copy)]
pub(super) struct Event {
    pub position: u32,
    /// `local << 1 | is_write`.
    packed: u32,
}

impl Event {
    pub(super) fn local(self) -> u32 {
        self.packed >> 1
    }

    pub(super) fn is_write(self) -> bool {
        self.packed & 1 != 0
    }
}

#[derive(Clone, Copy)]
pub(super) struct Edge {
    pub from: u32,
    /// A block, or a label if `flags` has [`EDGE_TO_LABEL`].
    pub to: u32,
    pub flags: u8,
}

/// A statement that `break`, `continue` or `return` may jump out of.
#[derive(Clone, Copy)]
enum JumpContext<'a> {
    Loop {
        break_label: u32,
        continue_label: u32,
    },
    Switch {
        break_label: u32,
    },
    Labeled {
        name: Ident<'a>,
        break_label: u32,
    },
    /// A `try` statement with a `finally` clause, which runs before any jump out of it completes.
    Finally {
        label: u32,
        crossing_start: u32,
    },
}

/// Where a jump goes once every `finally` clause on the way has run.
#[derive(Clone, Copy)]
enum Jump {
    Exit,
    /// To the break label of the context at this index.
    Break(u32),
    /// To the continue label of the context at this index.
    Continue(u32),
}

/// The graph, and scratch buffers reused across scopes.
pub(super) struct FlowBuffers<'a, 't> {
    pub blocks: ArenaVec<'t, Block>,
    pub events: ArenaVec<'t, Event>,
    pub edges: ArenaVec<'t, Edge>,
    /// `labels[label]` is the block the label is placed at, [`LABEL_UNUSED`] or [`LABEL_PENDING`].
    pub labels: ArenaVec<'t, u32>,
    /// Not in the arena, which may not outlive the AST that label names borrow from.
    contexts: Vec<JumpContext<'a>>,
    /// Jumps that reached the `finally` clause of an enclosing context, and continue after it.
    crossings: ArenaVec<'t, Jump>,
    /// Crossings of `finally` clauses that are being walked.
    parked: ArenaVec<'t, Jump>,
    /// The labels optional chains short-circuit to, innermost last.
    chains: ArenaVec<'t, u32>,
}

impl<'t> FlowBuffers<'_, 't> {
    pub(super) fn new_in(allocator: &'t Allocator) -> Self {
        Self {
            blocks: ArenaVec::new_in(&allocator),
            events: ArenaVec::new_in(&allocator),
            edges: ArenaVec::new_in(&allocator),
            labels: ArenaVec::new_in(&allocator),
            contexts: Vec::new(),
            crossings: ArenaVec::new_in(&allocator),
            parked: ArenaVec::new_in(&allocator),
            chains: ArenaVec::new_in(&allocator),
        }
    }

    fn clear(&mut self) {
        self.blocks.clear();
        self.events.clear();
        self.edges.clear();
        self.labels.clear();
        self.contexts.clear();
        self.crossings.clear();
        self.parked.clear();
        self.chains.clear();
    }
}

/// Builds the [`FlowBuffers`] of one scope, and counts the references and declarations of its
/// candidates.
pub(super) struct Walker<'w, 'a, 't> {
    scoping: &'w Scoping,
    local_index: &'w [u32],
    /// AST nodes that contain a reference to a candidate.
    marked: &'w BitSet<'t>,
    locals: &'w mut [Local],
    buffers: &'w mut FlowBuffers<'a, 't>,
    scope_id: ScopeId,
    position: u32,
    /// Number of write events recorded, to tell whether a branch wrote anything.
    writes: u32,
    aborted: bool,
    in_parameters: bool,
}

impl<'w, 'a, 't> Walker<'w, 'a, 't> {
    pub(super) fn new(
        scoping: &'w Scoping,
        local_index: &'w [u32],
        marked: &'w BitSet<'t>,
        locals: &'w mut [Local],
        buffers: &'w mut FlowBuffers<'a, 't>,
    ) -> Self {
        Self {
            scoping,
            local_index,
            marked,
            locals,
            buffers,
            scope_id: scoping.root_scope_id(),
            position: 0,
            writes: 0,
            aborted: false,
            in_parameters: false,
        }
    }

    /// Build the graph of `root`, the node of `scope_id`. Returns `false` if it can't be modeled.
    pub(super) fn walk(mut self, scope_id: ScopeId, root: Root<'a>) -> bool {
        self.buffers.clear();
        self.scope_id = scope_id;
        self.buffers.labels.push(LABEL_UNUSED);
        self.push_block();

        if let Some(params) = root.params() {
            if is_simple_parameter_list(params) {
                for param in &params.items {
                    if let BindingPattern::BindingIdentifier(id) = &param.pattern {
                        self.bind(id, true);
                    }
                }
            } else {
                self.in_parameters = true;
                self.visit_formal_parameters(params);
                self.in_parameters = false;
            }
        }
        match root {
            Root::Program(program) => self.visit_statements(&program.body),
            Root::Function(_, body) => self.visit_statements(&body.statements),
            Root::Arrow(arrow) => self.visit_arrow_function_body(&arrow.body),
            Root::StaticBlock(block) => self.visit_statements(&block.body),
        }

        let last = self.current();
        self.edge_to_label(last, EXIT_LABEL);
        let exit = self.start_block();
        self.place_label(EXIT_LABEL, exit);
        let end = self.tick();
        self.buffers.blocks[exit as usize].end = end;
        !self.aborted && self.position != u32::MAX
    }

    fn tick(&mut self) -> u32 {
        let position = self.position;
        self.position = position.saturating_add(1);
        position
    }

    fn current(&self) -> u32 {
        to_u32(self.buffers.blocks.len()) - 1
    }

    fn push_block(&mut self) -> u32 {
        let start = self.tick();
        let first_event = to_u32(self.buffers.events.len());
        self.buffers.blocks.push(Block { start, end: start, first_event, loop_end: 0 });
        self.current()
    }

    /// Close the current block and start a new one, without an edge between them.
    fn start_block(&mut self) -> u32 {
        let end = self.tick();
        let current = self.current() as usize;
        self.buffers.blocks[current].end = end;
        self.push_block()
    }

    /// Start a new block that the current block falls through to.
    fn next_block(&mut self) -> u32 {
        let from = self.current();
        let block = self.start_block();
        self.edge(from, block);
        block
    }

    fn edge(&mut self, from: u32, to: u32) {
        self.buffers.edges.push(Edge { from, to, flags: 0 });
    }

    fn exceptional_edges(&mut self, first: u32, to: u32) {
        for from in first..to {
            self.buffers.edges.push(Edge { from, to, flags: EDGE_EXCEPTIONAL });
        }
    }

    fn new_label(&mut self) -> u32 {
        self.buffers.labels.push(LABEL_UNUSED);
        to_u32(self.buffers.labels.len()) - 1
    }

    fn placed_label(&mut self, block: u32) -> u32 {
        let label = self.new_label();
        self.place_label(label, block);
        label
    }

    fn place_label(&mut self, label: u32, block: u32) {
        self.buffers.labels[label as usize] = block;
    }

    fn edge_to_label(&mut self, from: u32, label: u32) {
        let target = &mut self.buffers.labels[label as usize];
        if *target < LABEL_PENDING {
            let to = *target;
            self.edge(from, to);
        } else {
            *target = LABEL_PENDING;
            self.buffers.edges.push(Edge { from, to: label, flags: EDGE_TO_LABEL });
        }
    }

    /// Walk a branch taken from block `from`, returning the block it ends in.
    ///
    /// A branch that only reads and stays in one block is folded into `from`, as if it were
    /// always taken. That keeps the variables it reads live longer, which is safe.
    fn arm(&mut self, from: u32, walk: impl FnOnce(&mut Self)) -> u32 {
        let foldable = from == self.current();
        let blocks = self.buffers.blocks.len();
        let edges = self.buffers.edges.len();
        let labels = self.buffers.labels.len();
        let writes = self.writes;
        let block = self.start_block();
        self.edge(from, block);
        walk(self);
        if foldable && self.buffers.blocks.len() == blocks + 1 && self.writes == writes {
            self.buffers.blocks.pop();
            self.buffers.edges.truncate(edges);
            self.buffers.labels.truncate(labels);
            return from;
        }
        self.current()
    }

    /// Continue in a block reached from both `first` and `second`.
    fn join(&mut self, first: u32, second: u32) {
        let current = self.current();
        if first == current && second == current {
            return;
        }
        let block = self.start_block();
        self.edge(first, block);
        if second != first {
            self.edge(second, block);
        }
    }

    /// Walk code that may or may not run, then continue after it.
    fn conditional(&mut self, walk: impl FnOnce(&mut Self)) {
        let from = self.current();
        let end = self.arm(from, walk);
        self.join(from, end);
    }

    fn record(&mut self, local: u32, write: bool) {
        if self.locals[local as usize].revoked {
            return;
        }
        let position = self.tick();
        self.buffers.events.push(Event { position, packed: (local << 1) | u32::from(write) });
        self.writes += u32::from(write);
    }

    fn revoke(&mut self, local: u32) {
        self.locals[local as usize].revoked = true;
    }

    /// Count a reference to a candidate, returning its local and whether the reference writes.
    fn count_reference(&mut self, it: &IdentifierReference<'_>) -> Option<(u32, bool)> {
        let reference = self.scoping.get_reference(it.reference_id.get()?);
        let local = self.local_index[reference.symbol_id()?.index()];
        if local == NO_LOCAL {
            return None;
        }
        let candidate = &mut self.locals[local as usize];
        candidate.references += 1;
        if self.in_parameters {
            candidate.revoked = true;
        }
        Some((local, reference.is_write()))
    }

    fn write_reference(&mut self, it: &IdentifierReference<'_>) {
        if let Some((local, _)) = self.count_reference(it) {
            self.record(local, true);
        }
    }

    /// Count the declaration of a candidate, returning its local.
    fn count_binding(&mut self, it: &BindingIdentifier<'_>) -> Option<u32> {
        let local = self.local_index[it.symbol_id.get()?.index()];
        if local == NO_LOCAL {
            return None;
        }
        let candidate = &mut self.locals[local as usize];
        candidate.declarations += 1;
        if self.in_parameters {
            candidate.revoked = true;
        }
        Some(local)
    }

    fn bind(&mut self, it: &BindingIdentifier<'_>, write: bool) {
        if let Some(local) = self.count_binding(it)
            && write
        {
            self.record(local, true);
        }
    }

    fn bind_pattern(&mut self, pattern: &BindingPattern<'a>, write: bool) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => self.bind(id, write),
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.visit_property_key(&property.key);
                    self.bind_pattern(&property.value, write);
                }
                if let Some(rest) = &object.rest {
                    self.bind_pattern(&rest.argument, write);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.bind_pattern(element, write);
                }
                if let Some(rest) = &array.rest {
                    self.bind_pattern(&rest.argument, write);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.conditional(|walker| walker.visit_expression(&assignment.right));
                self.bind_pattern(&assignment.left, write);
            }
        }
    }

    /// Walk a destructuring assignment target, after the value assigned to it.
    fn assign_target(&mut self, target: &AssignmentTarget<'a>) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(id) => self.write_reference(id),
            match_member_expression!(AssignmentTarget) => {
                self.visit_member_expression(target.to_member_expression());
            }
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                for element in array.elements.iter().flatten() {
                    self.assign_maybe_default(element);
                }
                if let Some(rest) = &array.rest {
                    self.assign_target(&rest.target);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                for property in &object.properties {
                    match property {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                            if let Some(init) = &property.init {
                                self.conditional(|walker| walker.visit_expression(init));
                            }
                            self.write_reference(&property.binding);
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                            self.visit_property_key(&property.name);
                            self.assign_maybe_default(&property.binding);
                        }
                    }
                }
                if let Some(rest) = &object.rest {
                    self.assign_target(&rest.target);
                }
            }
            _ => walk_js::walk_assignment_target(self, target),
        }
    }

    fn assign_maybe_default(&mut self, target: &AssignmentTargetMaybeDefault<'a>) {
        let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) = target else {
            self.assign_target(target.to_assignment_target());
            return;
        };
        // A member target is evaluated before the default value, an identifier written after it.
        if let Some(member) = target.binding.as_member_expression() {
            self.visit_member_expression(member);
            self.conditional(|walker| walker.visit_expression(&target.init));
        } else {
            self.conditional(|walker| walker.visit_expression(&target.init));
            self.assign_target(&target.binding);
        }
    }

    /// Skip to the end of the innermost optional chain if the value just evaluated is nullish.
    fn short_circuit(&mut self) {
        let Some(&label) = self.buffers.chains.last() else {
            self.aborted = true;
            return;
        };
        let from = self.current();
        self.edge_to_label(from, label);
        self.next_block();
    }

    fn jump(&mut self, jump: Jump) {
        let from = self.current();
        self.jump_from(from, jump);
        self.start_block();
    }

    /// Add the edge of `jump` from block `from`: to the first `finally` clause on the way, or
    /// else to the target itself.
    fn jump_from(&mut self, from: u32, jump: Jump) {
        let floor = match jump {
            Jump::Exit => 0,
            Jump::Break(index) | Jump::Continue(index) => index as usize + 1,
        };
        for index in (floor..self.buffers.contexts.len()).rev() {
            if let JumpContext::Finally { label, .. } = self.buffers.contexts[index] {
                self.edge_to_label(from, label);
                self.buffers.crossings.push(jump);
                return;
            }
        }
        let label = match jump {
            Jump::Exit => Some(EXIT_LABEL),
            Jump::Break(index) => match self.buffers.contexts[index as usize] {
                JumpContext::Loop { break_label, .. }
                | JumpContext::Switch { break_label }
                | JumpContext::Labeled { break_label, .. } => Some(break_label),
                JumpContext::Finally { .. } => None,
            },
            Jump::Continue(index) => match self.buffers.contexts[index as usize] {
                JumpContext::Loop { continue_label, .. } => Some(continue_label),
                _ => None,
            },
        };
        match label {
            Some(label) => self.edge_to_label(from, label),
            None => self.aborted = true,
        }
    }

    fn walk_loop_body(&mut self, break_label: u32, continue_label: u32, body: &Statement<'a>) {
        self.buffers.contexts.push(JumpContext::Loop { break_label, continue_label });
        self.visit_statement(body);
        self.buffers.contexts.pop();
    }

    /// Start the block after a loop, which `exit_from` falls through to.
    fn finish_loop(&mut self, header: u32, break_label: u32, exit_from: Option<u32>) {
        let after = self.start_block();
        if let Some(from) = exit_from {
            self.edge(from, after);
        }
        self.place_label(break_label, after);
        let loop_end = self.buffers.blocks[after as usize].start - 1;
        self.buffers.blocks[header as usize].loop_end = loop_end;
    }

    fn walk_for_in_of(
        &mut self,
        left: &ForStatementLeft<'a>,
        right: &Expression<'a>,
        body: &Statement<'a>,
    ) {
        // Annex B: `for (var x = init in object)`.
        if let ForStatementLeft::VariableDeclaration(declaration) = left
            && declaration.declarations.iter().any(|declarator| declarator.init.is_some())
        {
            self.aborted = true;
            return;
        }
        self.visit_expression(right);
        let header = self.next_block();
        let break_label = self.new_label();
        let continue_label = self.placed_label(header);
        self.next_block();
        match left {
            ForStatementLeft::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    self.bind_pattern(&declarator.id, true);
                }
            }
            _ => self.assign_target(left.to_assignment_target()),
        }
        self.walk_loop_body(break_label, continue_label, body);
        let latch = self.current();
        self.edge(latch, header);
        self.finish_loop(header, break_label, Some(header));
    }
}

impl<'a> VisitJs<'a> for Walker<'_, 'a, '_> {
    /// Skip expressions without references to candidates: they record no events. Within an
    /// optional chain, an expression may still short circuit the chain.
    fn visit_expression(&mut self, it: &Expression<'a>) {
        if self.buffers.chains.is_empty() && !self.marked.has_bit(it.node_id().index()) {
            return;
        }
        walk_js::walk_expression(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if let Some((local, is_write)) = self.count_reference(it) {
            if is_write {
                self.revoke(local);
            } else {
                self.record(local, false);
            }
        }
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        if let Some(local) = self.count_binding(it) {
            self.revoke(local);
        }
    }

    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.visit_expression(&it.left);
        self.conditional(|walker| walker.visit_expression(&it.right));
    }

    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        self.visit_expression(&it.test);
        let from = self.current();
        let consequent = self.arm(from, |walker| walker.visit_expression(&it.consequent));
        let alternate = self.arm(from, |walker| walker.visit_expression(&it.alternate));
        self.join(consequent, alternate);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        match &it.left {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                if it.operator.is_assign() {
                    self.visit_expression(&it.right);
                    self.write_reference(id);
                    return;
                }
                let local = self.count_reference(id).map(|(local, _)| local);
                if let Some(local) = local {
                    self.record(local, false);
                }
                let write = |walker: &mut Self| {
                    walker.visit_expression(&it.right);
                    if let Some(local) = local {
                        walker.record(local, true);
                    }
                };
                if it.operator.is_logical() {
                    self.conditional(write);
                } else {
                    write(self);
                }
            }
            match_member_expression!(AssignmentTarget) => {
                self.visit_member_expression(it.left.to_member_expression());
                if it.operator.is_logical() {
                    self.conditional(|walker| walker.visit_expression(&it.right));
                } else {
                    self.visit_expression(&it.right);
                }
            }
            AssignmentTarget::ArrayAssignmentTarget(_)
            | AssignmentTarget::ObjectAssignmentTarget(_) => {
                self.visit_expression(&it.right);
                self.assign_target(&it.left);
            }
            _ => walk_js::walk_assignment_expression(self, it),
        }
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        match &it.argument {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                if let Some((local, _)) = self.count_reference(id) {
                    self.record(local, false);
                    self.record(local, true);
                }
            }
            match_member_expression!(SimpleAssignmentTarget) => {
                self.visit_member_expression(it.argument.to_member_expression());
            }
            _ => walk_js::walk_update_expression(self, it),
        }
    }

    fn visit_chain_expression(&mut self, it: &ChainExpression<'a>) {
        let blocks = self.buffers.blocks.len();
        let edges = self.buffers.edges.len();
        let labels = self.buffers.labels.len();
        let writes = self.writes;
        let label = self.new_label();
        self.buffers.chains.push(label);
        walk_js::walk_chain_expression(self, it);
        self.buffers.chains.pop();
        if self.writes == writes {
            // Without writes, treating every link of the chain as evaluated is safe.
            self.buffers.blocks.truncate(blocks);
            self.buffers.edges.truncate(edges);
            self.buffers.labels.truncate(labels);
        } else if self.buffers.labels[label as usize] == LABEL_PENDING {
            let end = self.next_block();
            self.place_label(label, end);
        }
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        self.visit_expression(&it.object);
        if it.optional {
            self.short_circuit();
        }
    }

    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        self.visit_expression(&it.object);
        if it.optional {
            self.short_circuit();
        }
        self.visit_expression(&it.expression);
    }

    fn visit_private_field_expression(&mut self, it: &PrivateFieldExpression<'a>) {
        self.visit_expression(&it.object);
        if it.optional {
            self.short_circuit();
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.visit_expression(&it.callee);
        if it.optional {
            self.short_circuit();
        }
        self.visit_arguments(&it.arguments);
    }

    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        let is_var = it.kind.is_var();
        for declarator in &it.declarations {
            if let Some(init) = &declarator.init {
                self.visit_expression(init);
                self.bind_pattern(&declarator.id, true);
                continue;
            }
            if is_var
                && let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(symbol_id) = id.symbol_id.get()
            {
                let local = self.local_index[symbol_id.index()];
                if local != NO_LOCAL {
                    self.locals[local as usize].bare = true;
                }
            }
            self.bind_pattern(&declarator.id, false);
        }
    }

    fn visit_function(&mut self, it: &Function<'a>, _flags: ScopeFlags) {
        // Annex B: evaluating a function declaration in a block also assigns it to the `var` of
        // the same name.
        if it.is_declaration()
            && let Some(id) = &it.id
            && let Some(symbol_id) = id.symbol_id.get()
            && self.scoping.symbol_scope_id(symbol_id) != self.scope_id
            && let Some(binding) = self.scoping.get_binding(self.scope_id, id.name)
        {
            let local = self.local_index[binding.index()];
            if local != NO_LOCAL {
                self.revoke(local);
            }
        }
    }

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _it: &Class<'a>) {}

    fn visit_with_statement(&mut self, _it: &WithStatement<'a>) {
        self.aborted = true;
    }

    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        // An exported binding is observable by importers whenever it is written.
        for specifier in &it.specifiers {
            if let ModuleExportName::IdentifierReference(id) = &specifier.local
                && let Some((local, _)) = self.count_reference(id)
            {
                self.revoke(local);
            }
        }
    }

    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        self.visit_expression(&it.test);
        let from = self.current();
        let consequent = self.arm(from, |walker| walker.visit_statement(&it.consequent));
        if let Some(alternate) = &it.alternate {
            let alternate = self.arm(from, |walker| walker.visit_statement(alternate));
            self.join(consequent, alternate);
        } else {
            self.join(from, consequent);
        }
    }

    fn visit_while_statement(&mut self, it: &WhileStatement<'a>) {
        let header = self.next_block();
        self.visit_expression(&it.test);
        let test_end = self.current();
        let break_label = self.new_label();
        let continue_label = self.placed_label(header);
        let body = self.start_block();
        self.edge(test_end, body);
        self.walk_loop_body(break_label, continue_label, &it.body);
        let latch = self.current();
        self.edge(latch, header);
        self.finish_loop(header, break_label, Some(test_end));
    }

    fn visit_do_while_statement(&mut self, it: &DoWhileStatement<'a>) {
        let header = self.next_block();
        let break_label = self.new_label();
        let continue_label = self.new_label();
        self.walk_loop_body(break_label, continue_label, &it.body);
        let test = self.next_block();
        self.place_label(continue_label, test);
        self.visit_expression(&it.test);
        let test_end = self.current();
        self.edge(test_end, header);
        self.finish_loop(header, break_label, Some(test_end));
    }

    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        match &it.init {
            Some(ForStatementInit::VariableDeclaration(declaration)) => {
                self.visit_variable_declaration(declaration);
            }
            Some(init) => self.visit_expression(init.to_expression()),
            None => {}
        }
        let header = self.next_block();
        if let Some(test) = &it.test {
            self.visit_expression(test);
        }
        let test_end = self.current();
        let break_label = self.new_label();
        let continue_label = self.new_label();
        let body = self.start_block();
        self.edge(test_end, body);
        self.walk_loop_body(break_label, continue_label, &it.body);
        let update = self.next_block();
        self.place_label(continue_label, update);
        if let Some(update) = &it.update {
            self.visit_expression(update);
        }
        let latch = self.current();
        self.edge(latch, header);
        self.finish_loop(header, break_label, it.test.is_some().then_some(test_end));
    }

    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        self.walk_for_in_of(&it.left, &it.right, &it.body);
    }

    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        self.walk_for_in_of(&it.left, &it.right, &it.body);
    }

    fn visit_switch_statement(&mut self, it: &SwitchStatement<'a>) {
        self.visit_expression(&it.discriminant);
        let first_label = to_u32(self.buffers.labels.len());
        for _ in &it.cases {
            self.new_label();
        }
        let break_label = self.new_label();

        let mut test_end = self.current();
        let mut default_label = break_label;
        let mut first_test = true;
        for (label, case) in (first_label..).zip(&it.cases) {
            let Some(test) = &case.test else {
                default_label = label;
                continue;
            };
            if !first_test {
                let block = self.start_block();
                self.edge(test_end, block);
            }
            first_test = false;
            self.visit_expression(test);
            test_end = self.current();
            self.edge_to_label(test_end, label);
        }
        self.edge_to_label(test_end, default_label);

        self.buffers.contexts.push(JumpContext::Switch { break_label });
        let mut previous = None;
        for (label, case) in (first_label..).zip(&it.cases) {
            let block = self.start_block();
            if let Some(previous) = previous {
                self.edge(previous, block);
            }
            self.place_label(label, block);
            self.visit_statements(&case.consequent);
            previous = Some(self.current());
        }
        self.buffers.contexts.pop();
        let after = self.start_block();
        if let Some(previous) = previous {
            self.edge(previous, after);
        }
        self.place_label(break_label, after);
    }

    fn visit_labeled_statement(&mut self, it: &LabeledStatement<'a>) {
        let break_label = self.new_label();
        self.buffers.contexts.push(JumpContext::Labeled { name: it.label.name, break_label });
        self.visit_statement(&it.body);
        self.buffers.contexts.pop();
        if self.buffers.labels[break_label as usize] == LABEL_PENDING {
            let after = self.next_block();
            self.place_label(break_label, after);
        }
    }

    fn visit_break_statement(&mut self, it: &BreakStatement<'a>) {
        let contexts = &self.buffers.contexts;
        let target = match &it.label {
            None => contexts.iter().rposition(|context| {
                matches!(context, JumpContext::Loop { .. } | JumpContext::Switch { .. })
            }),
            Some(label) => contexts.iter().rposition(|context| {
                matches!(context, JumpContext::Labeled { name, .. } if *name == label.name)
            }),
        };
        match target {
            Some(index) => self.jump(Jump::Break(to_u32(index))),
            None => self.aborted = true,
        }
    }

    fn visit_continue_statement(&mut self, it: &ContinueStatement<'a>) {
        let contexts = &self.buffers.contexts;
        let is_loop = |context: &JumpContext<'_>| matches!(context, JumpContext::Loop { .. });
        let target = match &it.label {
            None => contexts.iter().rposition(is_loop),
            Some(label) => contexts
                .iter()
                .rposition(|context| {
                    matches!(context, JumpContext::Labeled { name, .. } if *name == label.name)
                })
                .and_then(|labeled| {
                    contexts[labeled + 1..].iter().position(is_loop).map(|index| labeled + 1 + index)
                }),
        };
        match target {
            Some(index) => self.jump(Jump::Continue(to_u32(index))),
            None => self.aborted = true,
        }
    }

    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        if let Some(argument) = &it.argument {
            self.visit_expression(argument);
        }
        self.jump(Jump::Exit);
    }

    fn visit_throw_statement(&mut self, it: &ThrowStatement<'a>) {
        self.visit_expression(&it.argument);
        // The handler, if any, is reached through the exceptional edges of the `try` block.
        self.start_block();
    }

    fn visit_try_statement(&mut self, it: &TryStatement<'a>) {
        let finally_label = it.finalizer.as_ref().map(|_| {
            let label = self.new_label();
            let crossing_start = to_u32(self.buffers.crossings.len());
            self.buffers.contexts.push(JumpContext::Finally { label, crossing_start });
            label
        });

        let try_first = self.next_block();
        self.visit_block_statement(&it.block);
        let try_end = self.current();

        let catch_end = it.handler.as_ref().map(|handler| {
            let catch_entry = self.start_block();
            self.exceptional_edges(try_first, catch_entry);
            if let Some(param) = &handler.param {
                self.bind_pattern(&param.pattern, true);
            }
            self.visit_block_statement(&handler.body);
            self.current()
        });

        let (Some(finalizer), Some(finally_label)) = (&it.finalizer, finally_label) else {
            let after = self.start_block();
            self.edge(try_end, after);
            if let Some(catch_end) = catch_end {
                self.edge(catch_end, after);
            }
            return;
        };
        let Some(JumpContext::Finally { crossing_start, .. }) = self.buffers.contexts.pop() else {
            self.aborted = true;
            return;
        };
        let crossing_start = crossing_start as usize;
        let parked_start = self.buffers.parked.len();
        let FlowBuffers { crossings, parked, .. } = &mut *self.buffers;
        parked.extend(crossings.drain(crossing_start..));

        let finally_entry = self.start_block();
        self.place_label(finally_label, finally_entry);
        self.edge(try_end, finally_entry);
        if let Some(catch_end) = catch_end {
            self.edge(catch_end, finally_entry);
        }
        self.exceptional_edges(try_first, finally_entry);
        self.visit_block_statement(finalizer);
        let finally_end = self.current();
        for index in parked_start..self.buffers.parked.len() {
            let jump = self.buffers.parked[index];
            self.jump_from(finally_end, jump);
        }
        self.buffers.parked.truncate(parked_start);
        let after = self.start_block();
        self.edge(finally_end, after);
    }
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
