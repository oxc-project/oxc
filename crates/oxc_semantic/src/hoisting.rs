//! Bookkeeping for declarations hoisted out of blocks into their var scope.

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use oxc_str::{CompactStr, Ident};
use oxc_syntax::{scope::ScopeId, symbol::SymbolId};

use crate::scoping::Scoping;

/// Declarations hoisted out of blocks: `var` declarations, and Annex B function declarations
/// in sloppy mode.
///
/// Redeclaration checks need two answers about the blocks between a hoisted declaration and its
/// var scope:
///
/// * Which of them already bind the hoisted name: `{ let x; { var x; } }`.
/// * Whether a later declaration in a block conflicts with a declaration hoisted out of or
///   through it: `{ { var x; } let x; }`.
///
/// Both are answered without walking every block, so deeply nested blocks stay linear.
///
/// Only open blocks that a declaration is hoisted out of or through are indexed, when the
/// declaration is hoisted, so code without such declarations does almost no extra work.
/// Bindings declared in an indexed block after it was indexed are added at the next hoisting.
#[derive(Default)]
pub struct Hoisting<'a> {
    /// Indexed open blocks, outermost first.
    /// Every block between an indexed block and its var scope is indexed too.
    blocks: Vec<IndexedBlock>,
    /// The last of `blocks`, kept apart because every scope exit checks it.
    innermost_block_id: Option<ScopeId>,
    /// Bindings declared in indexed blocks, in the same order as `blocks`.
    bindings: Vec<BlockBinding>,
    /// Index into `bindings` of the innermost binding of each name.
    innermost_bindings: FxHashMap<CompactStr, u32>,
    /// Symbols from this index on were created after the last indexing.
    symbols_len: usize,
    /// The last declaration hoisted out of a block into each var scope, by `(var scope, name)`.
    hoisted: FxHashMap<(ScopeId, Ident<'a>), Hoisted>,
}

#[derive(Clone, Copy)]
struct IndexedBlock {
    scope_id: ScopeId,
    var_scope_id: ScopeId,
}

#[derive(Clone, Copy)]
struct BlockBinding {
    scope_id: ScopeId,
    symbol_id: SymbolId,
    /// The binding of the same name in the next outer indexed block.
    outer: Option<u32>,
}

#[derive(Clone, Copy)]
struct Hoisted {
    /// The scope containing the declaration.
    scope_id: ScopeId,
    symbol_id: SymbolId,
    /// A `var` is hoisted through every block up to its var scope.
    /// An Annex B function declaration only counts as hoisted out of its own block.
    is_annex_b_function: bool,
}

impl<'a> Hoisting<'a> {
    /// Get the nearest var scope of the current scope `scope_id`, itself if it is one,
    /// and index every block between them.
    pub fn var_scope_id(&mut self, scoping: &Scoping, scope_id: ScopeId) -> ScopeId {
        if scoping.scope_flags(scope_id).is_var() {
            return scope_id;
        }
        let mut new_blocks = SmallVec::<[ScopeId; 8]>::new();
        let mut var_scope_id = scope_id;
        for scope_id in scoping.scope_ancestors(scope_id) {
            var_scope_id = scope_id;
            if scoping.scope_flags(scope_id).is_var() {
                break;
            }
            // The blocks outside it are indexed up to their var scope already.
            if let Some(block) = self.blocks.last()
                && block.scope_id == scope_id
            {
                var_scope_id = block.var_scope_id;
                break;
            }
            new_blocks.push(scope_id);
        }

        // Only the innermost indexed block can have been the current scope since the last call,
        // so only it can have new bindings.
        if let Some(&block) = self.blocks.last() {
            for index in self.symbols_len..scoping.symbols_len() {
                let symbol_id = SymbolId::from_usize(index);
                if scoping.symbol_scope_id(symbol_id) == block.scope_id {
                    self.push_binding(block.scope_id, scoping.symbol_name(symbol_id), symbol_id);
                }
            }
        }
        self.symbols_len = scoping.symbols_len();

        for &scope_id in new_blocks.iter().rev() {
            self.blocks.push(IndexedBlock { scope_id, var_scope_id });
            self.innermost_block_id = Some(scope_id);
            for (name, &symbol_id) in scoping.get_bindings(scope_id) {
                self.push_binding(scope_id, name.as_str(), symbol_id);
            }
        }
        var_scope_id
    }

