use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::each_terminal_successor;
use crate::react_compiler_hir::visitors::terminal_fallthrough;
use crate::react_compiler_hir::*;
use crate::react_compiler_utils::FxIndexMap;
use crate::react_compiler_utils::FxIndexSet;
use crate::react_compiler_utils::IdentIndexMap;
use crate::scope::DeclKind;
use crate::scope::ImportBindingKind;
use crate::scope::ScopeId;
use crate::scope::ScopeResolver;
use crate::scope::SymbolId;

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use oxc_str::{Ident, format_ident};

use crate::react_compiler_lowering::identifier_loc_index::IdentifierLocIndex;

type BuildResult<'a> = Result<
    (HIR, Vec<Instruction<'a>>, IdentIndexMap<'a, SymbolId>, FxIndexMap<SymbolId, IdentifierId>),
    OxcDiagnostic,
>;

// ---------------------------------------------------------------------------
// Reserved word check (matches TS isReservedWord)
// ---------------------------------------------------------------------------

pub(crate) fn is_always_reserved_word(s: &str) -> bool {
    matches!(
        s,
        "break"
            | "case"
            | "catch"
            | "continue"
            | "debugger"
            | "default"
            | "do"
            | "else"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "class"
            | "const"
            | "enum"
            | "export"
            | "extends"
            | "import"
            | "super"
            | "null"
            | "true"
            | "false"
            | "delete"
    )
}

pub(crate) fn reserved_identifier_diagnostic(name: &str) -> OxcDiagnostic {
    ErrorCategory::Syntax.diagnostic("Expected a non-reserved identifier name").with_help(format!(
        "`{}` is a reserved word in JavaScript and cannot be used as an identifier name",
        name
    ))
}

// ---------------------------------------------------------------------------
// Scope types for tracking break/continue targets
// ---------------------------------------------------------------------------

