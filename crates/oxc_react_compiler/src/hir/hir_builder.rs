/// HIR Builder — infrastructure for constructing the HIR control-flow graph.
///
/// Port of `HIR/HIRBuilder.ts` from the React Compiler.
///
/// The `HirBuilder` is a helper class for constructing a control-flow graph
/// during the lowering from AST to HIR. It manages basic blocks, scopes
/// (loops, switches, labels), and exception handling.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::{CompilerError, SourceLocation, GENERATED_SOURCE};

use super::{
    environment::Environment,
    hir_types::{
        BasicBlock, BlockId, BlockKind, Effect, GotoVariant, Hir, Identifier, IdentifierId,
        Instruction, InstructionId, MutableRange, Place, Terminal,
    },
    types::make_type,
};

/// A work-in-progress block that does not yet have a terminal.
#[derive(Debug)]
pub struct WipBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub kind: BlockKind,
}

/// Create a new WipBlock with the given id and kind.
fn new_block(id: BlockId, kind: BlockKind) -> WipBlock {
    WipBlock { id, kind, instructions: Vec::new() }
}

/// A scope entry for tracking loops, switches, and labels.
#[derive(Debug)]
enum Scope {
    Loop(LoopScope),
    Switch(SwitchScope),
    Label(LabelScope),
}

#[derive(Debug)]
struct LoopScope {
    label: Option<String>,
    continue_block: BlockId,
    break_block: BlockId,
}

#[derive(Debug)]
struct SwitchScope {
    label: Option<String>,
    break_block: BlockId,
}

#[derive(Debug)]
struct LabelScope {
    label: String,
    break_block: BlockId,
}

/// Determines how instructions should be constructed to preserve exception semantics.
#[derive(Debug, Clone)]
pub enum ExceptionsMode {
    /// Mode for code not covered by explicit exception handling.
    ThrowExceptions,
    /// Mode for code covered by try/catch — requires modeling control flow to handler.
    CatchExceptions { handler: BlockId },
}

/// Helper class for constructing a HIR control-flow graph.
pub struct HirBuilder {
    completed: FxHashMap<BlockId, BasicBlock>,
    current: WipBlock,
    entry: BlockId,
    scopes: Vec<Scope>,
    env: Environment,
    exception_handler_stack: Vec<BlockId>,
    /// Accumulated errors during lowering.
    pub errors: CompilerError,
    /// Counts the number of `fbt` tag parents of the current node.
    pub fbt_depth: u32,
}

impl HirBuilder {
    /// Create a new `HirBuilder` with the given environment.
    pub fn new(env: Environment, entry_block_kind: Option<BlockKind>) -> Self {
        let entry = env.next_block_id_value();
        let current = new_block(BlockId(entry), entry_block_kind.unwrap_or(BlockKind::Block));
        Self {
            completed: FxHashMap::default(),
            current,
            entry: BlockId(entry),
            scopes: Vec::new(),
            env,
            exception_handler_stack: Vec::new(),
            errors: CompilerError::new(),
            fbt_depth: 0,
        }
    }

    /// Get a reference to the environment.
    pub fn environment(&self) -> &Environment {
        &self.env
    }