    fn push_binding(&mut self, scope_id: ScopeId, name: &str, symbol_id: SymbolId) {
        let index = u32::try_from(self.bindings.len()).unwrap();
        let outer = if let Some(innermost) = self.innermost_bindings.get_mut(name) {
            // The block is listed for this name already.
            if self.bindings[*innermost as usize].scope_id == scope_id {
                return;
            }
            Some(std::mem::replace(innermost, index))
        } else {
            self.innermost_bindings.insert(CompactStr::new(name), index);
            None
        };
        self.bindings.push(BlockBinding { scope_id, symbol_id, outer });
    }

    /// Forget `scope_id` when leaving it.
    #[inline]
    pub fn leave_scope(&mut self, scoping: &Scoping, scope_id: ScopeId) {
        if self.innermost_block_id == Some(scope_id) {
            self.leave_block(scoping);
        }
    }

    #[cold]
    #[inline(never)]
    fn leave_block(&mut self, scoping: &Scoping) {
        let Some(block) = self.blocks.pop() else { return };
        self.innermost_block_id = self.blocks.last().map(|block| block.scope_id);
        while let Some(binding) = self.bindings.last().copied()
            && binding.scope_id == block.scope_id
        {
            self.bindings.pop();
            let name = scoping.symbol_name(binding.symbol_id);
            if let Some(outer) = binding.outer {
                *self.innermost_bindings.get_mut(name).unwrap() = outer;
            } else {
                self.innermost_bindings.remove(name);
            }
        }
    }

    /// Get the innermost indexed block binding `name`, as an index for [`Hoisting::block_binding`].
    ///
    /// Right after [`Hoisting::var_scope_id`], the indexed blocks numbered after that var scope
    /// are exactly the blocks between it and the current scope.
    pub fn innermost_block_binding(&self, name: &str) -> Option<u32> {
        self.innermost_bindings.get(name).copied()
    }

    /// Get the block of the indexed binding at `index`, and the index of the binding with the
    /// same name in the next outer indexed block.
    pub fn block_binding(&self, index: u32) -> (ScopeId, Option<u32>) {
        let binding = self.bindings[index as usize];
        (binding.scope_id, binding.outer)
    }

    /// Record a `var` declared in block `scope_id` and hoisted to `var_scope_id`.
    pub fn hoist_var(
        &mut self,
        scope_id: ScopeId,
        var_scope_id: ScopeId,
        name: Ident<'a>,
        symbol_id: SymbolId,
    ) {
        let hoisted = Hoisted { scope_id, symbol_id, is_annex_b_function: false };
        self.hoisted.insert((var_scope_id, name), hoisted);
    }

    /// Record an Annex B function declared in block `scope_id` and hoisted to `var_scope_id`.
    pub fn hoist_annex_b_function(
        &mut self,
        scope_id: ScopeId,
        var_scope_id: ScopeId,
        name: Ident<'a>,
        symbol_id: SymbolId,
    ) {
        let hoisted = Hoisted { scope_id, symbol_id, is_annex_b_function: true };
        self.hoisted.insert((var_scope_id, name), hoisted);
    }

    /// Get the last declaration named `name` hoisted out of or through the open scope `scope_id`.
    pub fn hoisted_symbol_id(&self, scope_id: ScopeId, name: Ident<'a>) -> Option<SymbolId> {
        if self.hoisted.is_empty() {
            return None;
        }
        // Hoisting out of or through a block indexes it.
        let block = match self.blocks.last() {
            Some(block) if block.scope_id == scope_id => *block,
            _ => {
                let index = self.blocks.binary_search_by_key(&scope_id, |block| block.scope_id);
                self.blocks[index.ok()?]
            }
        };
        let hoisted = self.hoisted.get(&(block.var_scope_id, name))?;
        // Only the last declaration hoisted into the var scope needs checking. Scopes are numbered
        // in preorder, so the declarations hoisted through the open block `scope_id` are the ones
        // made in it or in a scope created later. If the last one was made earlier, so were all
        // the others.
        // An Annex B function declaration can only be hoisted while the var scope has no binding
        // of its name, so it is never preceded by a hoisted `var` of the same name.
        let is_hoisted_through = if hoisted.is_annex_b_function {
            hoisted.scope_id == scope_id
        } else {
            hoisted.scope_id >= scope_id
        };
        is_hoisted_through.then_some(hoisted.symbol_id)
    }
}
