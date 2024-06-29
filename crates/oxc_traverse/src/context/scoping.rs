use std::{cell::Cell, str};

use compact_str::{format_compact, CompactString};
#[allow(clippy::wildcard_imports)]
use oxc_ast::{
    ast::*,
    visit::{walk, Visit},
};
use oxc_semantic::{AstNodeId, Reference, ScopeTree, SymbolTable};
use oxc_span::{Atom, CompactStr, Span, SPAN};
use oxc_syntax::{
    reference::{ReferenceFlag, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use super::FinderRet;

/// Traverse scope context.
///
/// Contains the scope tree and symbols table, and provides methods to access them.
///
/// `current_scope_id` is the ID of current scope during traversal.
/// `walk_*` functions update this field when entering/exiting a scope.
pub struct TraverseScoping {
    scopes: ScopeTree,
    symbols: SymbolTable,
    current_scope_id: ScopeId,
}

// Public methods
impl TraverseScoping {
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

    /// Walk up trail of scopes to find a scope.
    ///
    /// `finder` is called with `ScopeId`.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    pub fn find_scope<F, O>(&self, finder: F) -> Option<O>
    where
        F: Fn(ScopeId) -> FinderRet<O>,
    {
        let mut scope_id = self.current_scope_id;
        loop {
            match finder(scope_id) {
                FinderRet::Found(res) => return Some(res),
                FinderRet::Stop => return None,
                FinderRet::Continue => {}
            }

            if let Some(parent_scope_id) = self.scopes.get_parent_id(scope_id) {
                scope_id = parent_scope_id;
            } else {
                return None;
            }
        }
    }

    /// Walk up trail of scopes to find a scope by checking `ScopeFlags`.
    ///
    /// `finder` is called with `ScopeFlags`.
    ///
    /// `finder` should return:
    /// * `FinderRet::Found(value)` to stop walking and return `Some(value)`.
    /// * `FinderRet::Stop` to stop walking and return `None`.
    /// * `FinderRet::Continue` to continue walking up.
    pub fn find_scope_by_flags<F, O>(&self, finder: F) -> Option<O>
    where
        F: Fn(ScopeFlags) -> FinderRet<O>,
    {
        self.find_scope(|scope_id| {
            let flags = self.scopes.get_flags(scope_id);
            finder(flags)
        })
    }

    /// Create new scope as child of current scope.
    ///
    /// `flags` provided are amended to inherit from parent scope's flags.
    pub fn create_scope_child_of_current(&mut self, flags: ScopeFlags) -> ScopeId {
        let flags = self.scopes.get_new_scope_flags(flags, self.current_scope_id);
        self.scopes.add_scope(Some(self.current_scope_id), flags)
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
        if let Some(current_child_scope_ids) = self.scopes.get_child_ids_mut(self.current_scope_id)
        {
            current_child_scope_ids.retain(|scope_id| !child_scope_ids.contains(scope_id));
        }

        // Create new scope as child of parent
        let new_scope_id = self.create_scope_child_of_current(flags);

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
    /// TODO(improve-on-babel)
    ///
    /// This function is fairly expensive, because it aims to replicate Babel's output.
    /// `name_is_unique` method below searches through every single binding in the entire program
    /// and does a string comparison on each. It also searches through every reference in entire program
    /// (though it will avoid string comparison on most of them).
    /// If the first name tried is already in use, it will repeat that entire search with a new name,
    /// potentially multiple times.
    ///
    /// We could improve this in one of 2 ways:
    ///
    /// 1. Semantic generate a hash set of all identifier names used in program.
    ///    Check for uniqueness would then be just 1 x hashset lookup for each name that's tried.
    ///    This would maintain output parity with Babel.
    ///    But building the hash set would add some overhead to semantic.
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
    /// This is a slightly different method from Babel, but hopefully close enough that output will
    /// match Babel for most (or maybe all) test cases.
    pub fn generate_uid(&mut self, name: &str, scope_id: ScopeId, flags: SymbolFlags) -> SymbolId {
        // Get name for UID
        let name = CompactStr::new(&self.find_uid_name(name));

        // Add binding to scope
        let symbol_id = self.symbols.create_symbol(SPAN, name.clone(), flags, scope_id);
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

    /// Create a reference bound to a `SymbolId`
    pub fn create_bound_reference(
        &mut self,
        name: CompactStr,
        symbol_id: SymbolId,
        flag: ReferenceFlag,
    ) -> ReferenceId {
        let reference =
            Reference::new_with_symbol_id(SPAN, name, AstNodeId::dummy(), symbol_id, flag);
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
        flag: ReferenceFlag,
    ) -> IdentifierReference<'a> {
        let reference_id = self.create_bound_reference(name.to_compact_str(), symbol_id, flag);
        IdentifierReference {
            span,
            name,
            reference_id: Cell::new(Some(reference_id)),
            reference_flag: flag,
        }
    }

    /// Create an unbound reference
    pub fn create_unbound_reference(
        &mut self,
        name: CompactStr,
        flag: ReferenceFlag,
        // TODO: meaning
    ) -> ReferenceId {
        let reference = Reference::new(SPAN, name.clone(), AstNodeId::dummy(), flag);
        let reference_id = self.symbols.create_reference(reference);
        self.scopes.add_root_unresolved_reference(name, reference_id, SymbolFlags::all());
        reference_id
    }

    /// Create an unbound `IdentifierReference`
    pub fn create_unbound_reference_id<'a>(
        &mut self,
        span: Span,
        name: Atom<'a>,
        flag: ReferenceFlag,
    ) -> IdentifierReference<'a> {
        let reference_id = self.create_unbound_reference(name.to_compact_str(), flag);
        IdentifierReference {
            span,
            name,
            reference_id: Cell::new(Some(reference_id)),
            reference_flag: flag,
        }
    }

    /// Create a reference optionally bound to a `SymbolId`.
    ///
    /// If you know if there's a `SymbolId` or not, prefer `TraverseCtx::create_bound_reference`
    /// or `TraverseCtx::create_unbound_reference`.
    pub fn create_reference(
        &mut self,
        name: CompactStr,
        symbol_id: Option<SymbolId>,
        flag: ReferenceFlag,
    ) -> ReferenceId {
        if let Some(symbol_id) = symbol_id {
            self.create_bound_reference(name, symbol_id, flag)
        } else {
            self.create_unbound_reference(name, flag)
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
        flag: ReferenceFlag,
    ) -> IdentifierReference<'a> {
        if let Some(symbol_id) = symbol_id {
            self.create_bound_reference_id(span, name, symbol_id, flag)
        } else {
            self.create_unbound_reference_id(span, name, flag)
        }
    }

    /// Create reference in current scope, looking up binding for `name`
    pub fn create_reference_in_current_scope(
        &mut self,
        name: CompactStr,
        flag: ReferenceFlag,
    ) -> ReferenceId {
        let meaning = if flag.intersects(ReferenceFlag::Type) {
            SymbolFlags::Type
        } else {
            SymbolFlags::Value
        };
        // let symbol_id = self.scopes.find_binding(self.current_scope_id,
        // name.as_str());
        let symbol_id = self.resolve_binding(name.as_str(), Some(meaning));
        self.create_reference(name, symbol_id, flag)
    }

    /// Attempt to resolve a symbol bound to a `name` by walking up the scope
    /// tree, starting at the current scope id.
    ///
    /// See [`TraverseScoping::resolve_binding_from_scope`] for more details.
    pub fn resolve_binding(&self, name: &str, meaning: Option<SymbolFlags>) -> Option<SymbolId> {
        self.resolve_binding_from_scope(self.current_scope_id, name, meaning)
    }

    /// Attempt to resolve a symbol bound to a `name` by walking up the scope
    /// tree, starting at the desired `scope_id`.
    ///
    /// Meaning refers to the kind of symbol desired. It will almost always be
    /// [`SymbolFlags::Value`] or [`SymbolFlags::Type`]. This prevents us from
    /// binding variable declarations to type references, e.g. in
    /// ```ts
    /// const x = 1;
    /// function foo(a: x) {}
    /// ```
    ///
    /// If you do not care about what kind of symbol is found, you can pass
    /// [`None`]; this is equivalent to [`SymbolFlags::all()`].
    ///
    /// This method returns [`None`] when no symbol is found with the desired
    /// `name`, or if a symbol is found but it does not have the correct meaning.
    pub fn resolve_binding_from_scope(
        &self,
        scope_id: ScopeId,
        name: &str,
        meaning: Option<SymbolFlags>,
    ) -> Option<SymbolId> {
        let meaning = meaning.unwrap_or(SymbolFlags::all());
        for scope_id in self.scopes.ancestors(scope_id) {
            if let Some(symbol_id) = self.scopes.get_binding(scope_id, name) {
                let symbol_flags = self.symbols.get_flag(symbol_id);
                if symbol_flags.intersects(meaning) {
                    return Some(symbol_id);
                }
            }
        }
        None
    }
}

// Methods used internally within crate
impl TraverseScoping {
    /// Create new `TraverseScoping`
    pub(super) fn new(scopes: ScopeTree, symbols: SymbolTable) -> Self {
        Self {
            scopes,
            symbols,
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
    fn find_uid_name(&self, name: &str) -> CompactString {
        let mut name = create_uid_name_base(name);

        // Try the name without a numerical postfix (i.e. plain `_temp`)
        if self.name_is_unique(&name) {
            return name;
        }

        // It's fairly common that UIDs may need a numerical postfix, so we try to keep string
        // operations to a minimum for postfixes up to 99 - using `replace_range` on a single
        // `CompactStr`, rather than generating a new string on each attempt.
        // Postfixes greater than 99 should be very uncommon, so don't bother optimizing.

        // Try single-digit postfixes (i.e. `_temp1`, `_temp2` ... `_temp9`)
        name.push('2');
        if self.name_is_unique(&name) {
            return name;
        }
        for c in b'3'..=b'9' {
            name.replace_range(name.len() - 1.., str::from_utf8(&[c]).unwrap());
            if self.name_is_unique(&name) {
                return name;
            }
        }

        // Try double-digit postfixes (i.e. `_temp10` ... `_temp99`)
        name.replace_range(name.len() - 1.., "1");
        name.push('0');
        let mut c1 = b'1';
        loop {
            if self.name_is_unique(&name) {
                return name;
            }
            for c2 in b'1'..=b'9' {
                name.replace_range(name.len() - 1.., str::from_utf8(&[c2]).unwrap());
                if self.name_is_unique(&name) {
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
            if self.name_is_unique(&name) {
                return name;
            }
        }

        panic!("Cannot generate UID");
    }

    fn name_is_unique(&self, name: &str) -> bool {
        // Check if any bindings in program with this name
        if self.symbols.names.iter().any(|n| n.as_str() == name) {
            return false;
        }

        // Check for unbound references in program with this name
        !self.symbols.references.iter().any(|reference| {
            if reference.symbol_id().is_some() {
                // Skip string comparison on bound references, as they'll also be in `symbols.names`
                // which already checked above
                false
            } else {
                reference.name().as_str() == name
            }
        })
    }
}

/// Create base for UID name based on provided `name`.
/// i.e. if `name` is "foo", returns "_foo".
/// We use `CompactString` to avoid any allocations where `name` is less than 22 bytes (the common case).
fn create_uid_name_base(name: &str) -> CompactString {
    // Trim `_`s from start, and `0-9`s from end.
    // Code below is equivalent to
    // `let name = name.trim_start_matches('_').trim_end_matches(|c: char| c.is_ascii_digit());`
    // but more efficient as operates on bytes not chars.
    let mut bytes = name.as_bytes();
    while bytes.first() == Some(&b'_') {
        bytes = &bytes[1..];
    }
    while matches!(bytes.last(), Some(b) if b.is_ascii_digit()) {
        bytes = &bytes[0..bytes.len() - 1];
    }
    // SAFETY: We started with a valid UTF8 `&str` and have only trimmed off ASCII characters,
    // so remainder must still be valid UTF8
    #[allow(unsafe_code)]
    let name = unsafe { str::from_utf8_unchecked(bytes) };

    // Create `CompactString` prepending name with `_`, and with 1 byte excess capacity.
    // The extra byte is to avoid reallocation if need to add a digit on the end later,
    // which will not be too uncommon.
    // Having to add 2 digits will be uncommon, so we don't allocate 2 extra bytes for 2 digits.
    let mut str = CompactString::with_capacity(name.len() + 2);
    str.push('_');
    str.push_str(name);
    str
}

/// Visitor that locates all child scopes.
/// NB: Child scopes only, not grandchild scopes.
/// Does not do full traversal - stops each time it hits a node with a scope.
struct ChildScopeCollector {
    scope_ids: Vec<ScopeId>,
}

impl ChildScopeCollector {
    fn new() -> Self {
        Self { scope_ids: vec![] }
    }
}

impl<'a> Visit<'a> for ChildScopeCollector {
    fn visit_block_statement(&mut self, stmt: &BlockStatement<'a>) {
        self.scope_ids.push(stmt.scope_id.get().unwrap());
    }

    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        if let Some(scope_id) = stmt.scope_id.get() {
            self.scope_ids.push(scope_id);
        } else {
            walk::walk_for_statement(self, stmt);
        }
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        if let Some(scope_id) = stmt.scope_id.get() {
            self.scope_ids.push(scope_id);
        } else {
            walk::walk_for_in_statement(self, stmt);
        }
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        if let Some(scope_id) = stmt.scope_id.get() {
            self.scope_ids.push(scope_id);
        } else {
            walk::walk_for_of_statement(self, stmt);
        }
    }

    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        self.scope_ids.push(stmt.scope_id.get().unwrap());
    }

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.scope_ids.push(clause.scope_id.get().unwrap());
    }

    fn visit_finally_clause(&mut self, clause: &BlockStatement<'a>) {
        self.scope_ids.push(clause.scope_id.get().unwrap());
    }

    fn visit_function(&mut self, func: &Function<'a>, _flags: Option<ScopeFlags>) {
        self.scope_ids.push(func.scope_id.get().unwrap());
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if let Some(scope_id) = class.scope_id.get() {
            self.scope_ids.push(scope_id);
        } else {
            walk::walk_class(self, class);
        }
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.scope_ids.push(block.scope_id.get().unwrap());
    }

    fn visit_arrow_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        self.scope_ids.push(expr.scope_id.get().unwrap());
    }

    fn visit_enum(&mut self, decl: &TSEnumDeclaration<'a>) {
        self.scope_ids.push(decl.scope_id.get().unwrap());
    }

    fn visit_ts_module_declaration(&mut self, decl: &TSModuleDeclaration<'a>) {
        self.scope_ids.push(decl.scope_id.get().unwrap());
    }

    fn visit_ts_type_parameter(&mut self, ty: &TSTypeParameter<'a>) {
        self.scope_ids.push(ty.scope_id.get().unwrap());
    }
}