    /// Get a mutable reference to the environment.
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.env
    }

    /// Get the kind of the current block.
    pub fn current_block_kind(&self) -> BlockKind {
        self.current.kind
    }

    /// Create a new temporary identifier.
    pub fn make_temporary(&mut self, loc: SourceLocation) -> Identifier {
        let id = self.env.next_identifier_id();
        make_temporary_identifier(id, loc)
    }

    /// Push an instruction onto the current block.
    ///
    /// If inside a try/catch, also generates a `maybe-throw` terminal and continuation block.
    pub fn push(&mut self, instruction: Instruction) {
        let loc = instruction.loc;
        self.current.instructions.push(instruction);

        if let Some(&handler) = self.exception_handler_stack.last() {
            let continuation = self.reserve(self.current.kind);
            self.terminate_with_continuation(
                Terminal::MaybeThrow(super::hir_types::MaybeThrowTerminal {
                    continuation: continuation.id,
                    handler: Some(handler),
                    id: InstructionId(0),
                    loc,
                }),
                continuation,
            );
        }
    }

    /// Execute a callback inside a try/catch exception handler scope.
    pub fn enter_try_catch(&mut self, handler: BlockId, f: impl FnOnce(&mut Self)) {
        self.exception_handler_stack.push(handler);
        f(self);
        self.exception_handler_stack.pop();
    }

    /// Resolve the current throw handler (innermost try/catch).
    pub fn resolve_throw_handler(&self) -> Option<BlockId> {
        self.exception_handler_stack.last().copied()
    }

    /// Finish building the HIR and return the completed control-flow graph.
    ///
    /// # Errors
    /// Returns an error if the builder encountered errors during construction.
    pub fn build(self) -> Result<Hir, CompilerError> {
        if self.errors.has_any_errors() {
            return Err(self.errors);
        }
        Ok(Hir { entry: self.entry, blocks: self.completed })
    }

    /// Terminate the current block with the given terminal, and start a new block.
    ///
    /// Returns the ID of the terminated block.
    pub fn terminate(&mut self, terminal: Terminal, next_block_kind: Option<BlockKind>) -> BlockId {
        let WipBlock { id, kind, instructions } = std::mem::replace(
            &mut self.current,
            new_block(BlockId(0), BlockKind::Block), // placeholder
        );
        let block_id = id;
        self.completed.insert(
            block_id,
            BasicBlock {
                kind,
                id: block_id,
                instructions,
                terminal,
                preds: FxHashSet::default(),
                phis: FxHashSet::default(),
            },
        );

        if let Some(next_kind) = next_block_kind {
            let next_id = self.env.next_block_id();
            self.current = new_block(next_id, next_kind);
        }
        block_id
    }

    /// Terminate the current block with the given terminal, and set a previously
    /// reserved block as the new current block.
    pub fn terminate_with_continuation(&mut self, terminal: Terminal, continuation: WipBlock) {
        let WipBlock { id, kind, instructions } = std::mem::replace(&mut self.current, continuation);
        self.completed.insert(
            id,
            BasicBlock {
                kind,
                id,
                instructions,
                terminal,
                preds: FxHashSet::default(),
                phis: FxHashSet::default(),
            },
        );
    }

    /// Reserve a block so that it can be referenced prior to construction.
    pub fn reserve(&mut self, kind: BlockKind) -> WipBlock {
        new_block(self.env.next_block_id(), kind)
    }

    /// Save a previously reserved block as completed.
    pub fn complete(&mut self, block: WipBlock, terminal: Terminal) {
        let WipBlock { id, kind, instructions } = block;
        self.completed.insert(
            id,
            BasicBlock {
                kind,
                id,
                instructions,
                terminal,
                preds: FxHashSet::default(),
                phis: FxHashSet::default(),
            },
        );
    }

    /// Sets the given WIP block as the current block, executes the callback to populate it,
    /// and then resets to the previous block.
    pub fn enter_reserved(&mut self, wip: WipBlock, f: impl FnOnce(&mut Self) -> Terminal) {
        let prev = std::mem::replace(&mut self.current, wip);
        let terminal = f(self);
        let WipBlock { id, kind, instructions } = std::mem::replace(&mut self.current, prev);
        self.completed.insert(
            id,
            BasicBlock {
                kind,
                id,
                instructions,
                terminal,
                preds: FxHashSet::default(),
                phis: FxHashSet::default(),
            },
        );
    }

    /// Create a new block and execute the callback inside it.
    pub fn enter(
        &mut self,
        next_block_kind: BlockKind,
        f: impl FnOnce(&mut Self, BlockId) -> Terminal,
    ) -> BlockId {
        let wip = self.reserve(next_block_kind);
        let wip_id = wip.id;
        self.enter_reserved(wip, |builder| f(builder, wip_id));
        wip_id
    }

    /// Execute inside a labeled scope.
    pub fn label<T>(
        &mut self,
        label: String,
        break_block: BlockId,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.scopes.push(Scope::Label(LabelScope { label, break_block }));
        let value = f(self);
        self.scopes.pop();
        value
    }

    /// Execute inside a switch scope.
    pub fn switch<T>(
        &mut self,
        label: Option<String>,
        break_block: BlockId,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.scopes.push(Scope::Switch(SwitchScope { label, break_block }));
        let value = f(self);
        self.scopes.pop();
        value
    }

    /// Execute inside a loop scope.
    pub fn enter_loop<T>(
        &mut self,
        label: Option<String>,
        continue_block: BlockId,
        break_block: BlockId,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.scopes.push(Scope::Loop(LoopScope { label, continue_block, break_block }));
        let value = f(self);
        self.scopes.pop();
        value
    }

    /// Lookup the block target for a break statement.
    ///
    /// # Errors
    /// Returns an error if no loop or switch is in scope.
    pub fn lookup_break(&self, label: Option<&str>) -> Result<BlockId, CompilerError> {
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Loop(s) => {
                    if label.is_none() || label == s.label.as_deref() {
                        return Ok(s.break_block);
                    }
                }
                Scope::Switch(s) => {
                    if label.is_none() || label == s.label.as_deref() {
                        return Ok(s.break_block);
                    }
                }
                Scope::Label(s) => {
                    if label == Some(s.label.as_str()) {
                        return Ok(s.break_block);
                    }
                }
            }
        }
        Err(CompilerError::invariant(
            "Expected a loop or switch to be in scope",
            None,
            GENERATED_SOURCE,
        ))
    }

    /// Lookup the block target for a continue statement.
    ///
    /// # Errors
    /// Returns an error if no loop is in scope.
    pub fn lookup_continue(&self, label: Option<&str>) -> Result<BlockId, CompilerError> {
        for scope in self.scopes.iter().rev() {
            if let Scope::Loop(s) = scope {
                if label.is_none() || label == s.label.as_deref() {
                    return Ok(s.continue_block);
                }
            }
        }
        Err(CompilerError::invariant(
            "Expected a loop to be in scope for continue",
            None,
            GENERATED_SOURCE,
        ))
    }
}

