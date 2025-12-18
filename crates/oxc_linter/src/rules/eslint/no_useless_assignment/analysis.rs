//! Liveness analysis based on dataflow over the Control Flow Graph (CFG).
//!
//! This module computes, for each local variable in a function, whether that
//! variable is live at a given point in the program. A variable is "live" if
//! its current value might be read before the next write.
//!
//! # Basic idea
//!
//! We represent sets of variables using the RWU (Read-Write-Use) table, which
//! tracks for each basic block and variable whether it is read, written, or used.
//!
//! We walk over the CFG in reverse execution order (backwards from successors
//! to predecessors). If we find a use of a variable, we mark it as live. If we
//! find an assignment to a variable, we mark it as written. When flows merge,
//! we take the union. For loops, we iterate until reaching a fixed point.
//!
//! # Implementation details
//!
//! The analysis leverages oxc's existing Control Flow Graph (CFG) and semantic
//! analysis. The CFG provides basic blocks and edges representing control flow,
//! while the semantic analysis provides symbol/reference information.
//!
//! # Algorithm
//!
//! Based on the Rust compiler's liveness analysis:
//! 1. Start from exit blocks and work backwards through the CFG
//! 2. For each basic block, process instructions in reverse order
//! 3. For each instruction, find all variable references and update RWU state
//! 4. Merge liveness information from successor blocks
//! 5. Iterate until reaching a fixed point (for loops)
//!
//! # Exception Flow Analysis
//!
//! The analysis includes basic support for try-catch exception flow:
//! - Detects when assignments are inside try blocks with exception handlers
//! - Conservatively keeps assignments live if they might throw before completing
//! - Prevents false positives for cases like: `let x = 'init'; try { x = call(); } catch {}`
//!
//! ## Exception Flow Analysis
//!
//! The implementation makes the **conservative assumption** that any statement within
//! a try block can potentially throw an exception. This simplifies the analysis while
//! remaining correct:
//!
//! ```javascript
//! try {
//!     bar = 2;        // Might not complete if an exception occurs
//!     unsafeFn();     // Can throw
//!     bar = 4;        // Might not be reached
//! } catch {}
//! return bar;         // Might return 2
//! ```
//!
//! The CFG provides error edges from blocks to catch handlers, and we conservatively
//! treat all statements in try blocks as potentially throwing. This avoids false
//! positives without requiring complex analysis of which specific expressions can throw.
//!
//! # Current Status: Work in Progress
//!
//! This implementation is incomplete and has known issues:
//!
//! ## Exception Flow Implementation
//!
//! The liveness analysis handles exception flow by making a conservative assumption:
//! **any statement in a try block can potentially throw**. This is implemented by:
//!
//! 1. Checking `block_has_exception_handlers()` to detect if a block is in a try statement
//! 2. Not killing liveness for writes in try blocks (the write might not complete)
//! 3. Not flagging writes as useless if they're in try blocks (they might be observable)
//! 4. Checking `is_in_try_block()` for later writes to determine if they might not complete
//!
//! This approach is simple, conservative, and correct - it may miss some dead assignments
//! in try blocks where we could prove a statement can't throw, but it avoids false positives.
//!
//! ## TODO: AST Traversal
//! - The `visit_node_for_references` function needs to properly traverse the AST
//! - Currently it only handles `IdentifierReference` directly without recursion
//! - Need to recursively walk all child nodes to find all references
//!
//! ## TODO: Proper Program Point Granularity
//! - Currently maps BasicBlockId directly to liveness states
//! - Rustc uses finer-grained "LiveNode" concept representing program points
//! - Multiple program points can exist within one basic block
//! - This coarser granularity may miss some dead assignments within blocks
//!
//! ## TODO: Testing
//! - No integration tests exist for this module
//! - Need tests for loops, conditionals, early exits, etc.
//!
//! NOTE: This module is not currently used by the active liveness rule, which uses
//! a simpler reference-based approach in mod.rs.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, BindingPatternKind, IdentifierReference, *};
use oxc_cfg::graph::Direction;
use oxc_cfg::graph::visit::{EdgeRef, IntoNodeIdentifiers};
use oxc_cfg::{
    BasicBlockId, BlockNodeId, ControlFlowGraph, EdgeType, Instruction, InstructionKind,
};
use oxc_semantic::{AstNodes, ScopeId, Scoping, SymbolId};
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;
use rustc_hash::{FxHashMap, FxHashSet};

use super::cfg_segmenter::CFGSegment;
use super::rwu_table::{RWUTable, ReadWriteUseData};

/// Access mode flags for variable accesses
const ACC_READ: u32 = 1;
const ACC_WRITE: u32 = 2;
const ACC_USE: u32 = 4;

/// Liveness analysis state for a function
pub struct Liveness<'a> {
    /// The control flow graph
    cfg: &'a ControlFlowGraph,
    /// Scoping and symbol information
    scoping: &'a Scoping,
    /// AST nodes for looking up node kinds
    nodes: &'a AstNodes<'a>,
    /// RWU table tracking read/write/use for each block and variable
    rwu_table: RWUTable,
    /// Map from CFG block node ID to basic block ID
    block_map: FxHashMap<BlockNodeId, BasicBlockId>,
    /// Exit blocks (blocks with no normal successors)
    exit_blocks: Vec<BlockNodeId>,
    /// Map from global SymbolId to local RWU table index
    symbol_to_local: FxHashMap<SymbolId, usize>,
    /// List of symbols being analyzed (in local index order)
    local_symbols: Vec<SymbolId>,
    /// Track which variables are live AFTER each write instruction
    /// Map: (NodeId of write instruction) -> Set of live variables after that write
    live_after_write: FxHashMap<NodeId, FxHashSet<SymbolId>>,
}

impl<'a> Liveness<'a> {
    /// Create a new liveness analysis for a CFG segment
    ///
    /// Only analyzes symbols declared in the segment's scope, not all symbols in the program.
    pub fn new(
        cfg: &'a ControlFlowGraph,
        segment: &CFGSegment,
        scoping: &'a Scoping,
        nodes: &'a AstNodes<'a>,
    ) -> Self {
        let num_blocks = cfg.basic_blocks.len();

        // Get only symbols declared in this scope and its children
        let local_symbols = Self::collect_scope_symbols(segment.scope_id, scoping);
        let num_vars = local_symbols.len();

        // #[cfg(debug_assertions)]
        // {
        //     eprintln!("Segment scope: {:?}, symbols: {}", segment.scope_id, num_vars);
        //     eprintln!(
        //         "  Direct bindings in scope: {}",
        //         scoping.iter_bindings_in(segment.scope_id).count()
        //     );
        // }

        // Build mapping from global SymbolId to local RWU table index
        let symbol_to_local: FxHashMap<SymbolId, usize> =
            local_symbols.iter().enumerate().map(|(i, &sym)| (sym, i)).collect();

        // Build a map from block node IDs to basic block IDs
        // Only include blocks in this segment
        let mut block_map = FxHashMap::default();
        for &node_id in &segment.blocks {
            if let Some(&block_id) = cfg.graph().node_weight(node_id) {
                block_map.insert(node_id, block_id);
            }
        }

        // Create RWU table
        let rwu_table = RWUTable::new(num_blocks, num_vars);

        // Find exit blocks (blocks with no successors within this segment)
        let exit_blocks = segment.exit_blocks.clone();

        Self {
            cfg,
            scoping,
            nodes,
            rwu_table,
            block_map,
            exit_blocks,
            symbol_to_local,
            local_symbols,
            live_after_write: FxHashMap::default(),
        }
    }

    /// Collect all symbols declared in a scope and its children
    fn collect_scope_symbols(scope_id: ScopeId, scoping: &Scoping) -> Vec<SymbolId> {
        let mut symbols = Vec::new();

        // Add symbols from this scope
        for symbol_id in scoping.iter_bindings_in(scope_id) {
            symbols.push(symbol_id);
        }

        // Recursively add symbols from child scopes
        if scoping.has_scope_child_ids() {
            for child_scope_id in scoping.get_scope_child_ids(scope_id) {
                symbols.extend(Self::collect_scope_symbols(*child_scope_id, scoping));
            }
        }

        symbols
    }

    /// Check if a symbol is captured by a closure
    ///
    /// A symbol is captured if any reference to it occurs inside a function/arrow function
    /// that is nested within the symbol's declaration scope.
    ///
    /// Returns (has_reads, has_writes) indicating whether the closure reads or writes the variable.
    fn is_captured_by_closure(&self, symbol_id: SymbolId) -> (bool, bool) {
        // Get the scope where the symbol is declared
        let symbol_scope_id = self.scoping.symbol_scope_id(symbol_id);

        // Get all references to this symbol
        let reference_ids = self.scoping.get_resolved_reference_ids(symbol_id);

        let mut has_closure_reads = false;
        let mut has_closure_writes = false;

        for &reference_id in reference_ids {
            let reference = self.scoping.get_reference(reference_id);
            let ref_node_id = reference.node_id();

            // Walk up from the reference node to find if there's a function boundary
            // between the reference and the symbol's scope
            if self.crosses_function_boundary(ref_node_id, symbol_scope_id) {
                // This reference is in a closure - check if it's a read or write
                if reference.is_read() {
                    has_closure_reads = true;
                }
                if reference.is_write() {
                    has_closure_writes = true;
                }
            }
        }

        (has_closure_reads, has_closure_writes)
    }

    /// Check if a symbol has any READ references in closures
    ///
    /// This is used to determine if a variable's value might be read by a closure,
    /// which would make assignments to it potentially useful even if they appear
    /// dead in the local control flow.
    fn has_closure_reads(&self, symbol_id: SymbolId) -> bool {
        let (has_reads, _) = self.is_captured_by_closure(symbol_id);
        has_reads
    }

