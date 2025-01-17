use std::ops::Deref;

use itertools::Itertools;
use rustc_hash::FxHashSet;

use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::{Declaration, Program, Statement};
use oxc_index::Idx;
use oxc_semantic::{ReferenceId, ScopeTree, SemanticBuilder, SymbolId, SymbolTable};
use oxc_span::Atom;

#[derive(Default, Debug, Clone, Copy)]
pub struct MangleOptions {
    pub top_level: bool,
    pub debug: bool,
}

type Slot = usize;

/// # Name Mangler / Symbol Minification
///
/// See:
///   * [esbuild](https://github.com/evanw/esbuild/blob/v0.24.0/docs/architecture.md#symbol-minification)
///
/// This algorithm is targeted for better gzip compression.
///
/// Visually, a slot is the index position for binding identifiers:
///
/// ```javascript
/// function slot0(slot2, slot3, slot4) {
///     slot2 = 1;
/// }
/// function slot1(slot2, slot3) {
///     function slot4() {
///         slot2 = 1;
///     }
/// }
/// ```
///
/// The slot number for a new scope starts after the maximum slot of the parent scope.
///
/// Occurrences of slots and their corresponding newly assigned short identifiers are:
/// - slot2: 4 - a
/// - slot3: 2 - b
/// - slot4: 2 - c
/// - slot0: 1 - d
/// - slot1: 1 - e
///
/// After swapping out the mangled names:
///
/// ```javascript
/// function d(a, b, c) {
///     a = 1;
/// }
/// function e(a, b) {
///     function c() {
///         a = 1;
///     }
/// }
/// ```
#[derive(Default)]
pub struct Mangler {
    symbol_table: SymbolTable,

    options: MangleOptions,
}

