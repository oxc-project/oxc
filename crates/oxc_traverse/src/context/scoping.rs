use std::{cell::Cell, collections::hash_map::Entry, str};

use compact_str::{format_compact, CompactString};
use rustc_hash::{FxHashMap, FxHashSet};

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, visit::Visit};
use oxc_semantic::{AstNodeId, Reference, ScopeTree, SymbolTable};
use oxc_span::{format_compact_str, Atom, CompactStr, Span, SPAN};
use oxc_syntax::{
    reference::{ReferenceFlags, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use super::{ast_operations::GatherNodeParts, identifier::to_identifier};
use crate::scopes_collector::ChildScopeCollector;

/// Traverse scope context.
///
/// Contains the scope tree and symbols table, and provides methods to access them.
///
/// `current_scope_id` is the ID of current scope during traversal.
/// `walk_*` functions update this field when entering/exiting a scope.
pub struct TraverseScoping {
    scopes: ScopeTree,
    symbols: SymbolTable,
    uid_names: Option<FxHashMap<CompactStr, FxHashSet<CompactStr>>>,
    current_scope_id: ScopeId,
}

// Public methods
impl TraverseScoping {
    pub fn into_symbol_table_and_scope_tree(self) -> (SymbolTable, ScopeTree) {
        (self.symbols, self.scopes)
    }

    /// Get current scope ID
    #[inline]
    pub fn current_scope_id(&self) -> ScopeId {
        self.current_scope_id
    }

    /// Get current scope flags
    #[inline]
    pub fn current_scope_flags(&self) -> ScopeFlags {
        self.scopes.get_flags(self.current_scope_id)
    }

    /// Get scopes tree
    #[inline]
    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    /// Get mutable scopes tree
    #[inline]
    pub fn scopes_mut(&mut self) -> &mut ScopeTree {
        &mut self.scopes
    }

    /// Get symbols table
    #[inline]
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    /// Get mutable symbols table
    #[inline]
    pub fn symbols_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbols
    }

    /// Get iterator over scopes, starting with current scope and working up
    pub fn ancestor_scopes(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.scopes.ancestors(self.current_scope_id)
    }

    /// Create new scope as child of provided scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn create_child_scope(&mut self, parent_id: ScopeId, flags: ScopeFlags) -> ScopeId {
        let flags = self.scopes.get_new_scope_flags(flags, parent_id);
        self.scopes.add_scope(Some(parent_id), AstNodeId::DUMMY, flags)
    }

    /// Create new scope as child of current scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn create_child_scope_of_current(&mut self, flags: ScopeFlags) -> ScopeId {
        self.create_child_scope(self.current_scope_id, flags)
    }

    /// Insert a scope into scope tree below a statement.
    ///
    /// Statement must be in current scope.
    /// New scope is created as child of current scope.
    /// All child scopes of the statement are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn insert_scope_below_statement(&mut self, stmt: &Statement, flags: ScopeFlags) -> ScopeId {
        let mut collector = ChildScopeCollector::new();
        collector.visit_statement(stmt);
        self.insert_scope_below(&collector.scope_ids, flags)
    }

    /// Insert a scope into scope tree below an expression.
    ///
    /// Expression must be in current scope.
    /// New scope is created as child of current scope.
    /// All child scopes of the expression are reassigned to be children of the new scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn insert_scope_below_expression(
        &mut self,
        expr: &Expression,
        flags: ScopeFlags,
    ) -> ScopeId {
        let mut collector = ChildScopeCollector::new();
        collector.visit_expression(expr);
        self.insert_scope_below(&collector.scope_ids, flags)
    }

    fn insert_scope_below(&mut self, child_scope_ids: &[ScopeId], flags: ScopeFlags) -> ScopeId {
        // Remove these scopes from parent's children
        if self.scopes.has_child_ids() {
            let current_child_scope_ids = self.scopes.get_child_ids_mut(self.current_scope_id);
            current_child_scope_ids.retain(|scope_id| !child_scope_ids.contains(scope_id));
        }

        // Create new scope as child of parent
        let new_scope_id = self.create_child_scope_of_current(flags);

        // Set scopes as children of new scope instead
        for &child_id in child_scope_ids {
            self.scopes.set_parent_id(child_id, Some(new_scope_id));
        }

        new_scope_id
    }

    /// Generate UID.
    ///
    /// Finds a unique variable name which does clash with any other variables used in the program.
    /// Generates a binding for it in scope provided.
    ///
    /// Based on Babel's `scope.generateUid` logic.
    /// <https://github.com/babel/babel/blob/3b1a3c0be9df65140260a316c1a21adcf948645d/packages/babel-traverse/src/scope/index.ts#L501-L523>
    ///
    /// # Differences from Babel
    ///
    /// This implementation aims to replicate Babel's behavior, but differs from Babel
    /// in the following ways:
    ///
    /// 1. Does not check that name is a valid JS identifier name.
    /// In most cases, we'll be creating a UID based on an existing variable name, in which case
    /// this check is redundant.
    /// Caller must ensure `name` is a valid JS identifier, after a `_` is prepended on start.
    /// The fact that a `_` will be prepended on start means providing an empty string or a string
    /// starting with a digit (0-9) is fine.
    ///
    /// 2. Does not convert to camel case.
    /// This seems unimportant.
    ///
    /// 3. Does not check var name against list of globals or "contextVariables"
    /// (which Babel does in `hasBinding`).
    /// No globals or "contextVariables" start with `_` anyway, so no need for this check.
    ///
    /// 4. Does not check this name is unique if used as a named statement label, only that it's unique
    /// as an identifier.
    /// If we need to generate unique labels for named statements, we should create a separate method
    /// `generate_uid_label`.
    ///
    /// 5. Does not check against list of other UIDs that have been created.
    /// `TraverseScoping::generate_uid` adds this name to symbols table, so when creating next UID,
    /// this one will be found and avoided, like any other existing binding. So it's not needed.
    ///
    /// # Potential improvements
    ///
    /// TODO(improve-on-babel):
    ///
    /// This function is fairly expensive, because it aims to replicate Babel's output.
    ///
    /// `init_uid_names` iterates through every single binding and unresolved reference in the entire AST,
    /// and builds a hashmap of `Vec`s of symbols which could clash with UIDs.
    /// Once that's built, it's cached, but still `find_uid_name` has to do a hashmap lookup,
    /// and a hashset insert.
    ///
    /// We could improve this in one of 3 ways:
    ///
    /// 1. Build the hash table in `SemanticBuilder` instead of iterating through all symbols again here.
    ///
    /// 2. Use a much simpler method:
    ///
    /// * During initial semantic pass, check for any existing identifiers starting with `_`.
    /// * Calculate what is the highest postfix number on `_...` identifiers (e.g. `_foo1`, `_bar8`).
    /// * Store that highest number in a counter which is global across the whole program.
    /// * When creating a UID, increment the counter, and make the UID `_<name><counter>`.
    ///
    /// i.e. if source contains identifiers `_foo1` and `_bar15`, create UIDs named `_qux16`,
    /// `_temp17` etc. They'll all be unique within the program.
    ///
    /// Minimal cost in semantic, and generating UIDs extremely cheap.
    ///
    /// This is a slightly different method from Babel, and unfortunately produces UID names
    /// which differ from Babel for some of its test cases.
    ///
    /// 3. If output is being minified anyway, use a method which produces less debuggable output,
    /// but is even simpler:
    ///
    /// * During initial semantic pass, check for any existing identifiers starting with `_`.
    /// * Find the highest number of leading `_`s for any existing symbol.
    /// * Generate UIDs with a counter starting at 0, prefixed with number of `_`s one greater than
    ///   what was found in AST.
    /// i.e. if source contains identifiers `_foo` and `__bar`, create UIDs names `___0`, `___1`,
    /// `___2` etc. They'll all be unique within the program.
    pub fn generate_uid(&mut self, name: &str, scope_id: ScopeId, flags: SymbolFlags) -> SymbolId {
        // Get name for UID
        let name = self.find_uid_name(name);

        // Add binding to scope
        let symbol_id =
            self.symbols.create_symbol(SPAN, name.clone(), flags, scope_id, AstNodeId::DUMMY);
        self.scopes.add_binding(scope_id, name, symbol_id);
        symbol_id
    }

    /// Generate UID in current scope.
    pub fn generate_uid_in_current_scope(&mut self, name: &str, flags: SymbolFlags) -> SymbolId {
        self.generate_uid(name, self.current_scope_id, flags)
    }

    /// Generate UID in root scope.
    pub fn generate_uid_in_root_scope(&mut self, name: &str, flags: SymbolFlags) -> SymbolId {
        self.generate_uid(name, self.scopes.root_scope_id(), flags)
    }

    /// Generate UID based on node.
    ///
    /// Recursively gathers the identifying names of a node, and joins them with `$`.
    ///
    /// Based on Babel's `scope.generateUidBasedOnNode` logic.
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L543>
    pub fn generate_uid_based_on_node<'a, T>(
        &mut self,
        node: &T,
        scope_id: ScopeId,
        flags: SymbolFlags,
    ) -> SymbolId
    where
        T: GatherNodeParts<'a>,
    {
        let mut parts = String::new();
        node.gather(&mut |part| {
            if !parts.is_empty() {
                parts.push('$');
            }
            parts.push_str(part);
        });
        let name = if parts.is_empty() { "ref" } else { parts.trim_start_matches('_') };
        self.generate_uid(&to_identifier(name.get(..20).unwrap_or(name)), scope_id, flags)
    }

    /// Generate UID in current scope based on node.
    pub fn generate_uid_in_current_scope_based_on_node<'a, T>(
        &mut self,
        node: &T,
        flags: SymbolFlags,
    ) -> SymbolId
    where
        T: GatherNodeParts<'a>,
    {
        self.generate_uid_based_on_node(node, self.current_scope_id, flags)
    }

    /// Create a reference bound to a `SymbolId`
    pub fn create_bound_reference(
        &mut self,
        symbol_id: SymbolId,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let reference = Reference::new_with_symbol_id(AstNodeId::DUMMY, symbol_id, flags);
        let reference_id = self.symbols.create_reference(reference);
        self.symbols.resolved_references[symbol_id].push(reference_id);
        reference_id
    }

    /// Create an `IdentifierReference` bound to a `SymbolId`
    pub fn create_bound_reference_id<'a>(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: SymbolId,
        flags: ReferenceFlags,
    ) -> IdentifierReference<'a> {
        let reference_id = self.create_bound_reference(symbol_id, flags);
        IdentifierReference { span, name, reference_id: Cell::new(Some(reference_id)) }
    }

    /// Create an unbound reference
    pub fn create_unbound_reference(
        &mut self,
        name: CompactStr,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let reference = Reference::new(AstNodeId::DUMMY, flags);
        let reference_id = self.symbols.create_reference(reference);
        self.scopes.add_root_unresolved_reference(name, reference_id);
        reference_id
    }

    /// Create an unbound `IdentifierReference`
    pub fn create_unbound_reference_id<'a>(
        &mut self,
        span: Span,
        name: Atom<'a>,
        flags: ReferenceFlags,
    ) -> IdentifierReference<'a> {
        let reference_id = self.create_unbound_reference(name.to_compact_str(), flags);
        IdentifierReference { span, name, reference_id: Cell::new(Some(reference_id)) }
    }

    /// Create a reference optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_reference`
    /// or `TraverseCtx::create_unbound_reference`.
    pub fn create_reference(
        &mut self,
        name: CompactStr,
        symbol_id: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        if let Some(symbol_id) = symbol_id {
            self.create_bound_reference(symbol_id, flags)
        } else {
            self.create_unbound_reference(name, flags)
        }
    }

    /// Create an `IdentifierReference` optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_reference_id`
    /// or `TraverseCtx::create_unbound_reference_id`.
    pub fn create_reference_id<'a>(
        &mut self,
        span: Span,
        name: Atom<'a>,
        symbol_id: Option<SymbolId>,
        flags: ReferenceFlags,
    ) -> IdentifierReference<'a> {
        if let Some(symbol_id) = symbol_id {
            self.create_bound_reference_id(span, name, symbol_id, flags)
        } else {
            self.create_unbound_reference_id(span, name, flags)
        }
    }

    /// Create reference in current scope, looking up binding for `name`
    pub fn create_reference_in_current_scope(
        &mut self,
        name: CompactStr,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let symbol_id = self.scopes.find_binding(self.current_scope_id, name.as_str());
        self.create_reference(name, symbol_id, flags)
    }

    /// Delete a reference.
    ///
    /// Provided `name` must match `reference_id`.
    pub fn delete_reference(&mut self, reference_id: ReferenceId, name: &str) {
        let symbol_id = self.symbols.get_reference(reference_id).symbol_id();
        if let Some(symbol_id) = symbol_id {
            self.symbols.delete_resolved_reference(symbol_id, reference_id);
        } else {
            self.scopes.delete_root_unresolved_reference(name, reference_id);
        }
    }

    /// Delete reference for an `IdentifierReference`.
    #[allow(clippy::missing_panics_doc)]
    pub fn delete_reference_for_identifier(&mut self, ident: &IdentifierReference) {
        // `unwrap` should never panic as `IdentifierReference`s should always have a `ReferenceId`
        self.delete_reference(ident.reference_id().unwrap(), &ident.name);
    }

    /// Clone `IdentifierReference` based on the original reference's `SymbolId` and name.
    ///
    /// This method makes a lookup of the `SymbolId` for the reference. If you need to create multiple
    /// `IdentifierReference`s for the same binding, it is better to look up the `SymbolId` only once,
    /// and generate `IdentifierReference`s with `TraverseScoping::create_reference_id`.
    pub fn clone_identifier_reference<'a>(
        &mut self,
        ident: &IdentifierReference<'a>,
        flags: ReferenceFlags,
    ) -> IdentifierReference<'a> {
        let reference =
            self.symbols().get_reference(ident.reference_id.get().unwrap_or_else(|| {
                unreachable!("IdentifierReference must have a reference_id");
            }));
        let symbol_id = reference.symbol_id();
        self.create_reference_id(ident.span, ident.name.clone(), symbol_id, flags)
    }

    /// Determine whether evaluating the specific input `node` is a consequenceless reference.
    ///
    /// I.E evaluating it won't result in potentially arbitrary code from being ran. The following are
    /// allowed and determined not to cause side effects:
    ///
    /// - `this` expressions
    /// - `super` expressions
    /// - Bound identifiers
    ///
    /// Based on Babel's `scope.isStatic` logic.
    /// <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L557>
    ///
    /// # Panics
    /// Can only panic if [`IdentifierReference`] does not have a reference_id, which it always should.
    #[inline]
    pub fn is_static(&self, expr: &Expression) -> bool {
        match expr {
            Expression::ThisExpression(_) | Expression::Super(_) => true,
            Expression::Identifier(ident) => self
                .symbols
                .get_reference(ident.reference_id.get().unwrap())
                .symbol_id()
                .is_some_and(|symbol_id| {
                    self.symbols.get_resolved_references(symbol_id).all(|r| !r.is_write())
                }),
            _ => false,
        }
    }
}