// =====================================================================================
// Standalone helper functions (ported from module-level functions in HIRBuilder.ts)
// =====================================================================================

/// Create a temporary Place.
pub fn create_temporary_place(env: &mut Environment, loc: SourceLocation) -> Place {
    Place {
        identifier: make_temporary_identifier(env.next_identifier_id(), loc),
        reactive: false,
        effect: Effect::Unknown,
        loc: GENERATED_SOURCE,
    }
}

/// Create a temporary identifier.
pub fn make_temporary_identifier(id: IdentifierId, loc: SourceLocation) -> Identifier {
    Identifier {
        id,
        declaration_id: super::hir_types::DeclarationId(id.0),
        name: None,
        mutable_range: MutableRange::default(),
        scope: None,
        type_: make_type(),
        loc,
    }
}

/// Mark instruction IDs on all instructions and terminals in the HIR.
pub fn mark_instruction_ids(func: &mut Hir) {
    let mut id = 0u32;
    for block in func.blocks.values_mut() {
        for instr in &mut block.instructions {
            id += 1;
            instr.id = InstructionId(id);
        }
        id += 1;
        set_terminal_id(&mut block.terminal, InstructionId(id));
    }
}

/// Mark predecessor blocks for all blocks in the HIR.
pub fn mark_predecessors(func: &mut Hir) {
    // Clear all predecessors
    for block in func.blocks.values_mut() {
        block.preds.clear();
    }

    // Collect (block_id, predecessor_id) pairs
    let mut pred_edges: Vec<(BlockId, BlockId)> = Vec::new();
    let mut visited = FxHashSet::default();
    let mut stack = vec![func.entry];

    while let Some(block_id) = stack.pop() {
        if !visited.insert(block_id) {
            continue;
        }
        let block = match func.blocks.get(&block_id) {
            Some(b) => b,
            None => continue,
        };
        for successor in each_terminal_successor(&block.terminal) {
            pred_edges.push((successor, block_id));
            stack.push(successor);
        }
    }

    // Apply predecessor edges
    for (block_id, pred_id) in pred_edges {
        if let Some(block) = func.blocks.get_mut(&block_id) {
            block.preds.insert(pred_id);
        }
    }
}