impl Mangler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_options(mut self, options: MangleOptions) -> Self {
        self.options = options;
        self
    }

    pub fn get_symbol_name(&self, symbol_id: SymbolId) -> &str {
        self.symbol_table.get_name(symbol_id)
    }

    pub fn get_reference_name(&self, reference_id: ReferenceId) -> Option<&str> {
        let symbol_id = self.symbol_table.get_reference(reference_id).symbol_id()?;
        Some(self.symbol_table.get_name(symbol_id))
    }

    #[must_use]
    pub fn build(self, program: &Program<'_>) -> Mangler {
        let semantic = SemanticBuilder::new().build(program).semantic;
        let (symbol_table, scope_tree) = semantic.into_symbol_table_and_scope_tree();
        self.build_with_symbols_and_scopes(symbol_table, &scope_tree, program)
    }

    #[must_use]
    pub fn build_with_symbols_and_scopes(
        self,
        symbol_table: SymbolTable,
        scope_tree: &ScopeTree,
        program: &Program<'_>,
    ) -> Mangler {
        if self.options.debug {
            self.build_with_symbols_and_scopes_impl(symbol_table, scope_tree, program, debug_name)
        } else {
            self.build_with_symbols_and_scopes_impl(symbol_table, scope_tree, program, base54)
        }
    }

    fn build_with_symbols_and_scopes_impl<
        const CAPACITY: usize,
        G: Fn(usize) -> InlineString<CAPACITY>,
    >(
        mut self,
        symbol_table: SymbolTable,
        scope_tree: &ScopeTree,
        program: &Program<'_>,
        generate_name: G,
    ) -> Mangler {
        let (exported_names, exported_symbols) = if self.options.top_level {
            Mangler::collect_exported_symbols(program)
        } else {
            Default::default()
        };

        let allocator = Allocator::default();

        // Mangle the symbol table by computing slots from the scope tree.
        // A slot is the occurrence index of a binding identifier inside a scope.
        let mut symbol_table = symbol_table;

        // Total number of slots for all scopes
        let mut total_number_of_slots: Slot = 0;

        // All symbols with their assigned slots. Keyed by symbol id.
        let mut slots: Vec<'_, Slot> = Vec::with_capacity_in(symbol_table.len(), &allocator);
        for _ in 0..symbol_table.len() {
            slots.push(0);
        }

        // Keep track of the maximum slot number for each scope
        let mut max_slot_for_scope = Vec::with_capacity_in(scope_tree.len(), &allocator);
        for _ in 0..scope_tree.len() {
            max_slot_for_scope.push(0);
        }

        // Walk the scope tree and compute the slot number for each scope
        let mut tmp_bindings = std::vec::Vec::with_capacity(100);
        for scope_id in scope_tree.descendants_from_root() {
            let bindings = scope_tree.get_bindings(scope_id);

            // The current slot number is continued by the maximum slot from the parent scope
            let parent_max_slot = scope_tree
                .get_parent_id(scope_id)
                .map_or(0, |parent_scope_id| max_slot_for_scope[parent_scope_id.index()]);

            let mut slot = parent_max_slot;

            if !bindings.is_empty() {
                // Sort `bindings` in declaration order.
                tmp_bindings.clear();
                tmp_bindings.extend(bindings.values().copied());
                tmp_bindings.sort_unstable();
                for symbol_id in &tmp_bindings {
                    slots[symbol_id.index()] = slot;
                    slot += 1;
                }
            }

            max_slot_for_scope[scope_id.index()] = slot;

            if slot > total_number_of_slots {
                total_number_of_slots = slot;
            }
        }

        let frequencies = self.tally_slot_frequencies(
            &symbol_table,
            &exported_symbols,
            scope_tree,
            total_number_of_slots,
            &slots,
            &allocator,
        );

        let root_unresolved_references = scope_tree.root_unresolved_references();
        let root_bindings = scope_tree.get_bindings(scope_tree.root_scope_id());

        let mut reserved_names = Vec::with_capacity_in(total_number_of_slots, &allocator);

        let mut count = 0;
        for _ in 0..total_number_of_slots {
            let name = loop {
                let name = generate_name(count);
                count += 1;
                // Do not mangle keywords and unresolved references
                let n = name.as_str();
                if !is_keyword(n)
                    && !is_special_name(n)
                    && !root_unresolved_references.contains_key(n)
                    && !(root_bindings.contains_key(n)
                        && (!self.options.top_level || exported_names.contains(n)))
                {
                    break name;
                }
            };
            reserved_names.push(name);
        }

        // Group similar symbols for smaller gzipped file
        // <https://github.com/google/closure-compiler/blob/c383a3a1d2fce33b6c778ef76b5a626e07abca41/src/com/google/javascript/jscomp/RenameVars.java#L475-L483>
        // Original Comment:
        // 1) The most frequent vars get the shorter names.
        // 2) If N number of vars are going to be assigned names of the same
        //    length, we assign the N names based on the order at which the vars
        //    first appear in the source. This makes the output somewhat less
        //    random, because symbols declared close together are assigned names
        //    that are quite similar. With this heuristic, the output is more
        //    compressible.
        //    For instance, the output may look like:
        //    var da = "..", ea = "..";
        //    function fa() { .. } function ga() { .. }

        let mut freq_iter = frequencies.iter();
        let mut symbols_renamed_in_this_batch = std::vec::Vec::with_capacity(100);
        let mut slice_of_same_len_strings = std::vec::Vec::with_capacity(100);
        // 2. "N number of vars are going to be assigned names of the same length"
        for (_, slice_of_same_len_strings_group) in
            &reserved_names.into_iter().chunk_by(InlineString::len)
        {
            // 1. "The most frequent vars get the shorter names"
            // (freq_iter is sorted by frequency from highest to lowest,
            //  so taking means take the N most frequent symbols remaining)
            slice_of_same_len_strings.clear();
            slice_of_same_len_strings.extend(slice_of_same_len_strings_group);
            symbols_renamed_in_this_batch.clear();
            symbols_renamed_in_this_batch
                .extend(freq_iter.by_ref().take(slice_of_same_len_strings.len()));

            debug_assert_eq!(symbols_renamed_in_this_batch.len(), slice_of_same_len_strings.len());

            // 2. "we assign the N names based on the order at which the vars first appear in the source."
            // sorting by slot enables us to sort by the order at which the vars first appear in the source
            // (this is possible because the slots are discovered currently in a DFS method which is the same order
            //  as variables appear in the source code)
            symbols_renamed_in_this_batch.sort_unstable_by_key(|a| a.slot);

            // here we just zip the iterator of symbols to rename with the iterator of new names for the next for loop
            let symbols_to_rename_with_new_names =
                symbols_renamed_in_this_batch.iter().zip(slice_of_same_len_strings.iter());

            // rename the variables
            for (symbol_to_rename, new_name) in symbols_to_rename_with_new_names {
                for &symbol_id in &symbol_to_rename.symbol_ids {
                    symbol_table.set_name(symbol_id, new_name);
                }
            }
        }

        self.symbol_table = symbol_table;
        self
    }

    fn tally_slot_frequencies<'a>(
        &'a self,
        symbol_table: &SymbolTable,
        exported_symbols: &FxHashSet<SymbolId>,
        scope_tree: &ScopeTree,
        total_number_of_slots: usize,
        slots: &[Slot],
        allocator: &'a Allocator,
    ) -> Vec<'a, SlotFrequency<'a>> {
        let root_scope_id = scope_tree.root_scope_id();
        let mut frequencies = Vec::with_capacity_in(total_number_of_slots, allocator);
        for _ in 0..total_number_of_slots {
            frequencies.push(SlotFrequency::new(allocator));
        }

        for (symbol_id, slot) in slots.iter().copied().enumerate() {
            let symbol_id = SymbolId::from_usize(symbol_id);
            if symbol_table.get_scope_id(symbol_id) == root_scope_id
                && (!self.options.top_level || exported_symbols.contains(&symbol_id))
            {
                continue;
            }
            if is_special_name(symbol_table.get_name(symbol_id)) {
                continue;
            }
            let index = slot;
            frequencies[index].slot = slot;
            frequencies[index].frequency +=
                symbol_table.get_resolved_reference_ids(symbol_id).len();
            frequencies[index].symbol_ids.push(symbol_id);
        }
        frequencies.sort_unstable_by_key(|x| std::cmp::Reverse(x.frequency));
        frequencies
    }

    fn collect_exported_symbols<'a>(
        program: &Program<'a>,
    ) -> (FxHashSet<Atom<'a>>, FxHashSet<SymbolId>) {
        program
            .body
            .iter()
            .filter_map(|statement| {
                let Statement::ExportNamedDeclaration(v) = statement else { return None };
                v.declaration.as_ref()
            })
            .flat_map(|decl| {
                if let Declaration::VariableDeclaration(decl) = decl {
                    itertools::Either::Left(
                        decl.declarations
                            .iter()
                            .filter_map(|decl| decl.id.get_binding_identifier()),
                    )
                } else {
                    itertools::Either::Right(decl.id().into_iter())
                }
            })
            .map(|id| (id.name.clone(), id.symbol_id()))
            .collect()
    }
}