// Methods used internally within crate
impl TraverseScoping {
    /// Create new `TraverseScoping`
    pub(super) fn new(scopes: ScopeTree, symbols: SymbolTable) -> Self {
        Self {
            scopes,
            symbols,
            uid_names: None,
            // Dummy value. Immediately overwritten in `walk_program`.
            current_scope_id: ScopeId::new(0),
        }
    }

    /// Set current scope ID
    #[inline]
    pub(crate) fn set_current_scope_id(&mut self, scope_id: ScopeId) {
        self.current_scope_id = scope_id;
    }

    /// Find a variable name which can be used as a UID
    fn find_uid_name(&mut self, name: &str) -> CompactStr {
        // If `uid_names` is not already populated, initialize it
        if self.uid_names.is_none() {
            self.init_uid_names();
        }
        let uid_names = self.uid_names.as_mut().unwrap();

        let base = get_uid_name_base(name);
        match uid_names.entry(CompactStr::from(base)) {
            Entry::Occupied(mut entry) => {
                let uid = CompactStr::from(get_unique_name(base, entry.get()));
                entry.get_mut().insert(uid.clone());
                uid
            }
            Entry::Vacant(entry) => {
                let uid = format_compact_str!("_{base}");
                entry.insert(FxHashSet::from_iter([uid.clone()]));
                uid
            }
        }
    }

