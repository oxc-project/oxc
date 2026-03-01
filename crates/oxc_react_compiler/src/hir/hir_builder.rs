/// HIR Builder — infrastructure for constructing the HIR control-flow graph.
///
/// Port of `HIR/HIRBuilder.ts` from the React Compiler.
///
/// The `HirBuilder` is a helper class for constructing a control-flow graph
/// during the lowering from AST to HIR. It manages basic blocks, scopes
/// (loops, switches, labels), and exception handling.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::{CompilerError, GENERATED_SOURCE, SourceLocation};

use super::{
    environment::Environment,
    find_context_identifiers::ContextIdentifiers,
    hir_types::{
        BasicBlock, BlockId, BlockKind, BlockMap, DeclarationId, Effect, GotoVariant, Hir,
        Identifier, IdentifierId, IdentifierName, Instruction, InstructionId, MutableRange,
        NonLocalBinding, Place, Terminal, UnreachableTerminal,
    },
    types::make_type,
};

/// The result of resolving an identifier reference.
///
/// Corresponds to the `VariableBinding` type in the TS `HIRBuilder.ts`.
pub enum VariableBinding {
    /// A local variable binding (declared in this function scope).
    Identifier { identifier: Identifier, binding_kind: BindingKind },
    /// A global or module-scope variable.
    NonLocal(NonLocalBinding),
}

/// The kind of a local binding (corresponds to Babel's `Binding.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Const,
    Let,
    Var,
    Param,
    Function,
}

/// Entry in the bindings map.
struct BindingEntry {
    /// A unique key to distinguish same-named variables in different scopes.
    /// We use a monotonically increasing counter for this.
    declaration_key: u32,
    /// The HIR Identifier for this binding.
    identifier: Identifier,
    /// The binding kind.
    kind: BindingKind,
    /// Whether this entry was eagerly pre-declared (before the actual declaration
    /// was processed). Pre-declared entries are created when an inner function
    /// captures a variable that hasn't been declared yet in the sequential lowering
    /// (e.g., hoisted references). When the real declaration is processed later,
    /// the pre-declared entry is upgraded to a normal entry.
    pre_declared: bool,
}

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
    completed: BlockMap,
    current: WipBlock,
    entry: BlockId,
    scopes: Vec<Scope>,
    env: Environment,
    exception_handler_stack: Vec<BlockId>,
    /// Accumulated errors during lowering.
    pub errors: CompilerError,
    /// Counts the number of `fbt` tag parents of the current node.
    pub fbt_depth: u32,

    /// Maps binding names to their HIR identifiers and metadata.
    ///
    /// This corresponds to `#bindings` in the TS `HIRBuilder`. When the same name
    /// is used in different scopes (shadowing), each gets a unique entry with a
    /// suffixed name (e.g., `x`, `x_0`, `x_1`).
    bindings: FxHashMap<String, BindingEntry>,

    /// Monotonically increasing counter for unique binding keys.
    next_binding_key: u32,

    /// The set of context identifiers (variables captured by inner closures).
    context_identifiers: ContextIdentifiers,

    /// The set of identifiers that have been hoisted (declared before their
    /// textual position because they are referenced by inner functions).
    /// Corresponds to `#hoistedIdentifiers` in the TS `Environment`.
    hoisted_identifiers: FxHashSet<String>,

    /// All binding names that will be declared in this function's scope.
    ///
    /// Pre-collected from the AST before lowering begins, this set includes
    /// ALL variable/function declarations in the function body, regardless of
    /// their position. This mimics Babel's scope analysis which knows about all
    /// bindings before any lowering occurs.
    ///
    /// Used by `gather_captured_context` to correctly identify captured variables
    /// even when the capturing function appears before the variable declaration
    /// in the source (e.g., `const foo = () => bar; const bar = 3;`).
    scope_binding_names: FxHashSet<String>,

    /// Stack for tracking block-scoped binding changes.
    ///
    /// When entering a block scope (e.g., an if-branch body), bindings added
    /// inside that scope need to be removed upon exiting to avoid polluting the
    /// outer scope. Each frame stores (binding_key, previous_value) pairs:
    /// - If previous_value is `Some(entry)`, restore that entry on scope exit.
    /// - If previous_value is `None`, remove the key entirely on scope exit.
    ///
    /// Only tracks bindings that are block-scoped (const/let), not var declarations.
    binding_scope_stack: Vec<Vec<(String, Option<BindingEntry>)>>,

    /// Module-scope bindings (imports and other module-level declarations).
    ///
    /// This corresponds to checking `parentFunction.scope.parent.getBinding()`
    /// in the TS `HIRBuilder.ts`. When the HIR builder cannot find a name in
    /// local `bindings`, it checks this map before falling back to
    /// `NonLocalBinding::Global`. This enables correct resolution of renamed
    /// imports (e.g., `import {useState as useReactState} from 'react'`).
    outer_bindings: FxHashMap<String, NonLocalBinding>,
}