enum Scope<'a> {
    Loop { label: Option<Ident<'a>>, continue_block: BlockId, break_block: BlockId },
    Label { label: Ident<'a>, break_block: BlockId },
    Switch { label: Option<Ident<'a>>, break_block: BlockId },
}

impl Scope<'_> {
    fn label(&self) -> Option<&str> {
        match self {
            Scope::Loop { label, .. } => label.as_deref(),
            Scope::Label { label, .. } => Some(label.as_str()),
            Scope::Switch { label, .. } => label.as_deref(),
        }
    }

    fn break_block(&self) -> BlockId {
        match self {
            Scope::Loop { break_block, .. } => *break_block,
            Scope::Label { break_block, .. } => *break_block,
            Scope::Switch { break_block, .. } => *break_block,
        }
    }
}

// ---------------------------------------------------------------------------
// WipBlock: a block under construction that does not yet have a terminal
// ---------------------------------------------------------------------------

pub struct WipBlock {
    pub id: BlockId,
    pub instructions: Vec<InstructionId>,
    pub kind: BlockKind,
}

fn new_block(id: BlockId, kind: BlockKind) -> WipBlock {
    WipBlock { id, kind, instructions: Vec::new() }
}

// ---------------------------------------------------------------------------
// HirBuilder: helper struct for constructing a CFG
// ---------------------------------------------------------------------------

pub struct HirBuilder<'a, 'b> {
    completed: FxIndexMap<BlockId, BasicBlock>,
    current: WipBlock,
    entry: BlockId,
    scopes: Vec<Scope<'a>>,
    /// Context identifiers: variables captured from an outer scope.
    /// Maps the outer scope's symbol to the source location where it was referenced.
    context: FxIndexMap<SymbolId, Option<Span>>,
    /// Resolved bindings: maps a symbol to the HIR IdentifierId created for it.
    bindings: FxIndexMap<SymbolId, IdentifierId>,
    /// Names already used by bindings, for collision avoidance.
    /// Maps name string -> how many times it has been used (for appending _0, _1, ...).
    used_names: IdentIndexMap<'a, SymbolId>,
    env: &'b mut Environment<'a>,
    scope: &'b ScopeResolver<'b, 'a>,
    exception_handler_stack: Vec<BlockId>,
    /// Flat instruction table being built up.
    instruction_table: Vec<Instruction<'a>>,
    /// Traversal context: counts the number of `fbt` tag parents
    /// of the current babel node.
    pub fbt_depth: u32,
    /// The scope of the function being compiled (for context identifier checks).
    function_scope: ScopeId,
    /// The scope of the outermost component/hook function (for gather_captured_context).
    component_scope: ScopeId,
    /// Symbols declared in scopes between component_scope and any inner
    /// function scope, that are referenced from an inner function scope.
    /// These need StoreContext/LoadContext instead of StoreLocal/LoadLocal.
    context_identifiers: rustc_hash::FxHashSet<SymbolId>,
    /// Index mapping identifier byte offsets to source locations and JSX status.
    identifier_spans: &'b IdentifierLocIndex,
}

impl<'a, 'b> HirBuilder<'a, 'b> {
    // -----------------------------------------------------------------------
    // M2: Core methods
    // -----------------------------------------------------------------------

    /// Create a new HirBuilder.
    ///
    /// - `env`: the shared environment (counters, arenas, error accumulator)
    /// - `scope`: the semantic scope resolver
    /// - `function_scope`: the ScopeId of the function being compiled
    /// - `bindings`: optional pre-existing bindings (e.g., from a parent function)
    /// - `context`: optional pre-existing captured context map
    /// - `entry_block_kind`: the kind of the entry block (defaults to `Block`)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        env: &'b mut Environment<'a>,
        scope: &'b ScopeResolver<'b, 'a>,
        function_scope: ScopeId,
        component_scope: ScopeId,
        context_identifiers: rustc_hash::FxHashSet<SymbolId>,
        bindings: Option<FxIndexMap<SymbolId, IdentifierId>>,
        context: Option<FxIndexMap<SymbolId, Option<Span>>>,
        entry_block_kind: Option<BlockKind>,
        used_names: Option<IdentIndexMap<'a, SymbolId>>,
        identifier_spans: &'b IdentifierLocIndex,
    ) -> Self {
        let entry = env.next_block_id();
        let kind = entry_block_kind.unwrap_or(BlockKind::Block);
        HirBuilder {
            completed: FxIndexMap::default(),
            current: new_block(entry, kind),
            entry,
            scopes: Vec::new(),
            context: context.unwrap_or_default(),
            bindings: bindings.unwrap_or_default(),
            used_names: used_names.unwrap_or_default(),
            env,
            scope,
            exception_handler_stack: Vec::new(),
            instruction_table: Vec::new(),
            fbt_depth: 0,
            function_scope,
            component_scope,
            context_identifiers,
            identifier_spans,
        }
    }

    /// Check if a scope is the component scope or a descendant of it.
    /// Used to determine whether a binding is local to the compiled function
    /// or belongs to an ancestor function scope (e.g., a factory function
    /// wrapping a nested component declaration).
    /// Uses component_scope (the outermost compiled function's scope) rather
    /// than function_scope because inner function expressions within the
    /// compiled function have their own function_scope but still consider
    /// the outer component's variables as local.
    fn is_scope_within_compiled_function(&self, scope_id: ScopeId) -> bool {
        self.scope.ancestors(scope_id).any(|id| id == self.component_scope)
    }

    /// Access the environment.
    pub fn environment(&self) -> &Environment<'a> {
        self.env
    }

    /// Access the environment mutably.
    pub fn environment_mut(&mut self) -> &mut Environment<'a> {
        self.env
    }

    /// Create a new unique TypeVar type, allocated from the environment's type arena
    /// so that TypeIds are consistent with identifier type slots.
    pub fn make_type(&mut self) -> Type<'a> {
        let type_id = self.env.make_type();
        Type::TypeVar { id: type_id }
    }

    /// Access the scope resolver.
    /// Returns the 'b reference to avoid conflicts with mutable borrows on self.
    pub fn scope(&self) -> &'b ScopeResolver<'b, 'a> {
        self.scope
    }

    /// The declaration identifier's span, when the declaration was recorded
    /// in the compiled function's identifier index.
    pub fn declaration_span(&self, symbol_id: SymbolId) -> Option<Span> {
        self.identifier_spans.declaration_span(symbol_id)
    }

    /// Access the function scope (the scope of the function being compiled).
    pub fn function_scope(&self) -> ScopeId {
        self.function_scope
    }

    /// Access the component scope.
    pub fn component_scope(&self) -> ScopeId {
        self.component_scope
    }

    /// Access the context map.
    pub fn context(&self) -> &FxIndexMap<SymbolId, Option<Span>> {
        &self.context
    }

    /// Access the pre-computed context identifiers set.
    pub fn context_identifiers(&self) -> &rustc_hash::FxHashSet<SymbolId> {
        &self.context_identifiers
    }

    /// Add a binding to the context identifiers set (used by hoisting).
    pub fn add_context_identifier(&mut self, symbol_id: SymbolId) {
        self.context_identifiers.insert(symbol_id);
    }

    /// Access the identifier location index.
    /// Returns the 'a reference to avoid conflicts with mutable borrows on self.
    pub fn identifier_spans(&self) -> &'b IdentifierLocIndex {
        self.identifier_spans
    }

    /// Access the bindings map.
    pub fn bindings(&self) -> &FxIndexMap<SymbolId, IdentifierId> {
        &self.bindings
    }

    /// Access the used names map.
    pub fn used_names(&self) -> &IdentIndexMap<'a, SymbolId> {
        &self.used_names
    }

    /// Merge used names from a child builder back into this builder.
    /// This ensures name deduplication works across function scopes.
    pub fn merge_used_names(&mut self, child_used_names: IdentIndexMap<'a, SymbolId>) {
        for (name, symbol_id) in child_used_names {
            self.used_names.entry(name).or_insert(symbol_id);
        }
    }

    /// Merge bindings (symbol -> IdentifierId) from a child builder back into this builder.
    /// This matches TS behavior where parent and child share the same #bindings map by reference,
    /// so bindings resolved by the child are automatically visible to the parent.
    pub fn merge_bindings(&mut self, child_bindings: FxIndexMap<SymbolId, IdentifierId>) {
        for (symbol_id, identifier_id) in child_bindings {
            self.bindings.entry(symbol_id).or_insert(identifier_id);
        }
    }

    /// Push an instruction onto the current block.
    ///
    /// Adds the instruction to the flat instruction table and records
    /// its InstructionId in the current block's instruction list.
    ///
    /// If an exception handler is active, also emits a MaybeThrow terminal
    /// after the instruction to model potential control flow to the handler,
    /// then continues in a new block.
    pub fn push(&mut self, instruction: Instruction<'a>) {
        let span = instruction.span;
        let instr_id = InstructionId::from_usize(self.instruction_table.len());
        self.instruction_table.push(instruction);
        self.current.instructions.push(instr_id);

        if let Some(&handler) = self.exception_handler_stack.last() {
            let continuation = self.reserve(self.current_block_kind());
            self.terminate_with_continuation(
                Terminal::MaybeThrow {
                    continuation: continuation.id,
                    handler: Some(handler),
                    id: EvaluationOrder::UNSET,
                    span,
                    effects: None,
                },
                continuation,
            );
        }
    }

    /// Insert `wip` into `completed` as a finished block terminated by `terminal`.
    ///
    /// Kept non-generic (and out-of-line) so the block-completion machinery compiles
    /// once instead of being duplicated into every generic `try_enter*` instantiation.
    #[inline(never)]
    fn complete_block(&mut self, wip: WipBlock, terminal: Terminal) {
        self.completed.insert(
            wip.id,
            BasicBlock {
                kind: wip.kind,
                id: wip.id,
                instructions: wip.instructions,
                terminal,
                preds: FxIndexSet::default(),
                phis: Vec::new(),
            },
        );
    }

    /// Terminate the current block with the given terminal and start a new block.
    ///
    /// If `next_block_kind` is `Some`, a new current block is created with that kind.
    /// Returns the BlockId of the completed block.
    pub fn terminate(&mut self, terminal: Terminal, next_block_kind: Option<BlockKind>) -> BlockId {
        // The placeholder block created here (`BlockId::PLACEHOLDER`) is only used when
        // next_block_kind is None, meaning this is the final terminate() call.
        // It will never be read or completed because build() consumes self
        // immediately after, and no further operations should occur on the builder.
        let wip =
            std::mem::replace(&mut self.current, new_block(BlockId::PLACEHOLDER, BlockKind::Block));
        let block_id = wip.id;

        self.complete_block(wip, terminal);

        if let Some(kind) = next_block_kind {
            let next_id = self.env.next_block_id();
            self.current = new_block(next_id, kind);
        }
        block_id
    }

    /// Terminate the current block with the given terminal, and set
    /// a previously reserved block as the new current block.
    pub fn terminate_with_continuation(&mut self, terminal: Terminal, continuation: WipBlock) {
        let wip = std::mem::replace(&mut self.current, continuation);
        self.complete_block(wip, terminal);
    }

    /// Reserve a new block so it can be referenced before construction.
    /// Use `terminate_with_continuation()` to make it current, or `complete()` to
    /// save it directly.
    pub fn reserve(&mut self, kind: BlockKind) -> WipBlock {
        let id = self.env.next_block_id();
        new_block(id, kind)
    }

    /// Like `enter_reserved`, but the closure returns a `Result<Terminal, OxcDiagnostic>`.
    pub fn try_enter_reserved(
        &mut self,
        wip: WipBlock,
        f: impl FnOnce(&mut Self) -> Result<Terminal, OxcDiagnostic>,
    ) -> Result<(), OxcDiagnostic> {
        let prev = std::mem::replace(&mut self.current, wip);
        let terminal = f(self)?;
        let completed_wip = std::mem::replace(&mut self.current, prev);
        self.complete_block(completed_wip, terminal);
        Ok(())
    }

    /// Like `enter`, but the closure returns a `Result<Terminal, OxcDiagnostic>`.
    pub fn try_enter(
        &mut self,
        kind: BlockKind,
        f: impl FnOnce(&mut Self, BlockId) -> Result<Terminal, OxcDiagnostic>,
    ) -> Result<BlockId, OxcDiagnostic> {
        let wip = self.reserve(kind);
        let wip_id = wip.id;
        self.try_enter_reserved(wip, |this| f(this, wip_id))?;
        Ok(wip_id)
    }

    /// Like `enter_try_catch`, but the closure returns a `Result`.
    pub fn try_enter_try_catch(
        &mut self,
        handler: BlockId,
        f: impl FnOnce(&mut Self) -> Result<(), OxcDiagnostic>,
    ) -> Result<(), OxcDiagnostic> {
        self.exception_handler_stack.push(handler);
        let result = f(self);
        self.exception_handler_stack.pop();
        result
    }

    /// Return the top of the exception handler stack, or None.
    pub fn resolve_throw_handler(&self) -> Option<BlockId> {
        self.exception_handler_stack.last().copied()
    }

    /// Push a Loop scope, run the closure, pop and verify.
    pub fn loop_scope<T>(
        &mut self,
        label: Option<Ident<'a>>,
        continue_block: BlockId,
        break_block: BlockId,
        f: impl FnOnce(&mut Self) -> Result<T, OxcDiagnostic>,
    ) -> Result<T, OxcDiagnostic> {
        self.scopes.push(Scope::Loop { label, continue_block, break_block });
        let value = f(self)?;
        let last = self.scopes.pop().expect("Mismatched loop scope: stack empty");
        match &last {
            Scope::Loop { label: l, continue_block: c, break_block: b } => {
                assert!(
                    *l == label && *c == continue_block && *b == break_block,
                    "Mismatched loop scope"
                );
            }
            _ => {
                return Err(ErrorCategory::Invariant
                    .diagnostic("Mismatched loop scope: expected Loop, got other"));
            }
        }
        Ok(value)
    }

    /// Push a Label scope, run the closure, pop and verify.
    pub fn label_scope<T>(
        &mut self,
        label: Ident<'a>,
        break_block: BlockId,
        f: impl FnOnce(&mut Self) -> Result<T, OxcDiagnostic>,
    ) -> Result<T, OxcDiagnostic> {
        self.scopes.push(Scope::Label { label, break_block });
        let value = f(self)?;
        let last = self.scopes.pop().expect("Mismatched label scope: stack empty");
        match &last {
            Scope::Label { label: l, break_block: b } => {
                assert!(*l == label && *b == break_block, "Mismatched label scope");
            }
            _ => {
                return Err(ErrorCategory::Invariant
                    .diagnostic("Mismatched label scope: expected Label, got other"));
            }
        }
        Ok(value)
    }

    /// Push a Switch scope, run the closure, pop and verify.
    pub fn switch_scope<T>(
        &mut self,
        label: Option<Ident<'a>>,
        break_block: BlockId,
        f: impl FnOnce(&mut Self) -> Result<T, OxcDiagnostic>,
    ) -> Result<T, OxcDiagnostic> {
        self.scopes.push(Scope::Switch { label, break_block });
        let value = f(self)?;
        let last = self.scopes.pop().expect("Mismatched switch scope: stack empty");
        match &last {
            Scope::Switch { label: l, break_block: b } => {
                assert!(*l == label && *b == break_block, "Mismatched switch scope");
            }
            _ => {
                return Err(ErrorCategory::Invariant
                    .diagnostic("Mismatched switch scope: expected Switch, got other"));
            }
        }
        Ok(value)
    }

    /// Look up the break target for the given label (or the innermost
    /// loop/switch if label is None).
    pub fn lookup_break(&self, label: Option<&str>) -> Result<BlockId, OxcDiagnostic> {
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Loop { .. } | Scope::Switch { .. } if label.is_none() => {
                    return Ok(scope.break_block());
                }
                _ if label.is_some() && scope.label() == label => {
                    return Ok(scope.break_block());
                }
                _ => continue,
            }
        }
        Err(ErrorCategory::Invariant
            .diagnostic("Expected a loop or switch to be in scope for break"))
    }

    /// Look up the continue target for the given label (or the innermost
    /// loop if label is None). Only loops support continue.
    pub fn lookup_continue(&self, label: Option<&str>) -> Result<BlockId, OxcDiagnostic> {
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Loop { label: scope_label, continue_block, .. } => {
                    if label.is_none() || label == scope_label.as_deref() {
                        return Ok(*continue_block);
                    }
                }
                _ => {
                    if label.is_some() && scope.label() == label {
                        return Err(ErrorCategory::Invariant
                            .diagnostic("Continue may only refer to a labeled loop"));
                    }
                }
            }
        }
        Err(ErrorCategory::Invariant.diagnostic("Expected a loop to be in scope for continue"))
    }

    /// Create a temporary identifier with a fresh id, returning its IdentifierId.
    pub fn make_temporary(&mut self, span: Option<Span>) -> IdentifierId {
        let id = self.env.next_identifier_id();
        // Update the span on the allocated identifier
        self.env.identifiers[id.index()].span = span;
        id
    }

    /// Record an error on the environment.
    /// Returns `Err` for Invariant errors (matching TS throw behavior).
    pub fn record_error(&mut self, error: OxcDiagnostic) -> Result<(), OxcDiagnostic> {
        self.env.record_error(error)
    }

    /// Record a diagnostic on the environment.
    pub fn record_diagnostic(&mut self, diagnostic: OxcDiagnostic) {
        self.env.record_diagnostic(diagnostic);
    }

    /// Check if a name has a local binding (non-module-level).
    /// This is used for checking if fbt/fbs JSX tags are local bindings
    /// (which is not supported).
    pub fn has_local_binding(&self, name: &str) -> bool {
        if let Some(symbol_id) = self.scope.find_binding_in_descendants(name, self.component_scope)
        {
            return self.scope.symbol_scope(symbol_id) != self.scope.program_scope();
        }
        false
    }

    /// Return the kind of the current block.
    pub fn current_block_kind(&self) -> BlockKind {
        self.current.kind
    }

    /// Construct the final HIR and instruction table from the completed blocks.
    ///
    /// Performs these post-build passes:
    /// 1. Reverse-postorder sort + unreachable block removal
    /// 2. Check for unreachable blocks containing FunctionExpression instructions
    /// 3. Remove unreachable for-loop updates
    /// 4. Remove dead do-while statements
    /// 5. Remove unnecessary try-catch
    /// 6. Number all instructions and terminals
    /// 7. Mark predecessor blocks
    pub fn build(mut self) -> BuildResult<'a> {
        let mut hir = HIR { blocks: std::mem::take(&mut self.completed), entry: self.entry };

        let mut instructions = std::mem::take(&mut self.instruction_table);

        let rpo_blocks = get_reverse_postordered_blocks(&hir);

        // Check for unreachable blocks that contain FunctionExpression instructions.
        // These could contain hoisted declarations that we can't safely remove.
        for (id, block) in &hir.blocks {
            if !rpo_blocks.contains_key(id) {
                let has_function_expr = block.instructions.iter().any(|&instr_id| {
                    matches!(
                        instructions[instr_id.index()].value,
                        InstructionValue::FunctionExpression { .. }
                    )
                });
                if has_function_expr {
                    let span = block
                        .instructions
                        .first()
                        .and_then(|&i| instructions[i.index()].span)
                        .or_else(|| block.terminal.span().copied());
                    self.env.record_error(
                        ErrorCategory::Todo
                            .diagnostic("Support functions with unreachable code that may contain hoisted declarations")
                            .with_labels(span),
                    )?;
                }
            }
        }

        hir.blocks = rpo_blocks;

        remove_unreachable_for_updates(&mut hir);
        remove_dead_do_while_statements(&mut hir);
        remove_unnecessary_try_catch(&mut hir);
        mark_instruction_ids(&mut hir, &mut instructions);
        mark_predecessors(&mut hir);

        Ok((hir, instructions, self.used_names, self.bindings))
    }

    // -----------------------------------------------------------------------
    // M3: Binding resolution methods
    // -----------------------------------------------------------------------

    /// Map a symbol to an HIR IdentifierId.
    ///
    /// On first encounter, creates a new Identifier with the given name and a fresh id.
    /// On subsequent encounters, returns the cached IdentifierId.
    /// Handles name collisions by appending `_0`, `_1`, etc.
    ///
    /// Records errors for variables named 'fbt' or 'this'.
    pub fn resolve_binding(
        &mut self,
        name: Ident<'a>,
        symbol_id: SymbolId,
    ) -> Result<IdentifierId, OxcDiagnostic> {
        self.resolve_binding_with_span(name, symbol_id, None)
    }

    /// Map a symbol to an HIR IdentifierId, with an optional source location.
    pub fn resolve_binding_with_span(
        &mut self,
        name: Ident<'a>,
        symbol_id: SymbolId,
        span: Option<Span>,
    ) -> Result<IdentifierId, OxcDiagnostic> {
        // Check for unsupported names BEFORE the cache check.
        // In TS, resolveBinding records fbt errors when node.name === 'fbt'. After a name collision
        // causes a rename (e.g., "fbt" -> "fbt_0"), TS's scope.rename changes the AST node's name,
        // preventing subsequent fbt error recording. We simulate this by checking whether the
        // resolved name for this binding is still "fbt" (not renamed to "fbt_0" etc.).
        if name == "fbt" {
            // Check if this binding was previously resolved to a renamed version
            let should_record_fbt_error =
                if let Some(&identifier_id) = self.bindings.get(&symbol_id) {
                    // Already resolved - check if the resolved name is still "fbt"
                    match &self.env.identifiers[identifier_id.index()].name {
                        Some(IdentifierName::Named(resolved_name)) => resolved_name == "fbt",
                        _ => false,
                    }
                } else {
                    // First resolution - always record
                    true
                };
            if should_record_fbt_error {
                let error_span = self.declaration_span(symbol_id).or(span);
                self.env.record_error(
                    ErrorCategory::Todo
                        .diagnostic("Support local variables named `fbt`")
                        .with_help("Local variables named `fbt` may conflict with the fbt plugin and are not yet supported")
                        .with_labels(error_span),
                )?;
            }
        }

        // If we've already resolved this binding, return the cached IdentifierId
        if let Some(&identifier_id) = self.bindings.get(&symbol_id) {
            return Ok(identifier_id);
        }

        if is_always_reserved_word(name.as_str()) {
            // Match TS behavior: makeIdentifierName throws for reserved words.
            return Err(reserved_identifier_diagnostic(name.as_str()));
        }

        // Find a unique name: start with the original name, then try name_0, name_1, ...
        let mut candidate = name;
        let mut index = 0u32;
        while let Some(&existing_symbol_id) = self.used_names.get(&candidate) {
            if existing_symbol_id == symbol_id {
                // Same binding, use this name
                break;
            }
            // Name collision with a different binding, try the next suffix
            candidate = format_ident!(self.env.allocator, "{name}_{index}");
            index += 1;
        }

        // Record the rename on each resolved reference of the binding, so codegen
        // can rename matching identifiers inside preserved TS type annotations.
        if candidate != name {
            for &reference_id in self.scope.reference_ids(symbol_id) {
                self.env.renames.insert(reference_id, candidate);
            }
        }

        // Allocate identifier in the arena
        let id = self.env.next_identifier_id();
        // Update the name and span on the allocated identifier
        self.env.identifiers[id.index()].name = Some(IdentifierName::Named(candidate));
        // Prefer the binding's declaration span over the reference span.
        // This matches TS behavior where Babel's resolveBinding returns the
        // binding identifier's original span (the declaration site).
        let decl_span = self.declaration_span(symbol_id);
        if let Some(ref dl) = decl_span {
            self.env.identifiers[id.index()].span = Some(*dl);
        } else if let Some(ref span) = span {
            self.env.identifiers[id.index()].span = Some(*span);
        }

        self.used_names.insert(candidate, symbol_id);
        self.bindings.insert(symbol_id, id);
        Ok(id)
    }

    /// Set the span on an identifier to the declaration-site span.
    /// This overrides any previously-set span (which may have come from a reference site).
    pub fn set_identifier_declaration_span(&mut self, id: IdentifierId, span: Span) {
        self.env.identifiers[id.index()].span = Some(span);
    }

    /// Resolve an identifier reference to a VariableBinding.
    ///
    /// Uses the scope resolver to determine whether the reference is:
    /// - Global (no binding found)
    /// - ImportDefault, ImportSpecifier, ImportNamespace (program-scope import binding)
    /// - ModuleLocal (program-scope non-import binding)
    /// - Identifier (local binding, resolved via resolve_binding)
    pub fn resolve_identifier(
        &mut self,
        name: Ident<'a>,
        span: Span,
        symbol: Option<SymbolId>,
    ) -> Result<VariableBinding<'a>, OxcDiagnostic> {
        let Some(symbol_id) = symbol else {
            // No binding found: this is a global
            return Ok(VariableBinding::Global { name });
        };
        // Treat type-only declarations as globals so the compiler
        // doesn't try to create/initialize HIR bindings for them.
        // TSEnumDeclaration is included because a function with an inline
        // enum is skipped (`skip_compilation`) and the enum binding is
        // never initialized in HIR.
        if matches!(
            self.scope.decl_kind(symbol_id),
            DeclKind::TSTypeAliasDeclaration
                | DeclKind::TSEnumDeclaration
                | DeclKind::TSModuleDeclaration
        ) {
            return Ok(VariableBinding::Global { name });
        }
        let symbol_scope = self.scope.symbol_scope(symbol_id);
        if symbol_scope == self.scope.program_scope() {
            // Module-level binding: check import info
            Ok(match self.scope.import_data(symbol_id) {
                Some(import_info) => match import_info.kind {
                    ImportBindingKind::Default => {
                        VariableBinding::ImportDefault { name, module: import_info.source }
                    }
                    ImportBindingKind::Named => VariableBinding::ImportSpecifier {
                        name,
                        module: import_info.source,
                        imported: import_info.imported.unwrap_or(name),
                    },
                    ImportBindingKind::Namespace => {
                        VariableBinding::ImportNamespace { name, module: import_info.source }
                    }
                },
                None => VariableBinding::ModuleLocal { name },
            })
        } else if !self.is_scope_within_compiled_function(symbol_scope) {
            Ok(VariableBinding::ModuleLocal { name })
        } else {
            let binding_kind = crate::react_compiler_lowering::convert_binding_kind(
                &self.scope.binding_kind(symbol_id),
            );
            let identifier_id = self.resolve_binding_with_span(name, symbol_id, Some(span))?;
            Ok(VariableBinding::Identifier { identifier: identifier_id, binding_kind })
        }
    }

    /// Check if an identifier reference resolves to a context identifier.
    ///
    /// A context identifier is a variable declared in an ancestor scope of the
    /// current function's scope, but NOT in the program scope itself and NOT
    /// in the function's own scope. These are "captured" variables from an
    /// enclosing function.
    pub fn is_context_identifier(&self, symbol: Option<SymbolId>) -> bool {
        match symbol {
            None => false,
            Some(symbol_id) => {
                if self.scope.symbol_scope(symbol_id) == self.scope.program_scope() {
                    return false;
                }
                self.context_identifiers.contains(&symbol_id)
            }
        }
    }

    /// Like `is_context_identifier`, for callers that already resolved a
    /// symbol instead of going through a reference node.
    pub fn is_context_binding(&self, symbol_id: SymbolId) -> bool {
        if self.scope.symbol_scope(symbol_id) == self.scope.program_scope() {
            return false;
        }
        self.context_identifiers.contains(&symbol_id)
    }

    /// Resolve the binding for a function declaration's id the way TS does:
    /// Babel's `path.scope.getBinding(name)` starts at the function's OWN
    /// scope, so a body-level local (or parameter) that shadows the function's
    /// name resolves to that inner binding rather than to the function's
    /// hoisted binding in the parent scope.
    ///
    /// Babel's `scope.rename` re-keys a scope's bindings when the TS builder
    /// renames a shadowed binding (e.g. `init` -> `init_0`), so a binding only
    /// matches if its *current* name — the resolved HIR identifier name once
    /// resolved — still equals `name`. A binding renamed *to* `name` overwrites
    /// the original key in Babel and takes precedence over an unresolved
    /// binding with that original name.
    ///
    /// Returns None when the walk resolves outside the compiled function
    /// (degraded scope info); callers should fall back to node-based
    /// resolution in that case.
    pub fn get_function_declaration_binding(
        &self,
        function_scope: ScopeId,
        name: &str,
    ) -> Option<SymbolId> {
        // None = unresolved binding; Some(matches) = resolved, current name comparison
        let resolved_name_matches = |sid: SymbolId| -> Option<bool> {
            let &identifier_id = self.bindings.get(&sid)?;
            match &self.env.identifiers[identifier_id.index()].name {
                Some(IdentifierName::Named(n)) => Some(n == name),
                _ => Some(false),
            }
        };
        let mut current = Some(function_scope);
        while let Some(id) = current {
            let mut found =
                self.scope.bindings_in(id).find(|&sid| resolved_name_matches(sid) == Some(true));
            if found.is_none() {
                if let Some(sid) = self.scope.get_binding(id, name) {
                    // Skip bindings that were renamed away from `name`.
                    if resolved_name_matches(sid) != Some(false) {
                        found = Some(sid);
                    }
                }
            }
            if let Some(sid) = found {
                if !self.is_scope_within_compiled_function(self.scope.symbol_scope(sid)) {
                    return None;
                }
                return Some(sid);
            }
            current = self.scope.scope_parent(id);
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Post-build helper functions
// ---------------------------------------------------------------------------

/// Compute a reverse-postorder of blocks reachable from the entry.
///
/// Visits successors in reverse order so that when the postorder list is
/// reversed, sibling edges appear in program order.
///
/// Blocks not reachable through successors are removed. Blocks that are
/// only reachable as fallthroughs (not through real successor edges) are
/// replaced with empty blocks that have an Unreachable terminal.
pub fn get_reverse_postordered_blocks(hir: &HIR) -> FxIndexMap<BlockId, BasicBlock> {
    let mut visited: FxIndexSet<BlockId> = FxIndexSet::default();
    let mut used: FxIndexSet<BlockId> = FxIndexSet::default();
    let mut used_fallthroughs: FxIndexSet<BlockId> = FxIndexSet::default();
    let mut postorder: Vec<BlockId> = Vec::new();

    fn visit(
        hir: &HIR,
        block_id: BlockId,
        is_used: bool,
        visited: &mut FxIndexSet<BlockId>,
        used: &mut FxIndexSet<BlockId>,
        used_fallthroughs: &mut FxIndexSet<BlockId>,
        postorder: &mut Vec<BlockId>,
    ) {
        let was_used = used.contains(&block_id);
        let was_visited = visited.contains(&block_id);
        visited.insert(block_id);
        if is_used {
            used.insert(block_id);
        }
        if was_visited && (was_used || !is_used) {
            return;
        }

        let block = hir
            .blocks
            .get(&block_id)
            .unwrap_or_else(|| panic!("[HIRBuilder] expected block {:?} to exist", block_id));

        // Visit successors in reverse order so that when we reverse the
        // postorder list, sibling edges come out in program order.
        let mut successors = each_terminal_successor(&block.terminal);
        successors.reverse();

        let fallthrough = terminal_fallthrough(&block.terminal);

        // Visit fallthrough first (marking as not-yet-used) to ensure its
        // block ID is emitted in the correct position.
        if let Some(ft) = fallthrough {
            if is_used {
                used_fallthroughs.insert(ft);
            }
            visit(hir, ft, false, visited, used, used_fallthroughs, postorder);
        }
        for successor in successors {
            visit(hir, successor, is_used, visited, used, used_fallthroughs, postorder);
        }

        if !was_visited {
            postorder.push(block_id);
        }
    }

    visit(hir, hir.entry, true, &mut visited, &mut used, &mut used_fallthroughs, &mut postorder);

    let mut blocks = FxIndexMap::default();
    for block_id in postorder.into_iter().rev() {
        let block = hir.blocks.get(&block_id).unwrap();
        if used.contains(&block_id) {
            blocks.insert(block_id, block.clone());
        } else if used_fallthroughs.contains(&block_id) {
            blocks.insert(
                block_id,
                BasicBlock {
                    kind: block.kind,
                    id: block_id,
                    instructions: Vec::new(),
                    terminal: Terminal::Unreachable {
                        id: block.terminal.evaluation_order(),
                        span: block.terminal.span().copied(),
                    },
                    preds: block.preds.clone(),
                    phis: Vec::new(),
                },
            );
        }
        // otherwise this block is unreachable and is dropped
    }

    blocks
}

/// For each block with a `For` terminal whose update block is not in the
/// blocks map, set update to None.
pub fn remove_unreachable_for_updates(hir: &mut HIR) {
    let block_ids: FxIndexSet<BlockId> = hir.blocks.keys().copied().collect();
    for block in hir.blocks.values_mut() {
        if let Terminal::For { update, .. } = &mut block.terminal {
            if let Some(update_id) = *update {
                if !block_ids.contains(&update_id) {
                    *update = None;
                }
            }
        }
    }
}

/// For each block with a `DoWhile` terminal whose test block is not in
/// the blocks map, replace the terminal with a Goto to the loop block.
pub fn remove_dead_do_while_statements(hir: &mut HIR) {
    let block_ids: FxIndexSet<BlockId> = hir.blocks.keys().copied().collect();
    for block in hir.blocks.values_mut() {
        let should_replace = if let Terminal::DoWhile { test, .. } = &block.terminal {
            !block_ids.contains(test)
        } else {
            false
        };
        if should_replace {
            if let Terminal::DoWhile { loop_block, id, span, .. } = std::mem::replace(
                &mut block.terminal,
                Terminal::Unreachable { id: EvaluationOrder::UNSET, span: None },
            ) {
                block.terminal =
                    Terminal::Goto { block: loop_block, variant: GotoVariant::Break, id, span };
            }
        }
    }
}

/// For each block with a `Try` terminal whose handler block is not in
/// the blocks map, replace the terminal with a Goto to the try block.
///
/// Also cleans up the fallthrough block's predecessors if the handler
/// was the only path to it.
pub fn remove_unnecessary_try_catch(hir: &mut HIR) {
    let block_ids: FxIndexSet<BlockId> = hir.blocks.keys().copied().collect();

    // Collect the blocks that need replacement and their associated data
    let replacements: Vec<(BlockId, BlockId, BlockId, BlockId, Option<Span>)> = hir
        .blocks
        .iter()
        .filter_map(|(&block_id, block)| {
            if let Terminal::Try { block: try_block, handler, fallthrough, span, .. } =
                &block.terminal
            {
                if !block_ids.contains(handler) {
                    return Some((block_id, *try_block, *handler, *fallthrough, *span));
                }
            }
            None
        })
        .collect();

    for (block_id, try_block, handler_id, fallthrough_id, span) in replacements {
        // Replace the terminal
        if let Some(block) = hir.blocks.get_mut(&block_id) {
            block.terminal = Terminal::Goto {
                block: try_block,
                id: EvaluationOrder::UNSET,
                span,
                variant: GotoVariant::Break,
            };
        }

        // Clean up fallthrough predecessor info
        if let Some(fallthrough) = hir.blocks.get_mut(&fallthrough_id) {
            if fallthrough.preds.len() == 1 && fallthrough.preds.contains(&handler_id) {
                // The handler was the only predecessor: remove the fallthrough block
                hir.blocks.shift_remove(&fallthrough_id);
            } else {
                fallthrough.preds.shift_remove(&handler_id);
            }
        }
    }
}

/// Sequentially number all instructions and terminals starting from 1.
pub fn mark_instruction_ids(hir: &mut HIR, instructions: &mut [Instruction]) {
    let mut order: u32 = 0;
    for block in hir.blocks.values_mut() {
        for &instr_id in &block.instructions {
            order += 1;
            instructions[instr_id.index()].id = EvaluationOrder::from_usize(order as usize);
        }
        order += 1;
        block.terminal.set_evaluation_order(EvaluationOrder::from_usize(order as usize));
    }
}

/// DFS from entry, for each successor add the predecessor's id to
/// the successor's preds set.
///
/// Note: This only visits direct successors (via `each_terminal_successor`),
/// not fallthrough blocks. Fallthrough blocks are reached indirectly via
/// Goto terminals from within branching blocks, matching the TypeScript
/// `markPredecessors` behavior.
pub fn mark_predecessors(hir: &mut HIR) {
    // Clear all preds first
    for block in hir.blocks.values_mut() {
        block.preds.clear();
    }

    let mut visited: FxIndexSet<BlockId> = FxIndexSet::default();

    fn visit(
        hir: &mut HIR,
        block_id: BlockId,
        prev_block_id: Option<BlockId>,
        visited: &mut FxIndexSet<BlockId>,
    ) {
        // Add predecessor
        if let Some(prev_id) = prev_block_id {
            if let Some(block) = hir.blocks.get_mut(&block_id) {
                block.preds.insert(prev_id);
            } else {
                return;
            }
        }

        if visited.contains(&block_id) {
            return;
        }
        visited.insert(block_id);

        // Get successors before mutating
        let successors = if let Some(block) = hir.blocks.get(&block_id) {
            each_terminal_successor(&block.terminal)
        } else {
            return;
        };

        for successor in successors {
            visit(hir, successor, Some(block_id), visited);
        }
    }

    visit(hir, hir.entry, None, &mut visited);
}

// ---------------------------------------------------------------------------
// Public helper functions
// ---------------------------------------------------------------------------

/// Create a temporary Place with a fresh identifier allocated in the arena.
pub fn create_temporary_place(env: &mut Environment<'_>, span: Option<Span>) -> Place {
    let id = env.next_identifier_id();
    // Update the span on the allocated identifier
    env.identifiers[id.index()].span = span;
    Place { identifier: id, reactive: false, effect: Effect::Unknown, span: None }
}