    /// Check if a block has exception handlers (outgoing Error edges)
    ///
    /// Returns true if the block has any outgoing **Explicit** Error edges, which means
    /// it's inside a try block and exceptions can be caught.
    ///
    /// Note: We check for Explicit edges only, not Implicit ones, because Implicit
    /// edges exist for all blocks (program-level error handling) but we only care
    /// about try-catch blocks.
    fn block_has_exception_handlers(&self, node: BlockNodeId) -> bool {
        use oxc_cfg::ErrorEdgeKind;

        for edge in self.cfg.graph().edges_directed(node, Direction::Outgoing) {
            if matches!(edge.weight(), EdgeType::Error(ErrorEdgeKind::Explicit)) {
                return true;
            }
        }
        false
    }

    /// Check if a node is inside a try statement block
    ///
    /// Walks up the AST from the given node to see if it's contained within
    /// a TryStatement's block (not the catch or finally blocks).
    fn is_in_try_block(&self, node_id: NodeId) -> bool {
        let mut current_node_id = node_id;

        loop {
            let node = self.nodes.get_node(current_node_id);

            // Check parent node
            let parent_id = self.nodes.parent_id(current_node_id);

            // Reached root
            if parent_id == current_node_id {
                return false;
            }

            let parent_node = self.nodes.get_node(parent_id);

            // Check if parent is a TryStatement and we're in its block
            if let AstKind::TryStatement(try_stmt) = parent_node.kind() {
                // Check if current node is within the try block
                // We need to check if we're in the block, not in catch or finally
                let try_block_span = try_stmt.block.span;
                let current_span = node.kind().span();

                // If our node is within the try block's span, we're in a try block
                if try_block_span.contains_inclusive(current_span) {
                    return true;
                }
            }

            current_node_id = parent_id;
        }
    }

    /// Check if a variable has any writes inside try blocks that come AFTER the current write
    /// AND can potentially throw
    ///
    /// This is used to conservatively determine if initial assignments to a variable
    /// should be kept alive because later assignments in try blocks might throw,
    /// leaving the initial value as the one that gets used.
    ///
    /// # Arguments
    /// * `symbol_id` - The variable symbol to check
    /// * `current_write_node` - The current write's NodeId (we only check for writes AFTER this)
    ///
    /// # Implementation Details
    ///
    /// This function enables detection of useless writes in try-catch scenarios:
    ///
    /// **Case 1: Throwing write protects earlier values**
    /// ```js
    /// let x = 'init';
    /// try { x = call().value; } catch {}  // call() might throw
    /// console.log(x);  // Could print 'init'
    /// // Result: 'init' assignment is kept (not flagged as useless)
    /// ```
    ///
    /// **Case 2: Non-throwing writes don't protect earlier values**
    /// ```js
    /// try {
    ///     bar = 2;  // Literal assignment, cannot throw
    ///     bar = 4;  // Literal assignment, cannot throw
    /// } catch {}
    /// // Result: bar = 2 is correctly flagged as useless
    /// ```
    ///
    /// We check if later writes can throw using `can_node_throw()` to distinguish these cases.
    fn has_later_writes_in_try_blocks(
        &self,
        symbol_id: SymbolId,
        current_write_node: NodeId,
    ) -> bool {
        // Get the span of the current write to determine what comes "after" it
        let current_span = self.nodes.get_node(current_write_node).kind().span();

        // Get all references to this symbol
        let reference_ids = self.scoping.get_resolved_reference_ids(symbol_id);

        // Check each reference to see if it's a write in a try block that comes after current write
        for reference_id in reference_ids {
            let reference = self.scoping.get_reference(*reference_id);

            // Only interested in writes
            if !reference.is_write() {
                continue;
            }

            let ref_node_id = reference.node_id();
            let ref_span = self.nodes.get_node(ref_node_id).kind().span();

            // Skip if this write comes before or at the same position as current write
            // We only care about writes that come AFTER (in program order)
            if ref_span.start <= current_span.start {
                continue;
            }

            // If this later write is in a try block, it might not complete due to an exception.
            // We conservatively assume any statement in a try block can throw.
            // This handles both:
            // 1. The write itself throwing: x = call()
            // 2. Statements between writes throwing: x = 1; unsafeFn(); x = 2;
            if self.is_in_try_block(ref_node_id) {
                return true;
            }
        }

        false
    }

    /// Check if walking from a node up to a target scope crosses a function boundary
    fn crosses_function_boundary(&self, start_node_id: NodeId, target_scope_id: ScopeId) -> bool {
        let target_scope_node_id = self.scoping.get_node_id(target_scope_id);

        // Walk up the AST from start_node_id towards target_scope_node_id
        let mut current_node_id = start_node_id;

        while current_node_id != target_scope_node_id {
            let node = self.nodes.get_node(current_node_id);

            // Check if this node creates a new function scope
            match node.kind() {
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    // Found a function boundary - the reference is in a nested function
                    return true;
                }
                _ => {}
            }

            // Move to parent node
            let parent_id = self.nodes.parent_id(current_node_id);

            // Check if we've reached the root (parent is itself)
            if parent_id == current_node_id {
                break;
            }

            current_node_id = parent_id;
        }