impl HirBuilder {
    /// Create a new `HirBuilder` with the given environment and context identifiers.
    pub fn new(
        mut env: Environment,
        entry_block_kind: Option<BlockKind>,
        context_identifiers: ContextIdentifiers,
        outer_bindings: FxHashMap<String, NonLocalBinding>,
    ) -> Self {
        let entry = env.next_block_id();
        let current = new_block(entry, entry_block_kind.unwrap_or(BlockKind::Block));
        Self {
            completed: BlockMap::default(),
            current,
            entry,
            scopes: Vec::new(),
            env,
            exception_handler_stack: Vec::new(),
            errors: CompilerError::new(),
            fbt_depth: 0,
            bindings: FxHashMap::default(),
            next_binding_key: 0,
            context_identifiers,
            hoisted_identifiers: FxHashSet::default(),
            scope_binding_names: FxHashSet::default(),
            binding_scope_stack: Vec::new(),
            outer_bindings,
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

    /// Get the module-scope outer bindings map.
    pub fn outer_bindings(&self) -> &FxHashMap<String, NonLocalBinding> {
        &self.outer_bindings
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

    // =====================================================================================
    // Binding resolution (port of HIRBuilder.ts resolveBinding / resolveIdentifier)
    // =====================================================================================

    /// Resolve a binding for a declared variable name, creating or returning an
    /// existing HIR `Identifier`.
    ///
    /// This corresponds to `resolveBinding()` in the TS `HIRBuilder.ts`.
    /// It ensures that each unique declaration gets a unique HIR Identifier,
    /// even when the same name is used in different scopes (shadowing is handled
    /// by appending `_0`, `_1`, etc.).
    ///
    /// `declaration_key` is a unique key identifying this specific declaration
    /// (distinct from other declarations with the same name).
    pub fn resolve_binding(
        &mut self,
        name: &str,
        declaration_key: u32,
        loc: SourceLocation,
    ) -> Identifier {
        // Check if we already have a binding for this name
        if let Some(entry) = self.bindings.get(name) {
            if entry.declaration_key == declaration_key {
                return entry.identifier.clone();
            }
            // If the existing entry was pre-declared (eagerly created for hoisted
            // references), upgrade it to a normal entry with the real declaration key.
            // This ensures that the pre-declared identifier is reused when the actual
            // declaration is processed later.
            if entry.pre_declared {
                let identifier = entry.identifier.clone();
                if let Some(entry) = self.bindings.get_mut(name) {
                    entry.declaration_key = declaration_key;
                    entry.pre_declared = false;
                }
                return identifier;
            }
        }

        // Try to find a unique name (handle shadowing)
        let original_name = name.to_string();
        let mut candidate = original_name.clone();
        let mut index = 0u32;
        loop {
            if let Some(existing) = self.bindings.get(&candidate) {
                if existing.declaration_key == declaration_key {
                    return existing.identifier.clone();
                }
                // If the existing entry was pre-declared, upgrade it
                if existing.pre_declared {
                    let identifier = existing.identifier.clone();
                    if let Some(existing) = self.bindings.get_mut(&candidate) {
                        existing.declaration_key = declaration_key;
                        existing.pre_declared = false;
                    }
                    return identifier;
                }
                // Name collision with a different declaration - try next suffix
                candidate = format!("{original_name}_{index}");
                index += 1;
            } else {
                // Found a free name
                break;
            }
        }

        let id = self.env.next_identifier_id();
        let identifier = Identifier {
            id,
            declaration_id: DeclarationId(id.0),
            name: Some(IdentifierName::Named(candidate.clone())),
            mutable_range: MutableRange::default(),
            scope: None,
            type_: make_type(),
            loc,
        };
        self.env.add_new_reference(&candidate);
        // Record this change so it can be undone when exiting a block scope.
        let previous = self.bindings.remove(&candidate);
        self.record_binding_change(&candidate, previous);
        self.bindings.insert(
            candidate,
            BindingEntry {
                declaration_key,
                identifier: identifier.clone(),
                kind: BindingKind::Let,
                pre_declared: false,
            },
        );
        identifier
    }

    /// Declare a local binding (parameter or variable) with the given name and kind.
    ///
    /// Returns the HIR `Identifier` for this binding and a `Place` referencing it.
    pub fn declare_binding(&mut self, name: &str, kind: BindingKind, loc: SourceLocation) -> Place {
        let key = self.next_binding_key;
        self.next_binding_key += 1;

        let identifier = self.resolve_binding(name, key, loc);

        // Update the binding kind
        let candidate =
            identifier.name.as_ref().map_or_else(|| name.to_string(), |n| n.value().to_string());
        if let Some(entry) = self.bindings.get_mut(&candidate) {
            entry.kind = kind;
        }

        Place { identifier, effect: Effect::Unknown, reactive: false, loc }
    }

    /// Register an existing identifier from an outer scope as a local binding.
    ///
    /// This is used for FunctionExpression context variables: the inner function
    /// shares the same IdentifierId as the outer scope, so reactivity can propagate.
    pub fn register_outer_binding(
        &mut self,
        name: &str,
        identifier: Identifier,
        kind: BindingKind,
    ) {
        let candidate =
            identifier.name.as_ref().map_or_else(|| name.to_string(), |n| n.value().to_string());
        self.bindings.insert(
            candidate,
            BindingEntry {
                declaration_key: self.next_binding_key,
                identifier,
                kind,
                pre_declared: false,
            },
        );
        self.next_binding_key += 1;
    }

    /// Resolve an identifier reference to determine if it's a local binding or
    /// a global/module-level reference.
    ///
    /// This corresponds to `resolveIdentifier()` in the TS `HIRBuilder.ts`.
    ///
    /// Returns `Some(VariableBinding)` if the identifier could be resolved.
    /// For local bindings, returns `VariableBinding::Identifier`.
    /// For globals, returns `VariableBinding::NonLocal(Global { name })`.
    pub fn resolve_identifier(&self, name: &str) -> VariableBinding {
        // Check our bindings map. Try the name directly first, then suffixed versions.
        if let Some(entry) = self.bindings.get(name) {
            return VariableBinding::Identifier {
                identifier: entry.identifier.clone(),
                binding_kind: entry.kind,
            };
        }

        // Check module-scope bindings (imports and other module-level declarations).
        // This corresponds to checking `parentFunction.scope.parent.getBinding()`
        // in the TS HIRBuilder.ts, which correctly resolves renamed imports like
        // `import {useState as useReactState} from 'react'` to ImportSpecifier.
        if let Some(binding) = self.outer_bindings.get(name) {
            return VariableBinding::NonLocal(binding.clone());
        }

        // Not found in local bindings or outer bindings -> it's a global
        VariableBinding::NonLocal(NonLocalBinding::Global { name: name.to_string() })
    }

    /// Check if a named identifier is a context identifier (captured by inner closures).
    ///
    /// This corresponds to `isContextIdentifier()` in the TS `HIRBuilder.ts`.
    pub fn is_context_identifier(&self, name: &str) -> bool {
        // Must be a local binding AND in the context identifiers set
        if self.bindings.contains_key(name) {
            return self.context_identifiers.contains(name);
        }
        false
    }

    /// Check if a name is in the context identifiers set, regardless of whether
    /// the binding has been declared yet. This is needed for destructuring patterns
    /// where we need to check if an identifier will be a context variable before
    /// declaring it (since the TS reference checks `getStoreKind` before the
    /// destructure's `lowerIdentifierForAssignment` declares the binding).
    pub fn will_be_context_identifier(&self, name: &str) -> bool {
        self.context_identifiers.contains(name)
    }

    /// Add a name to the context identifiers set and the hoisted identifiers set.
    ///
    /// This corresponds to `Environment.addHoistedIdentifier()` in the TS version,
    /// which adds the identifier to both `#contextIdentifiers` and `#hoistedIdentifiers`.
    pub fn add_hoisted_identifier(&mut self, name: &str) {
        self.context_identifiers.insert(name.to_string());
        self.hoisted_identifiers.insert(name.to_string());
    }

    /// Check if a name has already been hoisted.
    ///
    /// This corresponds to `Environment.isHoistedIdentifier()` in the TS version.
    pub fn is_hoisted_identifier(&self, name: &str) -> bool {
        self.hoisted_identifiers.contains(name)
    }

    /// Set the scope binding names for this function.
    ///
    /// This should be called before lowering the function body, with a set of
    /// ALL binding names that will be declared in this function's scope.
    /// This enables `has_scope_binding` to identify captured variables even
    /// when they haven't been declared yet in the sequential lowering.
    pub fn set_scope_binding_names(&mut self, names: FxHashSet<String>) {
        self.scope_binding_names = names;
    }

    /// Check if a name is declared anywhere in this function's scope.
    ///
    /// Unlike `resolve_identifier`, this checks the pre-collected set of ALL
    /// bindings, not just those declared so far in the sequential lowering.
    /// This is used by `gather_captured_context` to correctly identify captured
    /// variables in hoisted references.
    pub fn has_scope_binding(&self, name: &str) -> bool {
        self.bindings.contains_key(name) || self.scope_binding_names.contains(name)
    }

    /// Eagerly pre-declare a binding that hasn't been declared yet in the
    /// sequential lowering but is needed for captured context resolution.
    ///
    /// Creates a real HIR Identifier and binding entry marked as `pre_declared`.
    /// When `declare_binding` is called later for the same name (during the
    /// actual variable declaration processing), `resolve_binding` will recognize
    /// the pre-declared entry and reuse its identifier.
    pub fn pre_declare_binding(&mut self, name: &str, loc: SourceLocation) -> Place {
        // If already in bindings, return the existing one
        if let Some(entry) = self.bindings.get(name) {
            return Place {
                identifier: entry.identifier.clone(),
                effect: Effect::Unknown,
                reactive: false,
                loc,
            };
        }

        let key = self.next_binding_key;
        self.next_binding_key += 1;

        let id = self.env.next_identifier_id();
        let identifier = Identifier {
            id,
            declaration_id: DeclarationId(id.0),
            name: Some(IdentifierName::Named(name.to_string())),
            mutable_range: MutableRange::default(),
            scope: None,
            type_: make_type(),
            loc,
        };
        self.env.add_new_reference(name);
        // Record this change so it can be undone when exiting a block scope.
        let previous = self.bindings.remove(name);
        self.record_binding_change(name, previous);
        self.bindings.insert(
            name.to_string(),
            BindingEntry {
                declaration_key: key,
                identifier: identifier.clone(),
                kind: BindingKind::Let,
                pre_declared: true,
            },
        );
        Place { identifier, effect: Effect::Unknown, reactive: false, loc }
    }

    /// Push a new block-scoped binding scope onto the scope stack.
    ///
    /// Call this before lowering statements that form a new block scope (e.g.,
    /// the body of an `if` branch, a `for` loop body, or an explicit `{}`
    /// block statement) for `const` and `let` variables.
    ///
    /// The returned frame index is passed to `pop_binding_scope` on exit.
    ///
    /// This simulates Babel's scope-aware binding resolution: in JS, block-scoped
    /// variables (`const`/`let`) are only visible within their declaring block.
    /// Without this, an inner `const x = 1` inside an `if`-branch would pollute
    /// the outer scope's binding map, making outer `print(x)` resolve to the inner `x`.
    pub fn push_binding_scope(&mut self) {
        self.binding_scope_stack.push(Vec::new());
    }

    /// Pop the most recent block-scoped binding scope, restoring any bindings
    /// that were shadowed or newly added by code in that scope.
    pub fn pop_binding_scope(&mut self) {
        if let Some(frame) = self.binding_scope_stack.pop() {
            for (key, prev) in frame {
                match prev {
                    Some(entry) => {
                        self.bindings.insert(key, entry);
                    }
                    None => {
                        self.bindings.remove(&key);
                    }
                }
            }
        }
    }

    /// Record a binding change within the current scope frame (if any).
    ///
    /// Called internally when a binding is inserted to track what needs to be
    /// restored when the scope exits.
    fn record_binding_change(&mut self, key: &str, previous: Option<BindingEntry>) {
        if let Some(frame) = self.binding_scope_stack.last_mut() {
            // Only record the FIRST change for a given key per scope frame.
            // Subsequent changes within the same scope don't need separate tracking
            // because the restore will use the oldest "previous" value.
            let already_tracked = frame.iter().any(|(k, _)| k == key);
            if !already_tracked {
                frame.push((key.to_string(), previous));
            }
        }
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

    /// Finish building the HIR and return both the CFG and the updated environment.
    ///
    /// Use this instead of `build()` when you need the environment with its
    /// advanced ID counters (e.g., to store in the resulting `HIRFunction`).
    ///
    /// # Errors
    /// Returns an error if the builder encountered errors during construction.
    pub fn build_with_env(self) -> Result<(Hir, Environment), CompilerError> {
        if self.errors.has_any_errors() {
            return Err(self.errors);
        }
        Ok((Hir { entry: self.entry, blocks: self.completed }, self.env))
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
                phis: Vec::new(),
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
        let WipBlock { id, kind, instructions } =
            std::mem::replace(&mut self.current, continuation);
        self.completed.insert(
            id,
            BasicBlock {
                kind,
                id,
                instructions,
                terminal,
                preds: FxHashSet::default(),
                phis: Vec::new(),
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
                phis: Vec::new(),
            },
        );
    }

    /// Sets the given WIP block as the current block, executes the callback to populate it,
    /// and then resets to the previous block.
    pub fn enter_reserved(&mut self, wip: WipBlock, f: impl FnOnce(&mut Self) -> Terminal) {
        let wip_id = wip.id;
        let prev = std::mem::replace(&mut self.current, wip);
        let terminal = f(self);
        let WipBlock { id, kind, instructions } = std::mem::replace(&mut self.current, prev);

        // After the closure executes, self.current may have changed from the
        // original wip block if `terminate(_, None)` was called as the last
        // action inside the closure. In that case, `terminate` leaves a
        // placeholder with `BlockId(0)` as current, while the actual block
        // was already completed by `terminate`. We must NOT overwrite an
        // already-completed block that has a different ID from our wip block
        // (this would clobber unrelated blocks, such as the entry block at
        // BlockId(0)).
        //
        // The check: if the current block's ID changed from the wip block's
        // original ID, it means `terminate` or `terminate_with_continuation`
        // already completed the wip block. The current block is a stale
        // placeholder or a continuation. If its ID is already in
        // `completed` (and it's not the wip block), skip the insert.
        if id != wip_id && self.completed.contains_key(&id) {
            // The wip block was already completed by terminate() inside the
            // closure. The current placeholder has a different ID (e.g.
            // BlockId(0)) that already exists in completed. Don't overwrite.
            return;
        }

        self.completed.insert(
            id,
            BasicBlock {
                kind,
                id,
                instructions,
                terminal,
                preds: FxHashSet::default(),
                phis: Vec::new(),
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
            if let Scope::Loop(s) = scope
                && (label.is_none() || label == s.label.as_deref())
            {
                return Ok(s.continue_block);
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
        let Some(block) = func.blocks.get(&block_id) else { continue };
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

/// Reorder blocks in reverse postorder, removing unreachable blocks.
/// Port of `reversePostorderBlocks` from HIRBuilder.ts.
///
/// The TS reference calls this in various passes (pruneMaybeThrows, constantPropagation)
/// to keep blocks in RPO order. Passes like `infer_mutation_aliasing_ranges` depend on
/// RPO iteration order for correct mutable range computation.
pub fn reverse_postorder_blocks(body: &mut Hir) {
    enum Phase {
        PreVisit,
        PostVisit,
    }

    let mut visited: FxHashSet<BlockId> = FxHashSet::default();
    let mut used: FxHashSet<BlockId> = FxHashSet::default();
    let mut used_fallthroughs: FxHashSet<BlockId> = FxHashSet::default();
    let mut postorder: Vec<BlockId> = Vec::new();
    let mut stack: Vec<(BlockId, bool, Phase)> = Vec::new();
    stack.push((body.entry, true, Phase::PreVisit));

    while let Some((block_id, is_used, phase)) = stack.pop() {
        match phase {
            Phase::PreVisit => {
                let was_used = used.contains(&block_id);
                let was_visited = visited.contains(&block_id);
                visited.insert(block_id);
                if is_used {
                    used.insert(block_id);
                }
                if was_visited && (was_used || !is_used) {
                    continue;
                }

                let Some(block) = body.blocks.get(&block_id) else {
                    continue;
                };

                let successors: Vec<BlockId> =
                    each_terminal_successor(&block.terminal).into_iter().rev().collect();
                let fallthrough = block.terminal.fallthrough();

                if !was_visited {
                    stack.push((block_id, is_used, Phase::PostVisit));
                }

                for &successor in &successors {
                    stack.push((successor, is_used, Phase::PreVisit));
                }

                if let Some(ft) = fallthrough {
                    if is_used {
                        used_fallthroughs.insert(ft);
                    }
                    stack.push((ft, false, Phase::PreVisit));
                }
            }
            Phase::PostVisit => {
                postorder.push(block_id);
            }
        }
    }

    postorder.reverse();

    let mut new_blocks: BlockMap = BlockMap::default();
    for block_id in &postorder {
        if used.contains(block_id) {
            if let Some(block) = body.blocks.shift_remove(block_id) {
                new_blocks.insert(*block_id, block);
            }
        } else if used_fallthroughs.contains(block_id)
            && let Some(block) = body.blocks.shift_remove(block_id)
        {
            new_blocks.insert(
                *block_id,
                BasicBlock {
                    id: block.id,
                    kind: block.kind,
                    instructions: Vec::new(),
                    phis: block.phis,
                    preds: block.preds,
                    terminal: Terminal::Unreachable(UnreachableTerminal {
                        id: block.terminal.id(),
                        loc: block.terminal.loc(),
                    }),
                },
            );
        }
    }

    body.blocks = new_blocks;
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
            if let Some(block) = block
                && let Terminal::Try(ref t) = block.terminal
            {
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

/// Port of `removeUnreachableForUpdates` from HIRBuilder.ts.
///
/// If a `for` terminal's update block has been removed (unreachable), set it to None.
pub fn remove_unreachable_for_updates(body: &mut Hir) {
    let block_ids: Vec<BlockId> = body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let should_clear = {
            let Some(block) = body.blocks.get(&block_id) else { continue };
            if let Terminal::For(t) = &block.terminal {
                t.update.is_some_and(|update| !body.blocks.contains_key(&update))
            } else {
                false
            }
        };
        if should_clear {
            if let Some(block) = body.blocks.get_mut(&block_id)
                && let Terminal::For(t) = &mut block.terminal
            {
                t.update = None;
            }
        }
    }
}

/// Port of `removeDeadDoWhileStatements` from HIRBuilder.ts.
///
/// If the test condition of a DoWhile is unreachable, replace the terminal
/// with a goto to the loop block.
pub fn remove_dead_do_while_statements(body: &mut Hir) {
    let block_ids: Vec<BlockId> = body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let replacement = {
            let Some(block) = body.blocks.get(&block_id) else { continue };
            if let Terminal::DoWhile(t) = &block.terminal {
                if !body.blocks.contains_key(&t.test) {
                    Some(Terminal::Goto(super::hir_types::GotoTerminal {
                        id: t.id,
                        block: t.r#loop,
                        variant: GotoVariant::Break,
                        loc: t.loc,
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(new_terminal) = replacement {
            if let Some(block) = body.blocks.get_mut(&block_id) {
                block.terminal = new_terminal;
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

/// Compute the reverse-postorder (RPO) traversal of blocks in the HIR CFG.
///
/// This is the canonical block ordering for forward data-flow analyses.
/// Predecessors appear before successors (barring back edges from loops).
pub fn compute_rpo_order(entry: BlockId, blocks: &super::hir_types::BlockMap) -> Vec<BlockId> {
    enum Phase {
        PreVisit,
        PostVisit,
    }

    let mut visited: rustc_hash::FxHashSet<BlockId> = rustc_hash::FxHashSet::default();
    let mut postorder: Vec<BlockId> = Vec::new();
    let mut stack: Vec<(BlockId, Phase)> = vec![(entry, Phase::PreVisit)];

    while let Some((block_id, phase)) = stack.pop() {
        match phase {
            Phase::PreVisit => {
                if !visited.insert(block_id) {
                    continue;
                }
                stack.push((block_id, Phase::PostVisit));

                let Some(block) = blocks.get(&block_id) else {
                    continue;
                };

                // Push successors in reverse so they come out in forward order after RPO reversal
                let successors = each_terminal_successor(&block.terminal);
                for successor in successors.into_iter().rev() {
                    stack.push((successor, Phase::PreVisit));
                }
            }
            Phase::PostVisit => {
                postorder.push(block_id);
            }
        }
    }

    postorder.reverse();
    postorder
}

/// Iterate over all successor block IDs of a terminal.
pub fn each_terminal_successor(terminal: &Terminal) -> Vec<BlockId> {
    let mut successors = Vec::new();
    match terminal {
        Terminal::Unsupported(_)
        | Terminal::Unreachable(_)
        | Terminal::Throw(_)
        | Terminal::Return(_) => {}
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
            // TS eachTerminalSuccessor only yields init for 'for' terminals.
            // The other blocks (test, update, loop) are reached transitively
            // through the control flow within init/test/update, not directly
            // from the for-terminal's block.
            successors.push(t.init);
        }
        Terminal::ForOf(t) => {
            // TS eachTerminalSuccessor only yields init for 'for-of' terminals.
            successors.push(t.init);
        }
        Terminal::ForIn(t) => {
            // TS eachTerminalSuccessor only yields init for 'for-in' terminals.
            successors.push(t.init);
        }
        Terminal::DoWhile(t) => {
            // TS eachTerminalSuccessor only yields loop for 'do-while' terminals.
            // The test block is reached transitively through the loop body's goto,
            // not directly from the do-while terminal's block.
            successors.push(t.r#loop);
        }
        Terminal::While(t) => {
            // TS eachTerminalSuccessor only yields test for 'while' terminals.
            // The loop body is reached transitively through the Branch terminal
            // in the test block, not directly from the while-terminal's block.
            successors.push(t.test);
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
            // TS eachTerminalSuccessor only yields block for 'try' terminals.
            // The handler is only reachable via maybe-throw terminals inside the
            // try body, not directly from the try terminal. This ensures that
            // when PruneMaybeThrows nulls out maybe-throw handlers in blocks that
            // cannot throw, reversePostorderBlocks correctly removes the handler
            // block as unreachable, allowing removeUnnecessaryTryCatch to elide
            // the try terminal.
            successors.push(t.block);
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