    /// Initialize `uid_names`.
    ///
    /// Iterate through all symbols and unresolved references in AST and identify any var names
    /// which could clash with UIDs (start with `_`).
    ///
    /// Compile a hashmap mapping var names (without leading `_`s or trailing digits) to any symbols
    /// used in AST matching `<one or more underscores><name><zero or more digits>`.
    ///
    /// Once this map is created, generating a UID is a relatively quick operation, rather than
    /// iterating over all symbols and unresolved references every time generate a UID.
    fn init_uid_names(&mut self) {
        let mut uid_names = FxHashMap::<CompactStr, FxHashSet<CompactStr>>::default();
        for name in self.scopes.root_unresolved_references().keys().chain(self.symbols.names.iter())
        {
            if name.as_bytes().first() != Some(&b'_') {
                continue;
            }
            // SAFETY: We just checked 1st byte is `_`, so safe to trim it off
            let base = unsafe { str::from_utf8_unchecked(&name.as_bytes()[1..]) };
            let base = get_uid_name_base(base);

            match uid_names.entry(CompactStr::from(base)) {
                Entry::Occupied(mut entry) => {
                    if !entry.get().contains(name) {
                        entry.get_mut().insert(name.clone());
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(FxHashSet::from_iter([name.clone()]));
                }
            }
        }
        self.uid_names = Some(uid_names);
    }
}

/// Create base for UID name based on provided `name`.
/// Trim `_`s from start and digits from end.
/// i.e. `__foo123` -> `foo`
fn get_uid_name_base(name: &str) -> &str {
    // Equivalent to `name.trim_start_matches('_').trim_end_matches(|c: char| c.is_ascii_digit())`
    // but more efficient as operates on bytes not chars
    let mut bytes = name.as_bytes();
    while bytes.first() == Some(&b'_') {
        bytes = &bytes[1..];
    }
    while matches!(bytes.last(), Some(b) if b.is_ascii_digit()) {
        bytes = &bytes[0..bytes.len() - 1];
    }
    // SAFETY: We started with a valid UTF8 `&str` and have only trimmed off ASCII characters,
    // so remainder must still be valid UTF8
    unsafe { str::from_utf8_unchecked(bytes) }
}

fn get_unique_name(base: &str, used: &FxHashSet<CompactStr>) -> CompactString {
    // Create `CompactString` prepending name with `_`, and with 1 byte excess capacity.
    // The extra byte is to avoid reallocation if need to add a digit on the end later,
    // which will not be too uncommon.
    // Having to add 2 digits will be uncommon, so we don't allocate 2 extra bytes for 2 digits.
    let mut name = CompactString::with_capacity(base.len() + 2);
    name.push('_');
    name.push_str(base);

    let name_is_unique = |name: &str| !used.contains(name);

    // Try the name without a numerical postfix (i.e. plain `_temp`)
    if name_is_unique(&name) {
        return name;
    }

    // It's fairly common that UIDs may need a numerical postfix, so we try to keep string
    // operations to a minimum for postfixes up to 99 - using `replace_range` on a single
    // `CompactStr`, rather than generating a new string on each attempt.
    // Postfixes greater than 99 should be very uncommon, so don't bother optimizing.

    // Try single-digit postfixes (i.e. `_temp2`, `_temp3` ... `_temp9`)
    name.push('2');
    if name_is_unique(&name) {
        return name;
    }
    for c in b'3'..=b'9' {
        name.replace_range(name.len() - 1.., str::from_utf8(&[c]).unwrap());
        if name_is_unique(&name) {
            return name;
        }
    }

    // Try double-digit postfixes (i.e. `_temp10` ... `_temp99`)
    name.replace_range(name.len() - 1.., "1");
    name.push('0');
    let mut c1 = b'1';
    loop {
        if name_is_unique(&name) {
            return name;
        }
        for c2 in b'1'..=b'9' {
            name.replace_range(name.len() - 1.., str::from_utf8(&[c2]).unwrap());
            if name_is_unique(&name) {
                return name;
            }
        }
        if c1 == b'9' {
            break;
        }
        c1 += 1;
        name.replace_range(name.len() - 2.., str::from_utf8(&[c1, b'0']).unwrap());
    }

    // Try longer postfixes (`_temp100` upwards)
    let name_base = {
        name.pop();
        name.pop();
        &*name
    };
    for n in 100..=usize::MAX {
        let name = format_compact!("{}{}", name_base, n);
        if name_is_unique(&name) {
            return name;
        }
    }

    panic!("Cannot generate UID");
}
