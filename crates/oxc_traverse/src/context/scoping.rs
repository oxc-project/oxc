use std::str;

use compact_str::CompactString;
use itoa::Buffer as ItoaBuffer;
use rustc_hash::FxHashSet;

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, visit::Visit};
use oxc_semantic::{NodeId, Reference, ScopeTree, SymbolTable};
use oxc_span::{Atom, CompactStr, Span};
use oxc_syntax::{
    reference::{ReferenceFlags, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};

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
    uid_names: Option<FxHashSet<CompactStr>>,
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
        self.scopes.add_scope(Some(parent_id), NodeId::DUMMY, flags)
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

    /// Generate UID var name.
    ///
    /// Finds a unique variable name which does clash with any other variables used in the program.
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
    /// and builds a hashset of symbols which could clash with UIDs.
    /// Once that's built, it's cached, but `find_uid_name` still has to do at least one hashset lookup,
    /// and a hashset insert. If the first name tried is already in use, it will do another hashset lookup,
    /// potentially multiple times until a name which isn't taken is found.
    ///
    /// We could improve this in one of 3 ways:
    ///
    /// 1. Build the hashset in `SemanticBuilder` instead of iterating through all symbols again here.
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
    #[allow(clippy::missing_panics_doc)]
    pub fn generate_uid_name(&mut self, name: &str) -> CompactStr {
        // If `uid_names` is not already populated, initialize it
        if self.uid_names.is_none() {
            self.init_uid_names();
        }
        let uid_names = self.uid_names.as_mut().unwrap();

        let base = get_uid_name_base(name);
        let uid = get_unique_name(base, uid_names);
        uid_names.insert(uid.clone());
        uid
    }

    /// Create a reference bound to a `SymbolId`
    pub fn create_bound_reference(
        &mut self,
        symbol_id: SymbolId,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let reference = Reference::new_with_symbol_id(NodeId::DUMMY, symbol_id, flags);
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
        IdentifierReference::new_with_reference_id(span, name, Some(reference_id))
    }

    /// Create an unbound reference
    pub fn create_unbound_reference(
        &mut self,
        name: CompactStr,
        flags: ReferenceFlags,
    ) -> ReferenceId {
        let reference = Reference::new(NodeId::DUMMY, flags);
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
        IdentifierReference::new_with_reference_id(span, name, Some(reference_id))
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

    /// Initialize `uid_names`.
    ///
    /// Iterate through all symbols and unresolved references in AST and identify any var names
    /// which could clash with UIDs (start with `_`). Build a hash set containing them.
    ///
    /// Once this map is created, generating a UID is a relatively quick operation, rather than
    /// iterating over all symbols and unresolved references every time generate a UID.
    fn init_uid_names(&mut self) {
        let uid_names = self
            .scopes
            .root_unresolved_references()
            .keys()
            .chain(self.symbols.names.iter())
            .filter_map(|name| {
                if name.as_bytes().first() == Some(&b'_') {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
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

fn get_unique_name(base: &str, uid_names: &FxHashSet<CompactStr>) -> CompactStr {
    CompactStr::from(get_unique_name_impl(base, uid_names))
}

// TODO: We could make this function more performant, especially when it checks a lot of names
// before it reaches one that is unused.
// This function repeatedly creates strings which have only differ from each other by digits added on end,
// and then hashes each of those strings to test them against the hash set `uid_names`.
// Hashing strings is fairly expensive. As here only the end of the string changes on each iteration,
// we could calculate an "unfinished" hash not including the last block, and then just add the final
// block to "finish" the hash on each iteration. With `FxHash` this would be straight line code and only
// a few operations.
fn get_unique_name_impl(base: &str, uid_names: &FxHashSet<CompactStr>) -> CompactString {
    // Create `CompactString` prepending name with `_`, and with 1 byte excess capacity.
    // The extra byte is to avoid reallocation if need to add a digit on the end later,
    // which will not be too uncommon.
    // Having to add 2 digits will be uncommon, so we don't allocate 2 extra bytes for 2 digits.
    let mut name = CompactString::with_capacity(base.len() + 2);
    name.push('_');
    name.push_str(base);

    // It's fairly common that UIDs may need a numerical postfix, so we try to keep string
    // operations to a minimum for postfixes up to 99 - reusing a single `CompactString`,
    // rather than generating a new string on each attempt.
    // For speed we manipulate the string as bytes.
    // Postfixes greater than 99 should be very uncommon, so don't bother optimizing.
    //
    // SAFETY: Only modifications to string are replacing last byte/last 2 bytes with ASCII digits.
    // These bytes are already ASCII chars, so cannot produce an invalid UTF-8 string.
    // Writes are always in bounds (`bytes` is redefined after string grows due to `push`).
    unsafe {
        let name_is_unique = |bytes: &[u8]| {
            let name = str::from_utf8_unchecked(bytes);
            !uid_names.contains(name)
        };

        // Try the name without a numerical postfix (i.e. plain `_temp`)
        let bytes = name.as_bytes_mut();
        if name_is_unique(bytes) {
            return name;
        }

        // Try single-digit postfixes (i.e. `_temp2`, `_temp3` ... `_temp9`)
        name.push('2');
        let bytes = name.as_bytes_mut();
        if name_is_unique(bytes) {
            return name;
        }

        let last_index = bytes.len() - 1;
        for c in b'3'..=b'9' {
            *bytes.get_unchecked_mut(last_index) = c;
            if name_is_unique(bytes) {
                return name;
            }
        }

        // Try double-digit postfixes (i.e. `_temp10` ... `_temp99`)
        *bytes.get_unchecked_mut(last_index) = b'1';
        name.push('0');
        let bytes = name.as_bytes_mut();
        let last_index = last_index + 1;

        let mut c1 = b'1';
        loop {
            if name_is_unique(bytes) {
                return name;
            }
            for c2 in b'1'..=b'9' {
                *bytes.get_unchecked_mut(last_index) = c2;
                if name_is_unique(bytes) {
                    return name;
                }
            }
            if c1 == b'9' {
                break;
            }
            c1 += 1;

            let last_two: &mut [u8; 2] =
                bytes.get_unchecked_mut(last_index - 1..=last_index).try_into().unwrap();
            *last_two = [c1, b'0'];
        }
    }

    // Try longer postfixes (`_temp100` upwards)

    // Reserve space for 1 more byte for the additional 3rd digit.
    // Do this here so that `name.push_str(digits)` will never need to grow the string until it reaches
    // `n == 1000`, which makes the branch on "is there sufficient capacity to push?" in the loop below
    // completely predictable for `n < 1000`.
    name.reserve(1);

    // At this point, `name` has had 2 digits added on end. `base_len` is length without those 2 digits.
    let base_len = name.len() - 2;

    let mut buffer = ItoaBuffer::new();
    for n in 100..=u32::MAX {
        let digits = buffer.format(n);
        // SAFETY: `base_len` is always shorter than current `name.len()`, on a UTF-8 char boundary,
        // and `name` contains at least `base_len` initialized bytes
        unsafe { name.set_len(base_len) };
        name.push_str(digits);
        if !uid_names.contains(name.as_str()) {
            return name;
        }
    }

    // Limit for size of source text is `u32::MAX` bytes, so there cannot be `u32::MAX`
    // identifier names in the AST. So loop above cannot fail to find an unused name.
    unreachable!();
}

#[cfg(test)]
#[test]
fn test_get_unique_name() {
    let cases: &[(&[&str], &str, &str)] = &[
        (&[], "foo", "_foo"),
        (&["_foo"], "foo", "_foo2"),
        (&["_foo0", "_foo1"], "foo", "_foo"),
        (&["_foo2", "_foo3", "_foo4"], "foo", "_foo"),
        (&["_foo", "_foo2"], "foo", "_foo3"),
        (&["_foo", "_foo2", "_foo4"], "foo", "_foo3"),
        (&["_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8"], "foo", "_foo9"),
        (
            &["_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9"],
            "foo",
            "_foo10",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10",
            ],
            "foo",
            "_foo11",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10", "_foo11",
            ],
            "foo",
            "_foo12",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10", "_foo11", "_foo12", "_foo13", "_foo14", "_foo15", "_foo16", "_foo17",
                "_foo18",
            ],
            "foo",
            "_foo19",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10", "_foo11", "_foo12", "_foo13", "_foo14", "_foo15", "_foo16", "_foo17",
                "_foo18", "_foo19",
            ],
            "foo",
            "_foo20",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10", "_foo11", "_foo12", "_foo13", "_foo14", "_foo15", "_foo16", "_foo17",
                "_foo18", "_foo19", "_foo20",
            ],
            "foo",
            "_foo21",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10", "_foo11", "_foo12", "_foo13", "_foo14", "_foo15", "_foo16", "_foo17",
                "_foo18", "_foo19", "_foo20", "_foo21", "_foo22", "_foo23", "_foo24", "_foo25",
                "_foo26", "_foo27", "_foo28", "_foo29", "_foo30", "_foo31", "_foo32", "_foo33",
                "_foo34", "_foo35", "_foo36", "_foo37", "_foo38", "_foo39", "_foo40", "_foo41",
                "_foo42", "_foo43", "_foo44", "_foo45", "_foo46", "_foo47", "_foo48", "_foo49",
                "_foo50", "_foo51", "_foo52", "_foo53", "_foo54", "_foo55", "_foo56", "_foo57",
                "_foo58", "_foo59", "_foo60", "_foo61", "_foo62", "_foo63", "_foo64", "_foo65",
                "_foo66", "_foo67", "_foo68", "_foo69", "_foo70", "_foo71", "_foo72", "_foo73",
                "_foo74", "_foo75", "_foo76", "_foo77", "_foo78", "_foo79", "_foo80", "_foo81",
                "_foo82", "_foo83", "_foo84", "_foo85", "_foo86", "_foo87", "_foo88", "_foo89",
                "_foo90", "_foo91", "_foo92", "_foo93", "_foo94", "_foo95", "_foo96", "_foo97",
                "_foo98",
            ],
            "foo",
            "_foo99",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10", "_foo11", "_foo12", "_foo13", "_foo14", "_foo15", "_foo16", "_foo17",
                "_foo18", "_foo19", "_foo20", "_foo21", "_foo22", "_foo23", "_foo24", "_foo25",
                "_foo26", "_foo27", "_foo28", "_foo29", "_foo30", "_foo31", "_foo32", "_foo33",
                "_foo34", "_foo35", "_foo36", "_foo37", "_foo38", "_foo39", "_foo40", "_foo41",
                "_foo42", "_foo43", "_foo44", "_foo45", "_foo46", "_foo47", "_foo48", "_foo49",
                "_foo50", "_foo51", "_foo52", "_foo53", "_foo54", "_foo55", "_foo56", "_foo57",
                "_foo58", "_foo59", "_foo60", "_foo61", "_foo62", "_foo63", "_foo64", "_foo65",
                "_foo66", "_foo67", "_foo68", "_foo69", "_foo70", "_foo71", "_foo72", "_foo73",
                "_foo74", "_foo75", "_foo76", "_foo77", "_foo78", "_foo79", "_foo80", "_foo81",
                "_foo82", "_foo83", "_foo84", "_foo85", "_foo86", "_foo87", "_foo88", "_foo89",
                "_foo90", "_foo91", "_foo92", "_foo93", "_foo94", "_foo95", "_foo96", "_foo97",
                "_foo98", "_foo99",
            ],
            "foo",
            "_foo100",
        ),
        (
            &[
                "_foo", "_foo2", "_foo3", "_foo4", "_foo5", "_foo6", "_foo7", "_foo8", "_foo9",
                "_foo10", "_foo11", "_foo12", "_foo13", "_foo14", "_foo15", "_foo16", "_foo17",
                "_foo18", "_foo19", "_foo20", "_foo21", "_foo22", "_foo23", "_foo24", "_foo25",
                "_foo26", "_foo27", "_foo28", "_foo29", "_foo30", "_foo31", "_foo32", "_foo33",
                "_foo34", "_foo35", "_foo36", "_foo37", "_foo38", "_foo39", "_foo40", "_foo41",
                "_foo42", "_foo43", "_foo44", "_foo45", "_foo46", "_foo47", "_foo48", "_foo49",
                "_foo50", "_foo51", "_foo52", "_foo53", "_foo54", "_foo55", "_foo56", "_foo57",
                "_foo58", "_foo59", "_foo60", "_foo61", "_foo62", "_foo63", "_foo64", "_foo65",
                "_foo66", "_foo67", "_foo68", "_foo69", "_foo70", "_foo71", "_foo72", "_foo73",
                "_foo74", "_foo75", "_foo76", "_foo77", "_foo78", "_foo79", "_foo80", "_foo81",
                "_foo82", "_foo83", "_foo84", "_foo85", "_foo86", "_foo87", "_foo88", "_foo89",
                "_foo90", "_foo91", "_foo92", "_foo93", "_foo94", "_foo95", "_foo96", "_foo97",
                "_foo98", "_foo99", "_foo100",
            ],
            "foo",
            "_foo101",
        ),
    ];

    for (used, name, expected) in cases {
        let used = used.iter().map(|name| CompactStr::from(*name)).collect();
        assert_eq!(get_unique_name(name, &used), expected);
    }
}