fn is_special_name(name: &str) -> bool {
    matches!(name, "exports" | "arguments")
}

#[derive(Debug)]
struct SlotFrequency<'a> {
    pub slot: Slot,
    pub frequency: usize,
    pub symbol_ids: Vec<'a, SymbolId>,
}

impl<'a> SlotFrequency<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self { slot: 0, frequency: 0, symbol_ids: Vec::new_in(allocator) }
    }
}

#[rustfmt::skip]
fn is_keyword(s: &str) -> bool {
    matches!(s, "as" | "do" | "if" | "in" | "is" | "of" | "any" | "for" | "get"
            | "let" | "new" | "out" | "set" | "try" | "var" | "case" | "else"
            | "enum" | "from" | "meta" | "null" | "this" | "true" | "type"
            | "void" | "with")
}

#[repr(C, align(64))]
struct Aligned64([u8; 64]);

const BASE54_CHARS: Aligned64 =
    Aligned64(*b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789");

/// Get the shortest mangled name for a given n.
/// Code adapted from [terser](https://github.com/terser/terser/blob/8b966d687395ab493d2c6286cc9dd38650324c11/lib/scope.js#L1041-L1051)
//
// Maximum length of string is 11 (`ZrN6rN6rN6r` for `u64::MAX`), but set `CAPACITY` as 12,
// so the total size of `InlineString` is 16, including the `len` field.
// Then initializing the `InlineString` is a single `xmm` set, and with luck it'll sit in a register
// throughout this function.
#[expect(clippy::items_after_statements)]
fn base54(n: usize) -> InlineString<12> {
    let mut str = InlineString::new();

    let mut num = n;

    // Base 54 at first because these are the usable first characters in JavaScript identifiers
    // <https://tc39.es/ecma262/#prod-IdentifierStart>
    const FIRST_BASE: usize = 54;
    let byte = BASE54_CHARS.0[num % FIRST_BASE];
    // SAFETY: All `BASE54_CHARS` are ASCII. This is first byte we push, so can't be out of bounds.
    unsafe { str.push(byte) };
    num /= FIRST_BASE;

    // Base 64 for the rest because after the first character we can also use 0-9 too
    // <https://tc39.es/ecma262/#prod-IdentifierPart>
    const REST_BASE: usize = 64;
    while num > 0 {
        num -= 1;
        let byte = BASE54_CHARS.0[num % REST_BASE];
        // SAFETY: All `BASE54_CHARS` are ASCII.
        // Encoded string for `u64::MAX` is `ZrN6rN6rN6r` (11 bytes), so cannot push more `CAPACITY` (12).
        unsafe { str.push(byte) };
        num /= REST_BASE;
    }

    str
}

// Maximum length of string is 25 (`slot_18446744073709551615` for `u64::MAX`)
// but set `CAPACITY` as 28 so the total size of `InlineString` is 32, including the `len` field.
fn debug_name(n: usize) -> InlineString<28> {
    InlineString::from_str(&format!("slot_{n}"))
}

/// Short inline string.
///
/// `CAPACITY` determines the maximum length of the string.
#[repr(align(16))]
struct InlineString<const CAPACITY: usize> {
    len: u32,
    bytes: [u8; CAPACITY],
}

impl<const CAPACITY: usize> InlineString<CAPACITY> {
    /// Create empty [`InlineString`].
    #[inline]
    fn new() -> Self {
        const { assert!(CAPACITY <= u32::MAX as usize) };

        Self { bytes: [0; CAPACITY], len: 0 }
    }

    /// Create [`InlineString`] from `&str`.
    ///
    /// # Panics
    /// Panics if `s.len() > CAPACITY`.
    fn from_str(s: &str) -> Self {
        let mut bytes = [0; CAPACITY];
        let slice = &mut bytes[..s.len()];
        slice.copy_from_slice(s.as_bytes());
        Self { bytes, len: u32::try_from(s.len()).unwrap() }
    }

    /// Push a byte to the string.
    ///
    /// # SAFETY
    /// * Must not push more than `CAPACITY` bytes.
    /// * `byte` must be < 128 (an ASCII character).
    #[inline]
    unsafe fn push(&mut self, byte: u8) {
        debug_assert!((self.len as usize) < CAPACITY);
        debug_assert!(byte.is_ascii());

        *self.bytes.get_unchecked_mut(self.len as usize) = byte;
        self.len += 1;
    }

    /// Get length of string as `u32`.
    #[inline]
    fn len(&self) -> u32 {
        self.len
    }

    /// Get string as `&str` slice.
    #[inline]
    fn as_str(&self) -> &str {
        // SAFETY: If safety invariants of `push` have be complied with,
        // slice cannot be out of bounds, and contents of that slice is a valid UTF-8 string
        unsafe {
            let slice = self.bytes.get_unchecked(..self.len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl<const CAPACITY: usize> Deref for InlineString<CAPACITY> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod test {
    use super::base54;

    #[test]
    fn test_base54() {
        assert_eq!(&*base54(0), "a");
        assert_eq!(&*base54(25), "z");
        assert_eq!(&*base54(26), "A");
        assert_eq!(&*base54(51), "Z");
        assert_eq!(&*base54(52), "$");
        assert_eq!(&*base54(53), "_");
        assert_eq!(&*base54(54), "aa");
        assert_eq!(&*base54(55), "ab");

        if cfg!(target_pointer_width = "64") {
            assert_eq!(&*base54(usize::MAX), "ZrN6rN6rN6r");
        }

        if cfg!(target_pointer_width = "32") {
            assert_eq!(&*base54(usize::MAX), "vUdzUd");
        }
    }
}