/// Remove unnecessary try/catch terminals where the handler is unreachable.
pub fn remove_unnecessary_try_catch(func: &mut Hir) {
    let block_ids: Vec<BlockId> = func.blocks.keys().copied().collect();
    for block_id in block_ids {
        let should_convert = {
            let block = func.blocks.get(&block_id);
            match block.map(|b| &b.terminal) {
                Some(Terminal::Try(t)) => !func.blocks.contains_key(&t.handler),
                _ => false,
            }
        };

        if should_convert {
            let block = func.blocks.get_mut(&block_id);
            if let Some(block) = block {
                if let Terminal::Try(ref t) = block.terminal {
                    let target = t.block;
                    let loc = t.loc;
                    block.terminal = Terminal::Goto(super::hir_types::GotoTerminal {
                        id: InstructionId(0),
                        block: target,
                        variant: GotoVariant::Break,
                        loc,
                    });
                }
            }
        }
    }
}

// =====================================================================================
// Terminal helper functions
// =====================================================================================

/// Set the instruction ID on a terminal.
fn set_terminal_id(terminal: &mut Terminal, id: InstructionId) {
    match terminal {
        Terminal::Unsupported(t) => t.id = id,
        Terminal::Unreachable(t) => t.id = id,
        Terminal::Throw(t) => t.id = id,
        Terminal::Return(t) => t.id = id,
        Terminal::Goto(t) => t.id = id,
        Terminal::If(t) => t.id = id,
        Terminal::Branch(t) => t.id = id,
        Terminal::Switch(t) => t.id = id,
        Terminal::For(t) => t.id = id,
        Terminal::ForOf(t) => t.id = id,
        Terminal::ForIn(t) => t.id = id,
        Terminal::DoWhile(t) => t.id = id,
        Terminal::While(t) => t.id = id,
        Terminal::Logical(t) => t.id = id,
        Terminal::Ternary(t) => t.id = id,
        Terminal::Optional(t) => t.id = id,
        Terminal::Label(t) => t.id = id,
        Terminal::Sequence(t) => t.id = id,
        Terminal::MaybeThrow(t) => t.id = id,
        Terminal::Try(t) => t.id = id,
        Terminal::Scope(t) => t.id = id,
        Terminal::PrunedScope(t) => t.id = id,
    }
}

/// Iterate over all successor block IDs of a terminal.
pub fn each_terminal_successor(terminal: &Terminal) -> Vec<BlockId> {
    let mut successors = Vec::new();
    match terminal {
        Terminal::Unsupported(_) | Terminal::Unreachable(_) | Terminal::Throw(_) => {}
        Terminal::Return(_) => {}
        Terminal::Goto(t) => {
            successors.push(t.block);
        }
        Terminal::If(t) => {
            successors.push(t.consequent);
            successors.push(t.alternate);
        }
        Terminal::Branch(t) => {
            successors.push(t.consequent);
            successors.push(t.alternate);
        }
        Terminal::Switch(t) => {
            for case in &t.cases {
                successors.push(case.block);
            }
        }
        Terminal::For(t) => {
            successors.push(t.init);
            successors.push(t.test);
            if let Some(update) = t.update {
                successors.push(update);
            }
            successors.push(t.r#loop);
        }
        Terminal::ForOf(t) => {
            successors.push(t.init);
            successors.push(t.test);
            successors.push(t.r#loop);
        }
        Terminal::ForIn(t) => {
            successors.push(t.init);
            successors.push(t.r#loop);
        }
        Terminal::DoWhile(t) => {
            successors.push(t.r#loop);
            successors.push(t.test);
        }
        Terminal::While(t) => {
            successors.push(t.test);
            successors.push(t.r#loop);
        }
        Terminal::Logical(t) => {
            successors.push(t.test);
        }
        Terminal::Ternary(t) => {
            successors.push(t.test);
        }
        Terminal::Optional(t) => {
            successors.push(t.test);
        }
        Terminal::Label(t) => {
            successors.push(t.block);
        }
        Terminal::Sequence(t) => {
            successors.push(t.block);
        }
        Terminal::MaybeThrow(t) => {
            successors.push(t.continuation);
            if let Some(handler) = t.handler {
                successors.push(handler);
            }
        }
        Terminal::Try(t) => {
            successors.push(t.block);
            successors.push(t.handler);
        }
        Terminal::Scope(t) => {
            successors.push(t.block);
        }
        Terminal::PrunedScope(t) => {
            successors.push(t.block);
        }
    }
    successors
}