        false
    }

    /// Compute liveness for all variables in the function
    ///
    /// Uses a worklist algorithm with fixed-point iteration to handle loops.
    /// Processes blocks in reverse execution order (from exits to entry).
    pub fn compute(&mut self) {
        // Phase 1: Run block-level dataflow to convergence
        self.compute_block_level_liveness();

        // Phase 2: Track instruction-level liveness for useless assignment detection
        self.compute_instruction_level_liveness();
    }

    /// Helper for post-order traversal
    fn post_order_visit(
        &self,
        node: BlockNodeId,
        visited: &mut FxHashSet<BlockNodeId>,
        post_order: &mut Vec<BlockNodeId>,
    ) {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);

        // Visit successors first
        for succ_edge in self.cfg.graph().edges_directed(node, Direction::Outgoing) {
            if matches!(succ_edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable) {
                continue;
            }
            let succ = succ_edge.target();
            self.post_order_visit(succ, visited, post_order);
        }

        // Add this node after all successors
        post_order.push(node);
    }

    /// Phase 1: Compute block-level liveness (live-in/live-out for each block)
    ///
    /// This implements a worklist-based fixed-point iteration algorithm similar to rustc's
    /// dataflow framework. The algorithm:
    /// 1. Initializes all blocks to bottom (no variables live)
    /// 2. Processes blocks in postorder (successors before predecessors) for faster convergence
    /// 3. For each block, computes IN from OUT and the block's gen/kill sets
    /// 4. If IN changed, adds predecessors to worklist
    /// 5. Repeats until worklist is empty (fixed point reached)
    fn compute_block_level_liveness(&mut self) {
        // Compute postorder traversal for backward dataflow
        // Processing successors before predecessors gives better convergence
        let mut post_order = Vec::new();
        let mut visited = FxHashSet::default();

        // Start from exit blocks (which are in this segment)
        for &exit_node in &self.exit_blocks {
            self.post_order_visit(exit_node, &mut visited, &mut post_order);
        }

        // Also visit any blocks in THIS SEGMENT not reachable from exits
        for &node in self.block_map.keys() {
            if !visited.contains(&node) {
                self.post_order_visit(node, &mut visited, &mut post_order);
            }
        }

        // Initialize worklist with all blocks in postorder
        // Use VecDeque for FIFO processing, HashSet for O(1) membership checks
        use std::collections::VecDeque;
        let mut worklist: VecDeque<BlockNodeId> = post_order.iter().copied().collect();
        let mut in_worklist: FxHashSet<BlockNodeId> = post_order.iter().copied().collect();

        // Process blocks until worklist is empty (fixed point reached)
        // Note: Unlike some implementations, we don't impose a MAX_ITERATIONS limit.
        // The algorithm is guaranteed to converge for monotone frameworks (which liveness is),
        // so an iteration limit would only hide bugs. If this doesn't converge, there's a bug
        // in the transfer function or lattice operations.
        #[cfg(debug_assertions)]
        let mut iteration = 0;

        while let Some(node) = worklist.pop_front() {
            #[cfg(debug_assertions)]
            {
                iteration += 1;
            }

            in_worklist.remove(&node);

            // Propagate liveness through this block
            // Returns true if the block's IN state changed
            if self.propagate_block_simple(node) {
                // State changed - add predecessors to worklist
                for pred_edge in self.cfg.graph().edges_directed(node, Direction::Incoming) {
                    // Skip function boundaries and unreachable edges
                    if matches!(pred_edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable) {
                        continue;
                    }

                    let pred = pred_edge.source();
                    // Only add if not already in worklist
                    if in_worklist.insert(pred) {
                        worklist.push_back(pred);
                    }
                }
            }
        }

        // #[cfg(debug_assertions)]
        // eprintln!(
        //     "Liveness analysis converged in {iteration} iterations (blocks: {}, vars: {})",
        //     self.block_map.len(),
        //     self.local_symbols.len()
        // );
    }

    /// Phase 2: Compute instruction-level liveness for useless assignment detection
    /// This runs AFTER block-level liveness has converged
    fn compute_instruction_level_liveness(&mut self) {
        // Collect blocks in postorder (successors before predecessors)
        // This ensures we have computed successor liveness before processing each block
        let mut visited = FxHashSet::default();
        let mut post_order = Vec::new();

        // Start from exit blocks to ensure proper ordering
        // Exit blocks have no successors (within this segment)
        for &exit_node in &self.exit_blocks {
            if !visited.contains(&exit_node) {
                self.post_order_visit(exit_node, &mut visited, &mut post_order);
            }
        }

        // Also process any remaining unvisited blocks in THIS SEGMENT
        for &start_node in self.block_map.keys() {
            if !visited.contains(&start_node) {
                self.post_order_visit(start_node, &mut visited, &mut post_order);
            }
        }

        // Process blocks in post-order (successors before predecessors)
        // This is correct for backwards dataflow
        for &node in &post_order {
            let Some(&block_id) = self.block_map.get(&node) else {
                continue;
            };

            let basic_block = self.cfg.basic_block(node);

            // Skip unreachable blocks - assignments in unreachable code are by definition useless
            // but we don't want to report them because the entire block is unreachable
            if basic_block.is_unreachable() {
                continue;
            }

            // Create a working set of live variables starting from this block's live-out
            // The live-out was already computed in Phase 1 and stored in the RWU table
            //
            // NOTE: We use the CURRENT block's state, not successors', because Phase 1
            // already propagated liveness through the CFG. The RWU table entry for this
            // block represents its live-in, and we need to reconstruct live-out from it.
            //
            // Actually, the RWU table stores live-IN for each block. To get live-OUT,
            // we need to merge live-IN from all successors, which Phase 1 already did
            // and stored in the RWU table. But that's the LIVE-IN for THIS block after
            // processing instructions.
            //
            // Let me rethink: Phase 1 computes live-IN for each block by:
            // 1. Start with live-OUT = union of successors' live-IN
            // 2. Process instructions backwards
            // 3. Result is live-IN
            //
            // For Phase 2, we want to start with live-OUT. But Phase 1 doesn't store
            // live-OUT explicitly. We need to recompute it from successors.

            let mut live_set: FxHashSet<usize> = FxHashSet::default();

            // Compute live-out = union of successors' live-in
            for successor_edge in self.cfg.graph().edges_directed(node, Direction::Outgoing) {
                // Skip function boundaries and unreachable edges
                if matches!(successor_edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable)
                {
                    continue;
                }

                let succ_node = successor_edge.target();
                if let Some(&succ_block_id) = self.block_map.get(&succ_node) {
                    // Add all live variables from successor's live-in (stored as "reader" in RWU)
                    for i in 0..self.local_symbols.len() {
                        let local_var = SymbolId::new(i);
                        if self.rwu_table.get_reader(succ_block_id, local_var) {
                            live_set.insert(i);
                        }
                    }
                }
            }

            // Check if this block has exception handlers (is in a try block)
            let has_exception_handlers = self.block_has_exception_handlers(node);

            // Process instructions in reverse
            for instruction in basic_block.instructions().iter().rev() {
                if let Some(node_id) = instruction.node_id {
                    self.process_instruction_with_live_set(
                        block_id,
                        node_id,
                        &mut live_set,
                        instruction,
                        has_exception_handlers,
                    );
                }
            }
        }
    }

    /// Propagate liveness information for a single block (block-level only)
    /// Returns true if the block's liveness information changed
    fn propagate_block_simple(&mut self, node: BlockNodeId) -> bool {
        let Some(&block_id) = self.block_map.get(&node) else {
            return false;
        };

        let basic_block = self.cfg.basic_block(node);

        // Skip unreachable blocks - no point analyzing them
        if basic_block.is_unreachable() {
            return false;
        }

        // Save the old state to detect changes
        let old_state = self.get_block_state(block_id);

        // Compute OUT[B] = union of IN of all successors
        // We store this temporarily in the block's RWU table, but we'll compute IN[B] separately
        let mut out_state = vec![
            ReadWriteUseData { reader: false, writer: false, used: false };
            self.local_symbols.len()
        ];

        for successor_edge in self.cfg.graph().edges_directed(node, Direction::Outgoing) {
            // Skip function boundaries and unreachable edges
            if matches!(successor_edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable) {
                continue;
            }

            let succ_node = successor_edge.target();
            if let Some(&succ_block_id) = self.block_map.get(&succ_node) {
                // Union successor's IN into our OUT
                for i in 0..self.local_symbols.len() {
                    let local_var = SymbolId::new(i);
                    let succ_rwu = self.rwu_table.get(succ_block_id, local_var);
                    out_state[i].reader |= succ_rwu.reader;
                    out_state[i].writer |= succ_rwu.writer;
                    out_state[i].used |= succ_rwu.used;
                }
            }
        }

        // Start with OUT[B] as the base
        let mut in_state = out_state;

        // Process instructions in reverse to compute IN[B] from OUT[B]
        // Backward dataflow: IN[B] = USE[B] ∪ (OUT[B] - DEF[B])
        for instruction in basic_block.instructions().iter().rev() {
            if let Some(node_id) = instruction.node_id {
                // Process writes FIRST (going backwards): these kill liveness
                let writes = self.find_writes_in_node(node_id);
                for (symbol_id, _) in &writes {
                    if let Some(&local_idx) = self.symbol_to_local.get(symbol_id) {
                        // Write kills liveness (DEF)
                        in_state[local_idx].reader = false;
                        in_state[local_idx].writer = true;
                    }
                }

                // Process reads AFTER writes (going backwards): these generate liveness
                let reads = self.find_reads_in_node(node_id);
                for symbol_id in &reads {
                    if let Some(&local_idx) = self.symbol_to_local.get(symbol_id) {
                        // Read makes variable live (USE)
                        in_state[local_idx].reader = true;
                        in_state[local_idx].used = true;
                    }
                }
            }
        }

        // Update the RWU table with the computed IN[B]
        for (i, &rwu) in in_state.iter().enumerate() {
            let local_var = SymbolId::new(i);
            self.rwu_table.set(block_id, local_var, rwu);
        }

        // Check if anything changed
        !Self::states_equal(&old_state, &in_state)
    }

    /// Get a snapshot of the block's RWU state for all variables
    fn get_block_state(&self, block_id: BasicBlockId) -> Vec<ReadWriteUseData> {
        (0..self.local_symbols.len())
            .map(|i| self.rwu_table.get(block_id, SymbolId::new(i)))
            .collect()
    }

    /// Compare two RWU state vectors for equality
    fn states_equal(a: &[ReadWriteUseData], b: &[ReadWriteUseData]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter()
            .zip(b.iter())
            .all(|(x, y)| x.reader == y.reader && x.writer == y.writer && x.used == y.used)
    }

    /// Process an instruction with instruction-level liveness tracking
    ///
    /// Backward dataflow: process writes FIRST, then reads
    /// - First check writes: if variable not live, mark as useless, then kill liveness
    /// - Then process reads: make variables live
    ///
    /// For `i++` (read-modify-write):
    /// - Write happens first (going backwards): check liveness, then kill
    /// - Read happens second (going backwards): make live
    ///
    /// This ensures the PREVIOUS write sees `i` as live
    fn process_instruction_with_live_set(
        &mut self,
        _block_id: BasicBlockId,
        node_id: NodeId,
        live_set: &mut FxHashSet<usize>,
        instruction: &Instruction,
        has_exception_handlers: bool,
    ) {
        // Step 1: Process writes FIRST (backwards dataflow)
        let writes = self.find_writes_in_node(node_id);

        // For each write, check if the variable is currently live
        for (symbol_id, write_node_id) in &writes {
            if let Some(&local_idx) = self.symbol_to_local.get(symbol_id) {
                // If the variable is NOT currently live, this write is useless
                if !live_set.contains(&local_idx) {
                    // In a try block, assume any statement can throw and prevent the write
                    // from completing, so we conservatively don't flag it as useless
                    if has_exception_handlers {
                        continue;
                    }

                    // Conservative check for try-catch exception flow:
                    // If there are later writes to this variable in try blocks that might throw,
                    // we can't flag this write as useless because the later write might not complete.
                    //
                    // Example where this matters:
                    //   let x = 'init';              // NOT useless!
                    //   try { x = call(); } catch {} // call() might throw
                    //   console.log(x);              // Might print 'init'
                    //
                    // However, conditional branches (without exceptions) are handled correctly
                    // by the CFG-based analysis, so we DON'T need to be conservative there.
                    if self.has_later_writes_in_try_blocks(*symbol_id, *write_node_id) {
                        continue;
                    }

                    self.live_after_write.entry(*write_node_id).or_default().insert(*symbol_id);
                }
            }
        }

        // Kill liveness for writes
        // BUT: in a try block, any statement can throw and prevent the write from
        // completing, so DON'T kill liveness
        if !has_exception_handlers {
            for (symbol_id, write_node_id) in &writes {
                if let Some(&local_idx) = self.symbol_to_local.get(symbol_id) {
                    // Don't kill liveness if there are later writes in try blocks that might throw,
                    // because those writes might not complete and earlier values could still be used.
                    if self.has_later_writes_in_try_blocks(*symbol_id, *write_node_id) {
                        continue;
                    }
                    live_set.remove(&local_idx);
                }
            }
        }

        // Step 2: Process reads AFTER writes (going backwards, reads come "before" writes)
        let reads = self.find_reads_in_node(node_id);
        for symbol_id in reads {
            if let Some(&local_idx) = self.symbol_to_local.get(&symbol_id) {
                live_set.insert(local_idx);
            }
        }
    }

    /// Find all read operations within an AST node
    fn find_reads_in_node(&self, node_id: NodeId) -> Vec<SymbolId> {
        let mut reads = Vec::new();
        self.collect_reads(node_id, &mut reads);
        reads
    }

    /// Recursively collect all reads from a node
    fn collect_reads(&self, node_id: NodeId, reads: &mut Vec<SymbolId>) {
        let node = self.nodes.get_node(node_id);
        self.collect_reads_from_kind(node.kind(), reads);
    }

    /// Helper to collect reads from a specific AST kind
    fn collect_reads_from_kind(&self, kind: AstKind<'a>, reads: &mut Vec<SymbolId>) {
        match kind {
            AstKind::IdentifierReference(ident) => {
                if let Some(reference_id) = ident.reference_id.get() {
                    let reference = self.scoping.get_reference(reference_id);
                    if reference.is_read()
                        && let Some(symbol_id) = reference.symbol_id()
                    {
                        reads.push(symbol_id);
                    }
                }
            }
            // Statement wrappers - unwrap to find reads
            AstKind::ExpressionStatement(stmt) => {
                self.propagate_through_expr(&stmt.expression, reads);
            }
            AstKind::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    self.propagate_through_expr(arg, reads);
                }
            }
            AstKind::VariableDeclaration(decl) => {
                // Variable initializers contain reads
                for declarator in &decl.declarations {
                    if let Some(init) = &declarator.init {
                        self.propagate_through_expr(init, reads);
                    }
                }
            }
            // Recursively handle complex expressions
            AstKind::BinaryExpression(bin) => {
                self.propagate_through_expr(&bin.left, reads);
                self.propagate_through_expr(&bin.right, reads);
            }
            AstKind::LogicalExpression(log) => {
                self.propagate_through_expr(&log.left, reads);
                self.propagate_through_expr(&log.right, reads);
            }
            AstKind::CallExpression(call) => {
                self.propagate_through_expr(&call.callee, reads);
                for arg in &call.arguments {
                    match arg {
                        Argument::SpreadElement(spread) => {
                            self.propagate_through_expr(&spread.argument, reads);
                        }
                        arg => {
                            if let Some(expr) = arg.as_expression() {
                                self.propagate_through_expr(expr, reads);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Find all write operations within an AST node and return (SymbolId, NodeId) pairs
    fn find_writes_in_node(&self, node_id: NodeId) -> Vec<(SymbolId, NodeId)> {
        let mut writes = Vec::new();
        self.collect_writes(node_id, &mut writes);
        writes
    }

    /// Recursively collect all writes from a node
    fn collect_writes(&self, node_id: NodeId, writes: &mut Vec<(SymbolId, NodeId)>) {
        let node = self.nodes.get_node(node_id);
        let kind = node.kind();

        // Use the existing visitor to traverse and find writes
        self.collect_writes_from_kind(node_id, kind, writes);
    }

    /// Helper to collect writes from a specific AST kind
    fn collect_writes_from_kind(
        &self,
        node_id: NodeId,
        kind: AstKind<'a>,
        writes: &mut Vec<(SymbolId, NodeId)>,
    ) {
        match kind {
            AstKind::IdentifierReference(ident) => {
                // Check if this reference is a write
                if let Some(reference_id) = ident.reference_id.get() {
                    let reference = self.scoping.get_reference(reference_id);
                    // Collect ANY write, even if it's also a read (like i++)
                    if reference.is_write()
                        && let Some(symbol_id) = reference.symbol_id()
                    {
                        writes.push((symbol_id, node_id));
                    }
                }
            }
            // Statement wrappers - need to unwrap to find the actual operation
            AstKind::ExpressionStatement(stmt) => {
                self.collect_writes_from_expression(&stmt.expression, writes);
            }
            AstKind::VariableDeclaration(decl) => {
                // Process each declarator
                for declarator in &decl.declarations {
                    // Variable declarations with initializers are writes!
                    if declarator.init.is_some() {
                        // Extract the symbol from the binding pattern
                        self.collect_writes_from_binding_pattern(&declarator.id, node_id, writes);
                    }
                    // Also check for nested writes in the initializer
                    if let Some(init) = &declarator.init {
                        self.collect_writes_from_expression(init, writes);
                    }
                }
            }
            // For complex nodes, recurse through their structure
            AstKind::AssignmentExpression(assign) => {
                // The left side contains writes
                self.collect_writes_from_assignment_target(&assign.left, node_id, writes);
                // Also check the right side for nested writes
                self.collect_writes_from_expression(&assign.right, writes);
            }
            AstKind::UpdateExpression(update) => {
                self.collect_writes_from_simple_assignment_target(&update.argument, writes);
            }
            AstKind::VariableDeclarator(decl) => {
                // Variable declarations with initializers are writes!
                // The binding pattern defines what's being written to
                if decl.init.is_some() {
                    // Extract the symbol from the binding pattern
                    self.collect_writes_from_binding_pattern(&decl.id, node_id, writes);
                }
                // Also check for nested writes in the initializer
                if let Some(init) = &decl.init {
                    self.collect_writes_from_expression(init, writes);
                }
            }
            // Add other cases as needed, but for now this covers the main write scenarios
            _ => {}
        }
    }

    /// Collect writes from a binding pattern (used in variable declarations)
    fn collect_writes_from_binding_pattern(
        &self,
        pattern: &BindingPattern<'a>,
        node_id: NodeId,
        writes: &mut Vec<(SymbolId, NodeId)>,
    ) {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                // This is the simple case: let x = value
                // Find the symbol for this binding
                if let Some(symbol_id) = ident.symbol_id.get() {
                    writes.push((symbol_id, node_id));
                }
            }
            BindingPatternKind::ObjectPattern(obj) => {
                // Handle object destructuring: let { x, y } = obj
                for prop in &obj.properties {
                    // Each property has a pattern as its value
                    self.collect_writes_from_binding_pattern(&prop.value, node_id, writes);
                }
                // Handle rest: let { ...rest } = obj
                if let Some(rest) = &obj.rest {
                    self.collect_writes_from_binding_pattern(&rest.argument, node_id, writes);
                }
            }
            BindingPatternKind::ArrayPattern(arr) => {
                // Handle array destructuring: let [x, y] = arr
                for element in (&arr.elements).into_iter().flatten() {
                    self.collect_writes_from_binding_pattern(element, node_id, writes);
                }
                // Handle rest: let [a, ...rest] = arr
                if let Some(rest) = &arr.rest {
                    self.collect_writes_from_binding_pattern(&rest.argument, node_id, writes);
                }
            }
            BindingPatternKind::AssignmentPattern(assign) => {
                // Handle patterns with defaults: let x = value
                self.collect_writes_from_binding_pattern(&assign.left, node_id, writes);
            }
        }
    }

    fn collect_writes_from_assignment_target_maybe_default(
        &self,
        target: &AssignmentTargetMaybeDefault<'a>,
        node_id: NodeId,
        writes: &mut Vec<(SymbolId, NodeId)>,
    ) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                // Has a default value like { a = 10 } = obj
                self.collect_writes_from_assignment_target(&with_default.binding, node_id, writes);
            }
            target => {
                // No default - just a regular assignment target
                if let Some(simple_target) = target.as_assignment_target() {
                    self.collect_writes_from_assignment_target(simple_target, node_id, writes);
                }
            }
        }
    }

    fn collect_writes_from_assignment_target(
        &self,
        target: &AssignmentTarget<'a>,
        node_id: NodeId,
        writes: &mut Vec<(SymbolId, NodeId)>,
    ) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                if let Some(reference_id) = ident.reference_id.get() {
                    let reference = self.scoping.get_reference(reference_id);
                    if let Some(symbol_id) = reference.symbol_id() {
                        // Use the provided node_id (parent assignment/declaration)
                        writes.push((symbol_id, node_id));
                    }
                }
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                // Handle array destructuring: [a, b, c] = fn()
                for target in (&arr.elements).into_iter().flatten() {
                    self.collect_writes_from_assignment_target_maybe_default(
                        target, node_id, writes,
                    );
                }
                // Handle rest element: [a, ...rest] = fn()
                if let Some(rest) = &arr.rest {
                    self.collect_writes_from_assignment_target(&rest.target, node_id, writes);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                // Handle object destructuring: { a, b, c } = fn()
                for prop in &obj.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                            // Shorthand: { a } = fn()
                            if let Some(reference_id) = ident.binding.reference_id.get() {
                                let reference = self.scoping.get_reference(reference_id);
                                if let Some(symbol_id) = reference.symbol_id() {
                                    writes.push((symbol_id, node_id));
                                }
                            }
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                            // Full form: { foo: a } = fn()
                            self.collect_writes_from_assignment_target_maybe_default(
                                &prop.binding,
                                node_id,
                                writes,
                            );
                        }
                    }
                }
                // Handle rest element: { a, ...rest } = fn()
                if let Some(rest) = &obj.rest {
                    self.collect_writes_from_assignment_target(&rest.target, node_id, writes);
                }
            }
            // Member expressions like obj.prop or arr[i]
            // These don't write to tracked local variables, but may have reads in the object/index
            AssignmentTarget::StaticMemberExpression(_)
            | AssignmentTarget::PrivateFieldExpression(_)
            | AssignmentTarget::ComputedMemberExpression(_) => {
                // Member assignment like obj.x = value
                // We don't track object properties, so skip
            }
            // TypeScript type assertions - these wrap expressions
            // For writes, we need to find IdentifierReferences in the wrapped expression
            // This is complex, so for now we skip (TS type assertions in assignment targets are rare)
            AssignmentTarget::TSAsExpression(_)
            | AssignmentTarget::TSSatisfiesExpression(_)
            | AssignmentTarget::TSNonNullExpression(_)
            | AssignmentTarget::TSTypeAssertion(_) => {
                // TODO: Handle TS type assertions in assignment targets
                // For now skip - these are rare in practice
            }
        }
    }

    fn collect_writes_from_simple_assignment_target(
        &self,
        target: &SimpleAssignmentTarget<'a>,
        writes: &mut Vec<(SymbolId, NodeId)>,
    ) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target
            && let Some(reference_id) = ident.reference_id.get()
        {
            let reference = self.scoping.get_reference(reference_id);
            if let Some(symbol_id) = reference.symbol_id() {
                let node_id = self
                    .nodes
                    .iter()
                    .find(|n| {
                        if let AstKind::IdentifierReference(ref_ident) = n.kind() {
                            ref_ident.reference_id.get() == Some(reference_id)
                        } else {
                            false
                        }
                    })
                    .map_or(NodeId::DUMMY, oxc_semantic::AstNode::id);

                if node_id != NodeId::DUMMY {
                    writes.push((symbol_id, node_id));
                }
            }
        }
    }

    fn collect_writes_from_expression(
        &self,
        expr: &Expression<'a>,
        writes: &mut Vec<(SymbolId, NodeId)>,
    ) {
        // Recursively check for writes in nested expressions
        match expr {
            Expression::AssignmentExpression(assign) => {
                // Find the NodeId for this AssignmentExpression
                let expr_node_id = self.find_node_id_for_expression(expr);
                self.collect_writes_from_assignment_target(&assign.left, expr_node_id, writes);
                self.collect_writes_from_expression(&assign.right, writes);
            }
            Expression::UpdateExpression(update) => {
                self.collect_writes_from_simple_assignment_target(&update.argument, writes);
            }
            Expression::ParenthesizedExpression(paren) => {
                // Unwrap parenthesized expressions
                self.collect_writes_from_expression(&paren.expression, writes);
            }
            _ => {}
        }
    }

    /// Find the NodeId for an expression by searching through the AST nodes
    fn find_node_id_for_expression(&self, expr: &Expression<'a>) -> NodeId {
        // Get the span of the expression
        let expr_span = expr.span();

        // Search through nodes to find one with matching span and kind
        // We need to be more flexible with span matching since the span might include
        // the parentheses in a parenthesized expression
        for node in self.nodes.iter() {
            let node_span = node.span();

            // Check if spans match (exactly or contained within)
            if node_span == expr_span || node_span.contains_inclusive(expr_span) {
                match (node.kind(), expr) {
                    (AstKind::AssignmentExpression(_), Expression::AssignmentExpression(_)) => {
                        return node.id();
                    }
                    (AstKind::UpdateExpression(_), Expression::UpdateExpression(_)) => {
                        return node.id();
                    }
                    _ => {}
                }
            }
        }

        // Fallback to DUMMY if not found (shouldn't happen in practice)
        // If this happens, the write won't be tracked properly
        NodeId::DUMMY
    }

    fn collect_reads_from_assignment_target_maybe_default(
        &self,
        target: &AssignmentTargetMaybeDefault<'a>,
        reads: &mut Vec<SymbolId>,
    ) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                // Has a default value like { a = 10 } = obj
                // Read the default value expression
                self.propagate_through_expr(&with_default.init, reads);
                // And process the binding
                self.collect_reads_from_assignment_target(&with_default.binding, reads);
            }
            target => {
                // No default - just a regular assignment target
                if let Some(simple_target) = target.as_assignment_target() {
                    self.collect_reads_from_assignment_target(simple_target, reads);
                }
            }
        }
    }

    fn collect_reads_from_assignment_target(
        &self,
        target: &AssignmentTarget<'a>,
        reads: &mut Vec<SymbolId>,
    ) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                if let Some(reference_id) = ident.reference_id.get() {
                    let reference = self.scoping.get_reference(reference_id);
                    // For compound assignments (+=, -=, etc.), the target is also read
                    if reference.is_read()
                        && let Some(symbol_id) = reference.symbol_id()
                    {
                        reads.push(symbol_id);
                    }
                }
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                // Handle array destructuring reads (for compound assignments)
                for target in (&arr.elements).into_iter().flatten() {
                    self.collect_reads_from_assignment_target_maybe_default(target, reads);
                }
                if let Some(rest) = &arr.rest {
                    self.collect_reads_from_assignment_target(&rest.target, reads);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                // Handle object destructuring reads (for compound assignments)
                for prop in &obj.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                            if let Some(reference_id) = ident.binding.reference_id.get() {
                                let reference = self.scoping.get_reference(reference_id);
                                if reference.is_read()
                                    && let Some(symbol_id) = reference.symbol_id()
                                {
                                    reads.push(symbol_id);
                                }
                            }
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                            self.collect_reads_from_assignment_target_maybe_default(
                                &prop.binding,
                                reads,
                            );
                        }
                    }
                }
                if let Some(rest) = &obj.rest {
                    self.collect_reads_from_assignment_target(&rest.target, reads);
                }
            }
            // Member expressions - may have reads in the object/index part
            AssignmentTarget::StaticMemberExpression(member) => {
                // obj.prop - read obj
                self.propagate_through_expr(&member.object, reads);
            }
            AssignmentTarget::PrivateFieldExpression(member) => {
                // obj.#prop - read obj
                self.propagate_through_expr(&member.object, reads);
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                // obj[expr] - read both obj and expr
                self.propagate_through_expr(&member.object, reads);
                self.propagate_through_expr(&member.expression, reads);
            }
            // TypeScript type assertions - these wrap expressions
            AssignmentTarget::TSAsExpression(as_expr) => {
                // For compound assignments, the wrapped expression may be read
                self.propagate_through_expr(&as_expr.expression, reads);
            }
            AssignmentTarget::TSSatisfiesExpression(sat) => {
                self.propagate_through_expr(&sat.expression, reads);
            }
            AssignmentTarget::TSNonNullExpression(non_null) => {
                self.propagate_through_expr(&non_null.expression, reads);
            }
            AssignmentTarget::TSTypeAssertion(assert) => {
                self.propagate_through_expr(&assert.expression, reads);
            }
        }
    }

    /// Helper to propagate through JSX member expressions like <Foo.Bar />
    fn propagate_through_jsx_member_expr(
        &self,
        member: &JSXMemberExpression<'a>,
        reads: &mut Vec<SymbolId>,
    ) {
        // JSXMemberExpression can be nested: <A.B.C />
        // The object can be either an identifier or another member expression
        match &member.object {
            JSXMemberExpressionObject::IdentifierReference(ident_ref) => {
                if let Some(reference_id) = ident_ref.reference_id.get() {
                    let reference = self.scoping.get_reference(reference_id);
                    if reference.is_read()
                        && let Some(symbol_id) = reference.symbol_id()
                    {
                        reads.push(symbol_id);
                    }
                }
            }
            JSXMemberExpressionObject::MemberExpression(nested) => {
                // Recursively handle nested member expressions
                self.propagate_through_jsx_member_expr(nested, reads);
            }
            JSXMemberExpressionObject::ThisExpression(_) => {
                // <this.foo /> - no variable reference
            }
        }
        // The property is always a JSXIdentifier (not a variable reference)
    }

    /// Propagate liveness through an expression (inspired by rustc's propagate_through_expr)
    ///
    /// This recursively traverses the expression AST to find all variable reads.
    /// The naming follows the Rust compiler's convention.
    fn propagate_through_expr(&self, expr: &Expression<'a>, reads: &mut Vec<SymbolId>) {
        match expr {
            // Identifier - the leaf case where actual reads happen
            Expression::Identifier(ident) => {
                if let Some(reference_id) = ident.reference_id.get() {
                    let reference = self.scoping.get_reference(reference_id);
                    if reference.is_read()
                        && let Some(symbol_id) = reference.symbol_id()
                    {
                        reads.push(symbol_id);
                    }
                }
            }

            // Binary operators - propagate through both operands
            Expression::BinaryExpression(bin) => {
                self.propagate_through_expr(&bin.left, reads);
                self.propagate_through_expr(&bin.right, reads);
            }

            // Logical operators - propagate through both operands
            Expression::LogicalExpression(log) => {
                self.propagate_through_expr(&log.left, reads);
                self.propagate_through_expr(&log.right, reads);
            }

            // Unary operators - propagate through operand
            Expression::UnaryExpression(unary) => {
                self.propagate_through_expr(&unary.argument, reads);
            }

            // Update expressions (++/--) - read the target
            Expression::UpdateExpression(update) => {
                if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &update.argument
                    && let Some(reference_id) = ident.reference_id.get()
                {
                    let reference = self.scoping.get_reference(reference_id);
                    if reference.is_read()
                        && let Some(symbol_id) = reference.symbol_id()
                    {
                        reads.push(symbol_id);
                    }
                }
            }

            // Assignment - right side is read, left may be read for compound assignments
            Expression::AssignmentExpression(assign) => {
                self.propagate_through_expr(&assign.right, reads);
                self.collect_reads_from_assignment_target(&assign.left, reads);
            }

            // Member access - propagate through object and property
            Expression::StaticMemberExpression(member) => {
                self.propagate_through_expr(&member.object, reads);
            }
            Expression::ComputedMemberExpression(member) => {
                self.propagate_through_expr(&member.object, reads);
                self.propagate_through_expr(&member.expression, reads);
            }
            Expression::PrivateFieldExpression(member) => {
                self.propagate_through_expr(&member.object, reads);
            }

            // Function/method calls - propagate through callee and arguments
            Expression::CallExpression(call) => {
                self.propagate_through_expr(&call.callee, reads);
                for arg in &call.arguments {
                    match arg {
                        Argument::SpreadElement(spread) => {
                            self.propagate_through_expr(&spread.argument, reads);
                        }
                        arg => {
                            if let Some(expr) = arg.as_expression() {
                                self.propagate_through_expr(expr, reads);
                            }
                        }
                    }
                }
            }

            // New expression - propagate through callee and arguments
            Expression::NewExpression(new) => {
                // The callee (class/constructor) is read!
                self.propagate_through_expr(&new.callee, reads);
                // Arguments are also read
                for arg in &new.arguments {
                    match arg {
                        Argument::SpreadElement(spread) => {
                            self.propagate_through_expr(&spread.argument, reads);
                        }
                        arg => {
                            if let Some(expr) = arg.as_expression() {
                                self.propagate_through_expr(expr, reads);
                            }
                        }
                    }
                }
            }

            // Conditional (ternary) - propagate through all three parts
            Expression::ConditionalExpression(cond) => {
                self.propagate_through_expr(&cond.test, reads);
                self.propagate_through_expr(&cond.consequent, reads);
                self.propagate_through_expr(&cond.alternate, reads);
            }

            // Array expression - propagate through all elements
            Expression::ArrayExpression(arr) => {
                for element in &arr.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            self.propagate_through_expr(&spread.argument, reads);
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        element => {
                            if let Some(expr) = element.as_expression() {
                                self.propagate_through_expr(expr, reads);
                            }
                        }
                    }
                }
            }

            // Object expression - propagate through property values
            Expression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    match prop {
                        ObjectPropertyKind::ObjectProperty(p) => {
                            // Key may be computed - if so it's an expression
                            if p.computed
                                && let Some(expr) = p.key.as_expression()
                            {
                                self.propagate_through_expr(expr, reads);
                            }
                            self.propagate_through_expr(&p.value, reads);
                        }
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            self.propagate_through_expr(&spread.argument, reads);
                        }
                    }
                }
            }

            // Template literal - propagate through expressions
            Expression::TemplateLiteral(template) => {
                for expr in &template.expressions {
                    self.propagate_through_expr(expr, reads);
                }
            }
            Expression::TaggedTemplateExpression(tagged) => {
                self.propagate_through_expr(&tagged.tag, reads);
                for expr in &tagged.quasi.expressions {
                    self.propagate_through_expr(expr, reads);
                }
            }

            // Sequence expression - propagate through all expressions
            Expression::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    self.propagate_through_expr(expr, reads);
                }
            }

            // Parenthesized expression - propagate through inner expression
            Expression::ParenthesizedExpression(paren) => {
                self.propagate_through_expr(&paren.expression, reads);
            }

            // Await/Yield - propagate through argument
            Expression::AwaitExpression(await_expr) => {
                self.propagate_through_expr(&await_expr.argument, reads);
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(arg) = &yield_expr.argument {
                    self.propagate_through_expr(arg, reads);
                }
            }

            // Class expression - skip (classes have their own scope)
            Expression::ClassExpression(_) => {}

            // Function expressions - skip (functions have their own scope)
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
                // TODO: Handle closure captures
                // For now, skip as these create new scopes
            }

            // Import/Meta expressions
            Expression::ImportExpression(import) => {
                self.propagate_through_expr(&import.source, reads);
            }
            Expression::MetaProperty(_) => {}

            // Chain expression - propagate through expression
            Expression::ChainExpression(chain) => {
                match &chain.expression {
                    ChainElement::CallExpression(call) => {
                        // Reconstruct as Expression::CallExpression
                        // For now, manually traverse
                        self.propagate_through_expr(&call.callee, reads);
                        for arg in &call.arguments {
                            match arg {
                                Argument::SpreadElement(spread) => {
                                    self.propagate_through_expr(&spread.argument, reads);
                                }
                                arg => {
                                    if let Some(expr) = arg.as_expression() {
                                        self.propagate_through_expr(expr, reads);
                                    }
                                }
                            }
                        }
                    }
                    ChainElement::ComputedMemberExpression(member) => {
                        self.propagate_through_expr(&member.object, reads);
                        self.propagate_through_expr(&member.expression, reads);
                    }
                    ChainElement::StaticMemberExpression(member) => {
                        self.propagate_through_expr(&member.object, reads);
                    }
                    ChainElement::PrivateFieldExpression(member) => {
                        self.propagate_through_expr(&member.object, reads);
                    }
                    ChainElement::TSNonNullExpression(non_null) => {
                        self.propagate_through_expr(&non_null.expression, reads);
                    }
                }
            }

            // JSX - propagate through attributes and children
            Expression::JSXElement(jsx) => {
                // JSX element name - <Component /> where Component is a variable
                match &jsx.opening_element.name {
                    JSXElementName::IdentifierReference(ident_ref) => {
                        // This is a component reference like <MyComponent />
                        if let Some(reference_id) = ident_ref.reference_id.get() {
                            let reference = self.scoping.get_reference(reference_id);
                            if reference.is_read()
                                && let Some(symbol_id) = reference.symbol_id()
                            {
                                reads.push(symbol_id);
                            }
                        }
                    }
                    JSXElementName::MemberExpression(member) => {
                        // Handle <Foo.Bar /> - recursively check the object
                        self.propagate_through_jsx_member_expr(member, reads);
                    }
                    _ => {
                        // JSXElementName::Identifier (lowercase like <div />),
                        // JSXElementName::NamespacedName (<foo:bar />),
                        // JSXElementName::ThisExpression (<this />)
                        // These don't reference variables
                    }
                }

                // JSX attributes
                for attr in &jsx.opening_element.attributes {
                    match attr {
                        JSXAttributeItem::Attribute(a) => {
                            if let Some(JSXAttributeValue::ExpressionContainer(container)) =
                                &a.value
                                && let Some(expr) = container.expression.as_expression()
                            {
                                self.propagate_through_expr(expr, reads);
                            }
                        }
                        JSXAttributeItem::SpreadAttribute(spread) => {
                            self.propagate_through_expr(&spread.argument, reads);
                        }
                    }
                }
                // JSX children - recursively handle nested JSX
                for child in &jsx.children {
                    if let JSXChild::ExpressionContainer(container) = child
                        && let Some(expr) = container.expression.as_expression()
                    {
                        self.propagate_through_expr(expr, reads);
                    }
                    // Note: JSXChild::Element contains more JSX elements which will be
                    // processed when we visit their nodes in the CFG
                }
            }
            Expression::JSXFragment(frag) => {
                for child in &frag.children {
                    if let JSXChild::ExpressionContainer(container) = child
                        && let Some(expr) = container.expression.as_expression()
                    {
                        self.propagate_through_expr(expr, reads);
                    }
                }
            }

            // TypeScript specific - these don't contain runtime reads
            Expression::TSAsExpression(as_expr) => {
                self.propagate_through_expr(&as_expr.expression, reads);
            }
            Expression::TSSatisfiesExpression(sat) => {
                self.propagate_through_expr(&sat.expression, reads);
            }
            Expression::TSTypeAssertion(assert) => {
                self.propagate_through_expr(&assert.expression, reads);
            }
            Expression::TSNonNullExpression(non_null) => {
                self.propagate_through_expr(&non_null.expression, reads);
            }
            Expression::TSInstantiationExpression(inst) => {
                self.propagate_through_expr(&inst.expression, reads);
            }

            // Additional expression types
            Expression::PrivateInExpression(private_in) => {
                self.propagate_through_expr(&private_in.right, reads);
            }
            Expression::V8IntrinsicExpression(_) => {
                // V8 intrinsic - typically no variable reads
            }

            // Literals and other leaf expressions that don't read variables
            Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Super(_)
            | Expression::ThisExpression(_) => {
                // No variable reads in literals
            }
        }
    }

    /// Process an instruction node and all identifier references within it
    fn process_instruction_node(&mut self, block_id: BasicBlockId, node_id: NodeId) {
        // Get the AST node
        let node = self.nodes.get_node(node_id);

        // TODO: This needs to recursively traverse the AST to find all references
        // Currently it only processes the top-level node
        self.visit_node_for_references(block_id, node_id, node.kind());
    }

    /// Recursively visit a node and its children to find all identifier references
    ///
    /// This traverses the AST in a specific order important for dataflow analysis:
    /// - For assignments, process right side (reads) before left side (writes)
    /// - For declarations, process initializer before the binding pattern
    /// - For update expressions (++/--), mark as both read and write
    fn visit_node_for_references(
        &mut self,
        block_id: BasicBlockId,
        _node_id: NodeId,
        kind: AstKind<'a>,
    ) {
        // Handle identifier references directly
        if let AstKind::IdentifierReference(ident_ref) = kind {
            self.process_identifier_reference(block_id, ident_ref);
            return;
        }

        // For complex nodes, recurse to children in the proper order
        match kind {
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    self.visit_expression(block_id, init);
                }
            }
            AstKind::AssignmentExpression(assign) => {
                self.visit_expression(block_id, &assign.right);
                self.visit_assignment_target(block_id, &assign.left);
            }
            AstKind::UpdateExpression(update) => {
                self.visit_simple_assignment_target(block_id, &update.argument);
            }
            AstKind::BinaryExpression(binary) => {
                self.visit_expression(block_id, &binary.left);
                self.visit_expression(block_id, &binary.right);
            }
            AstKind::LogicalExpression(logical) => {
                self.visit_expression(block_id, &logical.left);
                self.visit_expression(block_id, &logical.right);
            }
            AstKind::UnaryExpression(unary) => {
                self.visit_expression(block_id, &unary.argument);
            }
            AstKind::CallExpression(call) => {
                self.visit_expression(block_id, &call.callee);
                for arg in &call.arguments {
                    self.visit_argument(block_id, arg);
                }
            }
            AstKind::ConditionalExpression(cond) => {
                self.visit_expression(block_id, &cond.test);
                self.visit_expression(block_id, &cond.consequent);
                self.visit_expression(block_id, &cond.alternate);
            }
            AstKind::ArrayExpression(array) => {
                for element in &array.elements {
                    self.visit_array_expression_element(block_id, element);
                }
            }
            AstKind::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    self.visit_object_property_kind(block_id, prop);
                }
            }
            AstKind::ReturnStatement(ret) => {
                if let Some(arg) = &ret.argument {
                    self.visit_expression(block_id, arg);
                }
            }
            AstKind::IfStatement(if_stmt) => {
                self.visit_expression(block_id, &if_stmt.test);
            }
            AstKind::ForStatement(for_stmt) => {
                if let Some(init) = &for_stmt.init {
                    self.visit_for_statement_init(block_id, init);
                }
                if let Some(test) = &for_stmt.test {
                    self.visit_expression(block_id, test);
                }
                if let Some(update) = &for_stmt.update {
                    self.visit_expression(block_id, update);
                }
            }
            AstKind::WhileStatement(while_stmt) => {
                self.visit_expression(block_id, &while_stmt.test);
            }
            AstKind::DoWhileStatement(do_while) => {
                self.visit_expression(block_id, &do_while.test);
            }
            AstKind::SwitchStatement(switch) => {
                self.visit_expression(block_id, &switch.discriminant);
            }
            AstKind::ExpressionStatement(expr_stmt) => {
                self.visit_expression(block_id, &expr_stmt.expression);
            }
            AstKind::ThrowStatement(throw) => {
                self.visit_expression(block_id, &throw.argument);
            }
            _ => {}
        }
    }

    /// Process a single identifier reference
    fn process_identifier_reference(
        &mut self,
        block_id: BasicBlockId,
        ident: &IdentifierReference,
    ) {
        // Get the reference ID
        let Some(reference_id) = ident.reference_id.get() else {
            return; // Unresolved reference, skip
        };

        // Get the reference to find the symbol and access type
        let reference = self.scoping.get_reference(reference_id);

        let Some(symbol_id) = reference.symbol_id() else {
            return; // No symbol (unresolved), skip
        };

        let acc = if reference.is_write() && reference.is_read() {
            // Read-modify-write operations like x += 1
            ACC_READ | ACC_WRITE | ACC_USE
        } else if reference.is_write() {
            // Pure write
            ACC_WRITE
        } else {
            // Pure read
            ACC_READ | ACC_USE
        };

        self.access_var(block_id, symbol_id, acc);
    }

    /// Check if a variable is live at the entry to a symbol's definition
    ///
    /// A variable is live on entry if there's a path from this point to a read
    /// of the variable without an intervening write.
    pub fn is_live_at_symbol(&self, symbol_id: SymbolId) -> bool {
        // Check if this symbol is in our scope
        let Some(&local_idx) = self.symbol_to_local.get(&symbol_id) else {
            return false;
        };

        let local_var = SymbolId::new(local_idx);

        // Find the block containing this declaration
        // For now, check if the variable is live in ANY block
        // TODO: This could be made more precise by tracking the specific block
        for block_id in self.block_map.values() {
            if self.rwu_table.get_reader(*block_id, local_var) {
                return true;
            }
        }
        false
    }

    /// Check if a variable is meaningfully used (not just written)
    pub fn is_variable_used(&self, symbol_id: SymbolId) -> bool {
        // Check if this symbol is in our scope
        let Some(&local_idx) = self.symbol_to_local.get(&symbol_id) else {
            return false;
        };

        let local_var = SymbolId::new(local_idx);

        for block_id in self.block_map.values() {
            if self.rwu_table.get_used(*block_id, local_var) {
                return true;
            }
        }
        false
    }

    /// Find useless assignments for a specific variable using TRUE CFG-based analysis
    ///
    /// Returns a vector of NodeIds representing write operations that are never read
    /// before being overwritten or the variable going out of scope.
    ///
    /// This uses the liveness information tracked during backward dataflow:
    /// - During compute(), we captured which variables are live AFTER each write
    /// - A write is useless if the variable is NOT live immediately after the write
    /// - This is fully path-sensitive and handles all control flow correctly
    pub fn useless_assignments(&self, symbol_id: SymbolId) -> Vec<NodeId> {
        // Only analyze symbols in our scope
        if !self.symbol_to_local.contains_key(&symbol_id) {
            return Vec::new();
        }

        // If the variable is READ by a closure, all assignments are potentially used
        // by the closure, so we can't determine if they're useless without interprocedural analysis.
        // However, if the closure only WRITES to the variable (never reads), we can still
        // detect useless assignments in the local scope.
        if self.has_closure_reads(symbol_id) {
            return Vec::new();
        }

        let mut useless = Vec::new();

        // Check each recorded write to see if this symbol was NOT live after it
        for (write_node_id, dead_symbols) in &self.live_after_write {
            if dead_symbols.contains(&symbol_id) {
                useless.push(*write_node_id);
            }
        }

        useless
    }

    /// Check if a node is inside a loop
    fn is_in_loop(&self, node: &crate::AstNode) -> bool {
        use oxc_syntax::node::NodeId;

        let mut current_id = node.id();

        loop {
            let parent_id = self.nodes.parent_id(current_id);

            if parent_id == NodeId::DUMMY {
                return false;
            }

            let parent = self.nodes.get_node(parent_id);

            match parent.kind() {
                AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_) => {
                    return true;
                }
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    // Stop at function boundaries
                    return false;
                }
                _ => {}
            }
            current_id = parent_id;
        }
    }

    /// Check if a node is inside a nested function (closure)
    fn is_in_nested_function(&self, node: &crate::AstNode) -> bool {
        use oxc_syntax::node::NodeId;

        let mut current_id = node.id();
        let mut function_count = 0;

        loop {
            let parent_id = self.nodes.parent_id(current_id);

            if parent_id == NodeId::DUMMY {
                break;
            }

            let parent = self.nodes.get_node(parent_id);

            match parent.kind() {
                AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) => {
                    function_count += 1;
                    // If we've found more than one function, the inner one is nested
                    if function_count > 1 {
                        return true;
                    }
                }
                _ => {}
            }
            current_id = parent_id;
        }

        false
    }

    /// Check if there are writes in conditional branches (if/switch statements)
    /// This is used to be conservative about flagging useless assignments
    fn has_conditional_writes(&self, references: &[&oxc_semantic::Reference]) -> bool {
        let mut write_count = 0;
        let mut has_conditional = false;

        for reference in references {
            if reference.is_write() && !reference.is_read() {
                write_count += 1;
                let node = self.nodes.get_node(reference.node_id());
                if self.is_in_conditional(node) {
                    has_conditional = true;
                }
            }
        }

        // If we have multiple writes and at least one is conditional, be conservative
        write_count > 1 && has_conditional
    }

    /// Check if a node is inside a conditional statement (if, switch, conditional expression)
    fn is_in_conditional(&self, node: &crate::AstNode) -> bool {
        use oxc_syntax::node::NodeId;

        let mut current_id = node.id();

        loop {
            let parent_id = self.nodes.parent_id(current_id);

            if parent_id == NodeId::DUMMY {
                return false;
            }

            let parent = self.nodes.get_node(parent_id);

            match parent.kind() {
                AstKind::IfStatement(_)
                | AstKind::SwitchStatement(_)
                | AstKind::ConditionalExpression(_) => {
                    return true;
                }
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    // Stop at function boundaries
                    return false;
                }
                _ => {}
            }
            current_id = parent_id;
        }
    }

    /// Check if a node is in unreachable code (after return/throw/break/continue)
    fn is_unreachable_node(&self, node: &crate::AstNode) -> bool {
        use oxc_syntax::node::NodeId;

        let ref_span = node.span();

        // Find the containing block or function body
        let mut current_id = node.id();

        loop {
            let parent_id = self.nodes.parent_id(current_id);

            if parent_id == NodeId::DUMMY {
                return false;
            }

            let parent = self.nodes.get_node(parent_id);

            match parent.kind() {
                // Found a block - check if there's a return/throw before this reference
                AstKind::BlockStatement(block) => {
                    // Check all statements in this block
                    for stmt in &block.body {
                        let stmt_span = stmt.span();

                        // If this statement is before our reference
                        if stmt_span.end <= ref_span.start {
                            // Check if it's a control flow statement that makes subsequent code unreachable
                            match stmt {
                                Statement::ReturnStatement(_)
                                | Statement::ThrowStatement(_)
                                | Statement::BreakStatement(_)
                                | Statement::ContinueStatement(_) => {
                                    return true;
                                }
                                _ => {}
                            }
                        }
                    }
                    // No early exit found in this block, keep checking parent blocks
                    current_id = parent_id;
                }
                AstKind::FunctionBody(body) => {
                    // Check all statements in the function body
                    for stmt in &body.statements {
                        let stmt_span = stmt.span();

                        // If this statement is before our reference
                        if stmt_span.end <= ref_span.start {
                            match stmt {
                                Statement::ReturnStatement(_)
                                | Statement::ThrowStatement(_)
                                | Statement::BreakStatement(_)
                                | Statement::ContinueStatement(_) => {
                                    return true;
                                }
                                _ => {}
                            }
                        }
                    }
                    return false;
                }
                // Stop at function boundaries
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    return false;
                }
                _ => {
                    current_id = parent_id;
                }
            }
        }
    }

    /// Get all useless assignments for all variables in this segment
    pub fn all_useless_assignments(&self) -> Vec<(SymbolId, NodeId)> {
        let mut result = Vec::new();
        for &symbol_id in &self.local_symbols {
            for node_id in self.useless_assignments(symbol_id) {
                result.push((symbol_id, node_id));
            }
        }
        result
    }

    /// Process a variable access at a given block
    ///
    /// This is the core of the liveness analysis. When processing backwards:
    /// - A READ makes the variable live (we need its value)
    /// - A WRITE kills liveness (we're overwriting the value)
    /// - A READ+WRITE is a use (like x += 1)
    fn access_var(&mut self, block_id: BasicBlockId, var: SymbolId, acc: u32) {
        // Check if this symbol is in our scope (not from an outer scope)
        let Some(&local_idx) = self.symbol_to_local.get(&var) else {
            // Symbol not in this segment, skip it
            return;
        };

        // Convert to local SymbolId for RWU table
        let local_var = SymbolId::new(local_idx);
        let mut rwu = self.rwu_table.get(block_id, local_var);

        if (acc & ACC_WRITE) != 0 {
            // A write kills liveness - the old value is no longer needed
            rwu.reader = false;
            rwu.writer = true;
        }

        // Important: if we both read/write, must process read second
        // or else the write will override. This ensures x += 1 is treated
        // as a use of x.
        if (acc & ACC_READ) != 0 {
            rwu.reader = true;
        }

        if (acc & ACC_USE) != 0 {
            rwu.used = true;
        }

        self.rwu_table.set(block_id, local_var, rwu);
    }

    /// Mark a variable as defined (written) at a given block
    ///
    /// This completely kills liveness - the variable is being freshly assigned,
    /// so we don't need any previous value.
    ///
    /// NOTE: This function is currently unused but is part of the complete
    /// liveness analysis API. It would be used when processing variable
    /// declarations that don't have initializers.
    #[expect(dead_code)]
    fn define(&mut self, block_id: BasicBlockId, var: SymbolId) {
        let used = self.rwu_table.get_used(block_id, var);
        self.rwu_table.set(block_id, var, ReadWriteUseData { reader: false, writer: false, used });
    }

    // ===== AST Visitor Helper Methods =====
    // These methods traverse expressions and statements to find all identifier references

    fn visit_expression(&mut self, block_id: BasicBlockId, expr: &Expression<'a>) {
        match expr {
            Expression::Identifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            Expression::BinaryExpression(binary) => {
                self.visit_expression(block_id, &binary.left);
                self.visit_expression(block_id, &binary.right);
            }
            Expression::LogicalExpression(logical) => {
                self.visit_expression(block_id, &logical.left);
                self.visit_expression(block_id, &logical.right);
            }
            Expression::UnaryExpression(unary) => {
                self.visit_expression(block_id, &unary.argument);
            }
            Expression::UpdateExpression(update) => {
                self.visit_simple_assignment_target(block_id, &update.argument);
            }
            Expression::CallExpression(call) => {
                self.visit_expression(block_id, &call.callee);
                for arg in &call.arguments {
                    self.visit_argument(block_id, arg);
                }
            }
            Expression::ConditionalExpression(cond) => {
                self.visit_expression(block_id, &cond.test);
                self.visit_expression(block_id, &cond.consequent);
                self.visit_expression(block_id, &cond.alternate);
            }
            Expression::ComputedMemberExpression(computed) => {
                self.visit_expression(block_id, &computed.object);
                self.visit_expression(block_id, &computed.expression);
            }
            Expression::StaticMemberExpression(static_member) => {
                self.visit_expression(block_id, &static_member.object);
            }
            Expression::PrivateFieldExpression(private) => {
                self.visit_expression(block_id, &private.object);
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    self.visit_array_expression_element(block_id, element);
                }
            }
            Expression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    self.visit_object_property_kind(block_id, prop);
                }
            }
            Expression::AssignmentExpression(assign) => {
                self.visit_expression(block_id, &assign.right);
                self.visit_assignment_target(block_id, &assign.left);
            }
            Expression::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    self.visit_expression(block_id, expr);
                }
            }
            Expression::ParenthesizedExpression(paren) => {
                self.visit_expression(block_id, &paren.expression);
            }
            Expression::TemplateLiteral(template) => {
                for expr in &template.expressions {
                    self.visit_expression(block_id, expr);
                }
            }
            Expression::TaggedTemplateExpression(tagged) => {
                self.visit_expression(block_id, &tagged.tag);
                for expr in &tagged.quasi.expressions {
                    self.visit_expression(block_id, expr);
                }
            }
            Expression::NewExpression(new_expr) => {
                self.visit_expression(block_id, &new_expr.callee);
                for arg in &new_expr.arguments {
                    self.visit_argument(block_id, arg);
                }
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(arg) = &yield_expr.argument {
                    self.visit_expression(block_id, arg);
                }
            }
            Expression::AwaitExpression(await_expr) => {
                self.visit_expression(block_id, &await_expr.argument);
            }
            Expression::ChainExpression(chain) => {
                self.visit_chain_element(block_id, &chain.expression);
            }
            // Don't traverse into nested functions - they have their own CFG segments
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {}
            // Literals don't contain references
            _ => {}
        }
    }

    fn visit_assignment_target(&mut self, block_id: BasicBlockId, target: &AssignmentTarget<'a>) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            AssignmentTarget::ComputedMemberExpression(computed) => {
                self.visit_expression(block_id, &computed.object);
                self.visit_expression(block_id, &computed.expression);
            }
            AssignmentTarget::StaticMemberExpression(static_member) => {
                self.visit_expression(block_id, &static_member.object);
            }
            AssignmentTarget::PrivateFieldExpression(private) => {
                self.visit_expression(block_id, &private.object);
            }
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                for element in (&array.elements).into_iter().flatten() {
                    self.visit_assignment_target_maybe_default(block_id, element);
                }
                if let Some(rest) = &array.rest {
                    self.visit_assignment_target(block_id, &rest.target);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    self.visit_assignment_target_property(block_id, prop);
                }
                if let Some(rest) = &obj.rest {
                    self.visit_assignment_target(block_id, &rest.target);
                }
            }
            _ => {}
        }
    }

    fn visit_simple_assignment_target(
        &mut self,
        block_id: BasicBlockId,
        target: &SimpleAssignmentTarget<'a>,
    ) {
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            SimpleAssignmentTarget::ComputedMemberExpression(computed) => {
                self.visit_expression(block_id, &computed.object);
                self.visit_expression(block_id, &computed.expression);
            }
            SimpleAssignmentTarget::StaticMemberExpression(static_member) => {
                self.visit_expression(block_id, &static_member.object);
            }
            SimpleAssignmentTarget::PrivateFieldExpression(private) => {
                self.visit_expression(block_id, &private.object);
            }
            _ => {}
        }
    }

    fn visit_argument(&mut self, block_id: BasicBlockId, arg: &Argument<'a>) {
        // Argument inherits all Expression variants plus SpreadElement
        match arg {
            Argument::SpreadElement(spread) => {
                self.visit_expression(block_id, &spread.argument);
            }
            Argument::Identifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            Argument::BinaryExpression(binary) => {
                self.visit_expression(block_id, &binary.left);
                self.visit_expression(block_id, &binary.right);
            }
            Argument::CallExpression(call) => {
                self.visit_expression(block_id, &call.callee);
                for arg in &call.arguments {
                    self.visit_argument(block_id, arg);
                }
            }
            // For other expression types, we'd need to handle them individually
            // For now, just ignore them - this is still better than nothing
            _ => {}
        }
    }

    fn visit_array_expression_element(
        &mut self,
        block_id: BasicBlockId,
        element: &ArrayExpressionElement<'a>,
    ) {
        match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                self.visit_expression(block_id, &spread.argument);
            }
            ArrayExpressionElement::Elision(_) => {}
            ArrayExpressionElement::Identifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            ArrayExpressionElement::BinaryExpression(binary) => {
                self.visit_expression(block_id, &binary.left);
                self.visit_expression(block_id, &binary.right);
            }
            ArrayExpressionElement::CallExpression(call) => {
                self.visit_expression(block_id, &call.callee);
                for arg in &call.arguments {
                    self.visit_argument(block_id, arg);
                }
            }
            // Handle other important expression types
            _ => {}
        }
    }

    fn visit_object_property_kind(
        &mut self,
        block_id: BasicBlockId,
        prop: &ObjectPropertyKind<'a>,
    ) {
        match prop {
            ObjectPropertyKind::ObjectProperty(obj_prop) => {
                if obj_prop.computed {
                    self.visit_property_key(block_id, &obj_prop.key);
                }
                self.visit_expression(block_id, &obj_prop.value);
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                self.visit_expression(block_id, &spread.argument);
            }
        }
    }

    fn visit_property_key(&mut self, block_id: BasicBlockId, key: &PropertyKey<'a>) {
        // PropertyKey inherits Expression, handle the key expression types
        match key {
            PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => {
                // These are identifiers but not identifier references
            }
            PropertyKey::Identifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            PropertyKey::BinaryExpression(binary) => {
                self.visit_expression(block_id, &binary.left);
                self.visit_expression(block_id, &binary.right);
            }
            _ => {}
        }
    }

    fn visit_assignment_target_maybe_default(
        &mut self,
        block_id: BasicBlockId,
        target: &AssignmentTargetMaybeDefault<'a>,
    ) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                self.visit_assignment_target(block_id, &with_default.binding);
                self.visit_expression(block_id, &with_default.init);
            }
            // All AssignmentTarget variants are inherited, just delegate to those handlers
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            AssignmentTargetMaybeDefault::ComputedMemberExpression(computed) => {
                self.visit_expression(block_id, &computed.object);
                self.visit_expression(block_id, &computed.expression);
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(static_member) => {
                self.visit_expression(block_id, &static_member.object);
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(private) => {
                self.visit_expression(block_id, &private.object);
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(array) => {
                for element in (&array.elements).into_iter().flatten() {
                    self.visit_assignment_target_maybe_default(block_id, element);
                }
                if let Some(rest) = &array.rest {
                    self.visit_assignment_target(block_id, &rest.target);
                }
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    self.visit_assignment_target_property(block_id, prop);
                }
                if let Some(rest) = &obj.rest {
                    self.visit_assignment_target(block_id, &rest.target);
                }
            }
            _ => {}
        }
    }

    fn visit_assignment_target_property(
        &mut self,
        block_id: BasicBlockId,
        prop: &AssignmentTargetProperty<'a>,
    ) {
        match prop {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident_prop) => {
                self.process_identifier_reference(block_id, &ident_prop.binding);
                if let Some(init) = &ident_prop.init {
                    self.visit_expression(block_id, init);
                }
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop_prop) => {
                if prop_prop.computed {
                    self.visit_property_key(block_id, &prop_prop.name);
                }
                self.visit_assignment_target_maybe_default(block_id, &prop_prop.binding);
            }
        }
    }

    fn visit_for_statement_init(&mut self, block_id: BasicBlockId, init: &ForStatementInit<'a>) {
        match init {
            ForStatementInit::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    if let Some(init) = &declarator.init {
                        self.visit_expression(block_id, init);
                    }
                }
            }
            // Handle key Expression variants that can appear in for loop init
            ForStatementInit::Identifier(ident) => {
                self.process_identifier_reference(block_id, ident);
            }
            ForStatementInit::AssignmentExpression(assign) => {
                self.visit_expression(block_id, &assign.right);
                self.visit_assignment_target(block_id, &assign.left);
            }
            ForStatementInit::CallExpression(call) => {
                self.visit_expression(block_id, &call.callee);
                for arg in &call.arguments {
                    self.visit_argument(block_id, arg);
                }
            }
            _ => {}
        }
    }

    fn visit_chain_element(&mut self, block_id: BasicBlockId, element: &ChainElement<'a>) {
        match element {
            ChainElement::CallExpression(call) => {
                self.visit_expression(block_id, &call.callee);
                for arg in &call.arguments {
                    self.visit_argument(block_id, arg);
                }
            }
            ChainElement::ComputedMemberExpression(computed) => {
                self.visit_expression(block_id, &computed.object);
                self.visit_expression(block_id, &computed.expression);
            }
            ChainElement::StaticMemberExpression(static_member) => {
                self.visit_expression(block_id, &static_member.object);
            }
            ChainElement::PrivateFieldExpression(private) => {
                self.visit_expression(block_id, &private.object);
            }
            ChainElement::TSNonNullExpression(_) => {}
        }
    }
}
