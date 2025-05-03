use std::{cmp, iter, slice, str};

use itoa::Buffer as ItoaBuffer;
use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, String as ArenaString};
use oxc_data_structures::assert_unchecked;
use oxc_semantic::Scoping;
use oxc_span::Atom;

/// Number of characters in range `a-z` or `A-Z` required to produce at least `u32::MAX` unique combinations
const POSTFIX_BYTES: usize = 6;
const _: () = {
    #[expect(clippy::cast_possible_truncation)]
    let max_combinations = 52u64.pow(POSTFIX_BYTES as u32);
    assert!(max_combinations >= u32::MAX as u64);
};

/// Unique identifier generator.
///
/// Can be either [`FastUidGenerator`] or [`DebugUidGenerator`],
/// depending on `debug` param passed to [`UidGenerator::new`].
#[expect(private_interfaces)]
pub enum UidGenerator<'a> {
    Fast(FastUidGenerator<'a>),
    Debug(DebugUidGenerator<'a>),
}

impl<'a> UidGenerator<'a> {
    /// Create [`UidGenerator`].
    ///
    /// * If `debug` is `false`, returns a fast generator which produces UIDs of form `$a`, `$b` etc.
    /// * If `debug` is `true`, returns a slower generator which produces UIDs better for debugging.
    pub(super) fn new(debug: bool, scoping: &Scoping, allocator: &'a Allocator) -> Self {
        if debug {
            Self::Debug(DebugUidGenerator::new(scoping, allocator))
        } else {
            Self::Fast(FastUidGenerator::new(scoping, allocator))
        }
    }

    /// Create a unique identifier.
    ///
    /// This method will never return the same UID twice.
    ///
    /// The form of the UID depends on value of `debug` passed to [`UidGenerator::new`].
    ///
    /// For more details, see:
    ///
    /// * [`FastUidGenerator::create`]
    /// * [`DebugUidGenerator::create`]
    pub(super) fn create(&mut self, name: &str) -> Atom<'a> {
        match self {
            Self::Fast(generator) => generator.create(),
            Self::Debug(generator) => generator.create(name),
        }
    }
}

/// Unique identifier generator which produces short var names, using a fast algorithm.
///
/// [`FastUidGenerator::new`] searches all symbols and unresolved references in AST for those that
/// begin with `$`. It finds the longest `$` prefix.
///
/// [`FastUidGenerator::create`] uses that information to generate a unique identifier which does not
/// clash with any existing name.
///
/// Generated UIDs are `$a`, `$b`, ... `$z`, `$A`, `$B`, ... `$Z`, `$aa`, `$ab`, ...
///
/// If AST already contains a symbol that begins with `$`, generated UIDs are `$$a`, `$$b`, etc.
/// If AST contains a symbol with a longer `$` prefix, generated UIDs are prefixed with 1 more `$`
/// than the longest.
/// e.g. existing symbol `$$$foo` -> UIDs `$$$$a`, `$$$$b`, etc.
/// In practice, long prefixes should be very rare.
///
/// `$` is used as the prefix instead of `_`, because it's rare that JS code uses `$` in identifiers,
/// so makes it less likely that a long prefix is required.
///
/// # Implementation details
///
/// `FastUidGenerator` owns a small string buffer.
///
/// Buffer starts as "$$$$$$`".
/// When generating a UID, the last byte is incremented.
/// i.e. "$$$$$$`" -> `$$$$$$a` -> `$$$$$$b` -> `$$$$$$c`.
///
/// All the pointers stored in the type point to different places in that buffer:
///
/// ```no_compile
/// $$$$abc
/// ^       `buffer_start_ptr`
///    ^    `active_ptr`
///       ^ `last_letter_ptr`
/// ```
///
/// "Active" part of the buffer is the section which is used as UID:
/// ```no_compile
/// Buffer: $$$$$$a
/// Active:      ^^
/// ```
///
/// 52nd UID is `$Z`, after which the UID grows in length to `$aa` ("rollover").
/// The active part of the buffer expands in place:
/// ```no_compile
/// Buffer: $$$$$aa
/// Active:     ^^^
/// ```
///
/// This in place expansion means the buffer never has to reallocate.
///
/// Using a pre-built string which is manually mutated (usually requiring just incrementing the last byte)
/// is more efficient than a `u32` counter which is converted to a string on each call to
/// [`FastUidGenerator::create`].
///
/// Using pointers to access the buffer makes the fast path for generating a UID (last byte is not `Z`,
/// so no "rollover" required) as cheap as possible - only a handful of instructions.
pub struct FastUidGenerator<'a> {
    /// Pointer to start of buffer
    buffer_start_ptr: *mut u8,
    /// Pointer to start of active string in buffer
    active_ptr: *const u8,
    /// Pointer to last letter in buffer
    last_letter_ptr: *mut u8,
    /// Allocator
    allocator: &'a Allocator,
}

impl<'a> FastUidGenerator<'a> {
    /// Create [`FastUidGenerator`].
    fn new(scoping: &Scoping, allocator: &'a Allocator) -> Self {
        // Find the symbol or unresolved references with maximum number of `$`s on start
        let mut dollar_count = 0;
        let mut update_dollar_count = |name: &str| {
            let this_dollar_count =
                name.as_bytes().iter().position(|&b| b != b'$').unwrap_or(name.len());
            dollar_count = cmp::max(dollar_count, this_dollar_count);
        };

        for name in scoping.symbol_names() {
            update_dollar_count(name);
        }
        for &name in scoping.root_unresolved_references().keys() {
            update_dollar_count(name);
        }

        // We will prefix UIDs with 1 more `$` than the longest `$` prefix in existing symbols
        dollar_count += 1;

        // Create a buffer large enough to contain all possible UID names.
        // Fill it with `$`s and a final "`".
        // If `dollar_count` is 1 (no symbols found starting with a `$`),
        // buffer contains "$$$$$$`" (7 bytes).
        // If the maximum number of UIDs are created, buffer will end up containing
        // `$ZZZZZZ` (also 7 bytes).
        // If an existing symbol was found which starts with `$$`, buffer needs to be longer.
        // Buffer will contain "$$$$$$$$`" (9 bytes). Maximum UID is `$$$ZZZZZZ` (also 9 bytes).
        let len = dollar_count + POSTFIX_BYTES;
        let mut buffer = String::with_capacity(len);
        buffer.extend(iter::repeat_n('$', len - 1));
        buffer.push('`'); // "`" is the character before `a`
        let buffer = buffer.into_boxed_str();

        // Convert `Box` to pointer.
        // We can't hold onto the `Box` because `Box` contains a `Unique` pointer and we want
        // to access `buffer`'s data via raw pointers.
        let buffer_start_ptr = Box::into_raw(buffer).cast::<u8>();

        // Get pointer to last byte in `buffer` (which is currently "`").
        // SAFETY: `buffer` is `len` bytes long, and `len > 0`, so `len - 1` cannot be out of bounds.
        let last_letter_ptr = unsafe { buffer_start_ptr.add(len - 1) };

        // Get pointer to start of active string in `buffer`.
        // If `dollar_count` is 1 (no symbols found starting with a `$`), active string is "$`".
        // If `dollar_count` is 3 (symbol found starting with `$$`), active string is "$$$`".
        // SAFETY: `last_letter_ptr` points to last byte in `buffer`.
        // `buffer`'s length is `dollar_count + POSTFIX_BYTES`, and `POSTFIX_BYTES > 0`,
        // so `last_letter_ptr - dollar_count` cannot be out of bounds of `buffer`.
        let active_ptr = unsafe { last_letter_ptr.sub(dollar_count) };

        Self { buffer_start_ptr, active_ptr, last_letter_ptr, allocator }
    }

    /// Create a unique identifier.
    ///
    /// UID will be of the form `$a`, with a sufficient number of dollars on start to avoid clash
    /// with any existing var names.
    ///
    /// This method will never return the same UID twice.
    #[inline] // `#[inline]` to inline into `TraverseCtx::generate_uid_name`
    pub(super) fn create(&mut self) -> Atom<'a> {
        // SAFETY: `last_letter_ptr` points to last byte of the buffer.
        // All bytes of the buffer are initialized. No other references to buffer exist.
        let last_letter = unsafe { self.last_letter_ptr.as_mut().unwrap_unchecked() };
        if *last_letter == b'Z' {
            return self.rollover();
        }

        // Increment letter, unless letter is `z` in which case jump to `A`.
        // Performed with arithmetic to avoid a branch. https://godbolt.org/z/Kxo9Wc98K
        *last_letter = last_letter
            .wrapping_add(1 + u8::from(*last_letter == b'z') * (b'A'.wrapping_sub(b'z') - 1));

        self.get_active()
    }

    /// Create UID when last letter is `Z`, so the previous letter needs to be incremented.
    ///
    /// Marked `#[cold]` and `#[inline(never)]` as will only happen once every 52 UIDs.
    #[cold]
    #[inline(never)]
    fn rollover(&mut self) -> Atom<'a> {
        self.rollover_update();
        self.get_active()
    }

    fn rollover_update(&mut self) {
        let mut letter_ptr = self.last_letter_ptr;

        // SAFETY: `letter_ptr` starts pointing to last byte of buffer, and is decremented.
        // Loop exits if it gets to `$`. There's always at least one `$` at start of buffer,
        // so the loop can't run beyond the start.
        // All bytes in buffer are initialized, so reading any byte is valid.
        unsafe {
            loop {
                // Set letter to `a`
                let letter = letter_ptr.as_mut().unwrap_unchecked();
                *letter = b'a';

                // Move back to previous letter
                letter_ptr = letter_ptr.sub(1);
                let letter = letter_ptr.as_mut().unwrap_unchecked();

                // If we've reached `$`, need to extend active string
                if *letter == b'$' {
                    break;
                }

                // Increment letter
                if (*letter | 32) < b'z' {
                    // `| 32` converts `A-Z` to lower case, so this matches `a-y` or `A-Y`
                    *letter += 1;
                    return;
                }
                if *letter == b'z' {
                    *letter = b'A';
                    return;
                }

                // Letter is `Z`. Need to change it to `a` and increment previous letter
                debug_assert_eq!(*letter, b'Z');
            }
        }

        // Extend active string.
        // We can only create a maximum of `POSTFIX_BYTES` letters.
        // SAFETY: Buffer is originally created with length at least `POSTFIX_BYTES + 1`.
        // `last_letter_ptr` points to the last byte so subtracting `POSTFIX_BYTES - 1` is in bounds.
        let earliest_letter_ptr = unsafe { self.last_letter_ptr.sub(POSTFIX_BYTES - 1) };
        assert!(letter_ptr.cast_const() >= earliest_letter_ptr, "Created too many UIDs");

        // Add another `a` on start (loop above has already converted all existing letters to `a`).
        // So we started with `$ZZ` and now end up with `$aaa`.
        // SAFETY: `letter_ptr` is in bounds of buffer. All bytes of buffer are initialized.
        let letter = unsafe { letter_ptr.as_mut().unwrap_unchecked() };
        *letter = b'a';

        // Extend active string forwards by 1 byte.
        // SAFETY: Buffer is created with length `POSTFIX_BYTES + dollar_count`.
        // `active_ptr` is `dollar_count` less than the first letter.
        // We just increased number of letters by 1, and checked new number of letters does not
        // exceed `POSTFIX_BYTES`, so `active_ptr - 1` cannot be before start of buffer.
        self.active_ptr = unsafe { self.active_ptr.sub(1) };
    }

    /// Get the active string (current UID) and allocate into arena. Return UID as an [`Atom`].
    //
    // `#[inline(always)]` to inline into `create`, to keep the path for no rollover as fast as possible
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn get_active(&self) -> Atom<'a> {
        // SAFETY: `active_ptr` points within buffer. `last_letter_ptr + 1` is end of buffer.
        // The distance between the two is at least 2 bytes.
        // All bytes in buffer are initialized.
        // Buffer contains only ASCII bytes, so any slice of it is a valid UTF-8 string.
        let uid = unsafe {
            let end_ptr = self.last_letter_ptr.add(1).cast_const();
            assert_unchecked!(end_ptr > self.active_ptr);
            #[expect(clippy::cast_sign_loss)]
            let len = end_ptr.offset_from(self.active_ptr) as usize;
            let slice = slice::from_raw_parts(self.active_ptr, len);
            str::from_utf8_unchecked(slice)
        };
        Atom::from(self.allocator.alloc_str(uid))
    }
}

impl Drop for FastUidGenerator<'_> {
    fn drop(&mut self) {
        // Reconstitute the original `Box<str>` created in `new`, and drop it.
        // SAFETY:
        // `buffer_start_ptr` points to start of the buffer.
        // `last_letter_ptr` points to last byte of the buffer.
        // So a slice from `buffer_start_ptr` to `last_letter_ptr + 1` is the whole buffer.
        // All bytes in buffer are initialized, and buffer contains only ASCII bytes,
        // so is a valid UTF-8 string.
        // No other references to buffer exist, so safe to give ownership of it to a `Box`.
        unsafe {
            let end_ptr = self.last_letter_ptr.add(1);
            assert_unchecked!(end_ptr > self.buffer_start_ptr);
            #[expect(clippy::cast_sign_loss)]
            let len = end_ptr.offset_from(self.buffer_start_ptr) as usize;
            let slice = slice::from_raw_parts_mut(self.buffer_start_ptr, len);
            let str = str::from_utf8_unchecked_mut(slice);
            let _box = Box::from_raw(str);
        }
    }
}

/// Unique identifier generator which produces debug-friendly variable names.
///
/// When initialized with [`DebugUidGenerator::new`], creates a catalog of all symbols and unresolved references
/// in the AST which begin with `_`.
///
/// [`DebugUidGenerator::create`] uses that catalog to generate a unique identifier which does not clash with
/// any existing name.
///
/// Such UIDs are based on the base name provided. They start with `_` and end with digits if required to
/// maintain uniqueness. e.g. given base name of `foo`, UIDs will be `_foo`, `_foo2`, `_foo3` etc.
///
/// Roughly based on Babel's `scope.generateUid` logic, but with some differences (see below).
/// <https://github.com/babel/babel/blob/3b1a3c0be9df65140260a316c1a21adcf948645d/packages/babel-traverse/src/scope/index.ts#L501-L523>
///
/// # Algorithm
///
/// UIDs are generated in series for each "base" name.
/// "Base" name is the provided name with `_`s trimmed from the start, and digits trimmed from the end.
///
/// During cataloging of existing symbols, for each base name it's recorded:
///
/// 1. Largest number of leading `_`s.
/// 2. Largest numeric postfix for that base name.
///
/// UIDs are generated for that base name with that number of leading underscores, and with ascending
/// numeric postfix.
///
/// | Existing symbols | Generated UIDs                  |
/// |------------------|---------------------------------|
/// | (none)           | `_foo`, `_foo2`, `_foo3`        |
/// | `_foo`           | `_foo2`, `_foo3`, `_foo4`       |
/// | `_foo3`          | `_foo4`, `_foo5`, `_foo6`       |
/// | `__foo`          | `__foo2`, `__foo3`, `__foo4`    |
/// | `___foo5`        | `___foo6`, `___foo7`, `___foo8` |
/// | `_foo8`, `__foo` | `__foo2`, `__foo3`, `__foo4`    |
///
/// This algorithm requires at most 1 hashmap lookup and 1 hashmap insert per UID generated.
///
/// # Differences from Babel
///
/// This implementation aims to replicate Babel's behavior, but differs from Babel
/// in the following ways:
///
/// 1. Does not check that name provided as "base" for the UID is a valid JS identifier name.
///    In most cases, we're creating a UID based on an existing variable name, in which case
///    this check is redundant.
///    Caller must ensure `name` is a valid JS identifier, after a `_` is prepended on start.
///    The fact that a `_` will be prepended on start means providing an empty string or a string
///    starting with a digit (0-9) is fine.
///
/// 2. Does not convert to camel case.
///    This seems unimportant.
///
/// 3. Does not check var name against list of globals or "contextVariables"
///    (which Babel does in `hasBinding`).
///    No globals or "contextVariables" start with `_` anyway, so no need for this check.
///
/// 4. Does not check this name is unique if used as a named statement label,
///    only that it's unique as an identifier.
///
/// 5. Uses a slightly different algorithm for generating names (see above).
///    The resulting UIDs are similar enough to Babel's algorithm to fail only 1 of Babel's tests.
struct DebugUidGenerator<'a> {
    names: FxHashMap<&'a str, UidName>,
    allocator: &'a Allocator,
}

/// Details of next UID for a base name.
//
// `#[repr(align(8))]` on 64-bit platforms so can fit in a single register.
#[cfg_attr(target_pointer_width = "64", repr(align(8)))]
#[derive(Clone, Copy)]
struct UidName {
    /// Digits appended to end of name.
    /// When generating a UID, increment this field and use that as the postfix.
    /// This field is never 0, so postfix will be at least 2.
    postfix: u32,
    /// Number of underscores to prefix name with.
    underscore_count: u32,
}

impl<'a> DebugUidGenerator<'a> {
    /// Create [`DebugUidGenerator`].
    fn new(scoping: &Scoping, allocator: &'a Allocator) -> Self {
        let mut generator = Self { names: FxHashMap::default(), allocator };

        for name in scoping.symbol_names() {
            generator.add(name);
        }
        for &name in scoping.root_unresolved_references().keys() {
            generator.add(name);
        }

        generator
    }

    /// Add a record to [`DebugUidGenerator`].
    fn add(&mut self, name: &str) {
        // If `name` does not start with `_`, exit
        if name.as_bytes().first() != Some(&b'_') {
            return;
        }

        // Trim off underscores from start of `name`
        let original_len = name.len();
        // SAFETY: We just check first byte of `name` is `_`
        let name = unsafe { name.get_unchecked(1..) };
        let mut name = name.trim_start_matches('_');
        #[expect(clippy::cast_possible_truncation)]
        let underscore_count = (original_len - name.len()) as u32;
        let mut uid_name = UidName { underscore_count, postfix: 1 };

        // Find digits on end of `name`
        let last_non_digit_index = name.as_bytes().iter().rposition(|&b| !b.is_ascii_digit());
        let parts = match last_non_digit_index {
            Some(last_non_digit_index) => {
                if last_non_digit_index == name.len() - 1 {
                    // No digits on end
                    None
                } else {
                    // Name ends with digits
                    let digit_index = last_non_digit_index + 1;
                    debug_assert!(name.as_bytes().get(digit_index).is_some_and(u8::is_ascii_digit));
                    // SAFETY: There's an ASCII digit at `digit_index`, so slicing `name` at that index
                    // is guaranteed to yield 2 valid UTF-8 strings. `digit_index` cannot be out of bounds.
                    unsafe {
                        let without_digits = name.get_unchecked(..digit_index);
                        let digits = name.get_unchecked(digit_index..);
                        Some((without_digits, digits))
                    }
                }
            }
            None => {
                if name.is_empty() {
                    // Name consists purely of `_`s e.g. `_` or `___`
                    None
                } else {
                    // Name consists of `_`s followed by digits e.g. `_123`
                    Some(("", name))
                }
            }
        };

        if let Some((without_digits, digits)) = parts {
            const U32_MAX_LEN: usize = "4294967295".len(); // 4294967295 = u32::MAX
            // SAFETY: `digits` cannot be empty
            let first_digit = unsafe { *digits.as_bytes().get_unchecked(0) };
            if first_digit == b'0' || digits.len() > U32_MAX_LEN {
                // We don't create UIDs with postfix starting with 0, or greater than `u32::MAX`,
                // so can ignore this - can't clash
                return;
            }
            if let Ok(n) = digits.parse::<u32>() {
                if n == 1 {
                    // We don't create UIDs with postfix of 1, so can ignore this - can't clash
                    return;
                }
                name = without_digits;
                uid_name.postfix = n;
            } else {
                // Digits represent a number greater than `u32::MAX`.
                // We don't create UIDs with postfix over `u32::MAX` so can ignore this - can't clash.
                return;
            }
        }

        // Unfortunately can't use `Entry` API here because `name` doesn't have required lifetime `'a`,
        // because it comes from `Semantic`'s arena, not the AST arena
        if let Some(existing_uid_name) = self.names.get_mut(name) {
            if uid_name.underscore_count > existing_uid_name.underscore_count
                || (uid_name.underscore_count == existing_uid_name.underscore_count
                    && uid_name.postfix > existing_uid_name.postfix)
            {
                *existing_uid_name = uid_name;
            }
        } else {
            let name = self.allocator.alloc_str(name);
            self.names.insert(name, uid_name);
        }
    }

    /// Create a unique identifier.
    ///
    /// The UID returned will be added to the list of used identifiers, so this method will never
    /// return the same UID twice.
    ///
    /// Caller must ensure `name` is a valid JS identifier, after a `_` is prepended on start.
    /// The fact that a `_` will be prepended on start means providing an empty string or a string
    /// starting with a digit (0-9) is fine.
    ///
    /// Please see docs for [`DebugUidGenerator`] for further info.
    fn create(&mut self, name: &str) -> Atom<'a> {
        // Get the base name, with `_`s trimmed from start, and digits trimmed from end.
        // i.e. `__foo123` -> `foo`.
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
        let base = unsafe { str::from_utf8_unchecked(bytes) };

        // Generate UID.
        // Unfortunately can't use `Entry` API here as `name` doesn't have required lifetime `'a`.
        if let Some(uid_name) = self.names.get_mut(base) {
            // AST contains identifier(s) with this base already.
            // Get next postfix.
            if uid_name.postfix < u32::MAX {
                // Increment `postfix`
                uid_name.postfix += 1;
            } else {
                // Identifier `_<base>4294967295` was already used.
                // Can't increment `postfix` as it would wrap around, so increment `underscore_count` instead.
                // It shouldn't be possible for `underscore_count` to be `u32::MAX` too, because that
                // would require an identifier comprising `u32::MAX` x underscores in source text.
                // That's theoretically possible, but source text is limited to `u32::MAX` bytes,
                // so it'd be the entirety of the source text. Therefore `postfix` would be 1.
                uid_name.underscore_count += 1;
                uid_name.postfix = 2;
            }

            // Format UID `_<base><postfix>`.
            // If `underscore_count > 1`, add further underscores to the start.
            let mut buffer = ItoaBuffer::new();
            let digits = buffer.format(uid_name.postfix);

            let uid = if uid_name.underscore_count == 1 {
                ArenaString::from_strs_array_in(["_", base, digits], self.allocator)
            } else {
                let mut uid = ArenaString::with_capacity_in(
                    uid_name.underscore_count as usize + base.len() + digits.len(),
                    self.allocator,
                );
                uid.extend(iter::repeat_n("_", uid_name.underscore_count as usize));
                uid.push_str(base);
                uid.push_str(digits);
                uid
            };

            Atom::from(uid)
        } else {
            let uid = Atom::from(ArenaString::from_strs_array_in(["_", base], self.allocator));
            // SAFETY: String starts with `_`, so trimming off that byte leaves a valid UTF-8 string
            let base = unsafe { uid.as_str().get_unchecked(1..) };
            self.names.insert(base, UidName { underscore_count: 1, postfix: 1 });
            uid
        }
    }
}

#[cfg(test)]
#[test]
fn fast_uids() {
    use oxc_span::SPAN;
    use oxc_syntax::{node::NodeId, scope::ScopeId, symbol::SymbolFlags};

    // (&[ initial, ... ], &[ expected_uid, ... ])
    #[rustfmt::skip]
    let cases: &[(&[&str], &[&str])] = &[
        (
            &[],
            &[
                "$a", "$b", "$c", "$d", "$e", "$f", "$g", "$h", "$i", "$j", "$k", "$l", "$m",
                "$n", "$o", "$p", "$q", "$r", "$s", "$t", "$u", "$v", "$w", "$x", "$y", "$z",
                "$A", "$B", "$C", "$D", "$E", "$F", "$G", "$H", "$I", "$J", "$K", "$L", "$M",
                "$N", "$O", "$P", "$Q", "$R", "$S", "$T", "$U", "$V", "$W", "$X", "$Y", "$Z",
                "$aa", "$ab", "$ac", "$ad", "$ae", "$af", "$ag", "$ah", "$ai", "$aj", "$ak", "$al", "$am",
                "$an", "$ao", "$ap", "$aq", "$ar", "$as", "$at", "$au", "$av", "$aw", "$ax", "$ay", "$az",
                "$aA", "$aB", "$aC", "$aD", "$aE", "$aF", "$aG", "$aH", "$aI", "$aJ", "$aK", "$aL", "$aM",
                "$aN", "$aO", "$aP", "$aQ", "$aR", "$aS", "$aT", "$aU", "$aV", "$aW", "$aX", "$aY", "$aZ",
                "$ba", "$bb", "$bc", "$bd", "$be", "$bf", "$bg", "$bh", "$bi", "$bj", "$bk", "$bl", "$bm",
                "$bn", "$bo", "$bp", "$bq", "$br", "$bs", "$bt", "$bu", "$bv", "$bw", "$bx", "$by", "$bz",
                "$bA", "$bB", "$bC", "$bD", "$bE", "$bF", "$bG", "$bH", "$bI", "$bJ", "$bK", "$bL", "$bM",
                "$bN", "$bO", "$bP", "$bQ", "$bR", "$bS", "$bT", "$bU", "$bV", "$bW", "$bX", "$bY", "$bZ",
                "$ca",
            ],
        ),
        (
            &["foo", "bar$", "_$qux"],
            &[
                "$a", "$b", "$c", "$d", "$e", "$f", "$g", "$h", "$i", "$j", "$k", "$l", "$m",
                "$n", "$o", "$p", "$q", "$r", "$s", "$t", "$u", "$v", "$w", "$x", "$y", "$z",
                "$A", "$B", "$C", "$D", "$E", "$F", "$G", "$H", "$I", "$J", "$K", "$L", "$M",
                "$N", "$O", "$P", "$Q", "$R", "$S", "$T", "$U", "$V", "$W", "$X", "$Y", "$Z",
                "$aa", "$ab", "$ac", "$ad", "$ae", "$af", "$ag", "$ah", "$ai", "$aj", "$ak", "$al", "$am",
                "$an", "$ao", "$ap", "$aq", "$ar", "$as", "$at", "$au", "$av", "$aw", "$ax", "$ay", "$az",
                "$aA", "$aB", "$aC", "$aD", "$aE", "$aF", "$aG", "$aH", "$aI", "$aJ", "$aK", "$aL", "$aM",
                "$aN", "$aO", "$aP", "$aQ", "$aR", "$aS", "$aT", "$aU", "$aV", "$aW", "$aX", "$aY", "$aZ",
                "$ba", "$bb", "$bc", "$bd", "$be", "$bf", "$bg", "$bh", "$bi", "$bj", "$bk", "$bl", "$bm",
                "$bn", "$bo", "$bp", "$bq", "$br", "$bs", "$bt", "$bu", "$bv", "$bw", "$bx", "$by", "$bz",
                "$bA", "$bB", "$bC", "$bD", "$bE", "$bF", "$bG", "$bH", "$bI", "$bJ", "$bK", "$bL", "$bM",
                "$bN", "$bO", "$bP", "$bQ", "$bR", "$bS", "$bT", "$bU", "$bV", "$bW", "$bX", "$bY", "$bZ",
                "$ca",
            ],
        ),
        (
            &["$"],
            &[
                "$$a", "$$b", "$$c", "$$d", "$$e", "$$f", "$$g", "$$h", "$$i", "$$j", "$$k", "$$l", "$$m",
                "$$n", "$$o", "$$p", "$$q", "$$r", "$$s", "$$t", "$$u", "$$v", "$$w", "$$x", "$$y", "$$z",
                "$$A", "$$B", "$$C", "$$D", "$$E", "$$F", "$$G", "$$H", "$$I", "$$J", "$$K", "$$L", "$$M",
                "$$N", "$$O", "$$P", "$$Q", "$$R", "$$S", "$$T", "$$U", "$$V", "$$W", "$$X", "$$Y", "$$Z",
                "$$aa", "$$ab", "$$ac", "$$ad", "$$ae", "$$af", "$$ag", "$$ah", "$$ai", "$$aj", "$$ak", "$$al", "$$am",
                "$$an", "$$ao", "$$ap", "$$aq", "$$ar", "$$as", "$$at", "$$au", "$$av", "$$aw", "$$ax", "$$ay", "$$az",
                "$$aA", "$$aB", "$$aC", "$$aD", "$$aE", "$$aF", "$$aG", "$$aH", "$$aI", "$$aJ", "$$aK", "$$aL", "$$aM",
                "$$aN", "$$aO", "$$aP", "$$aQ", "$$aR", "$$aS", "$$aT", "$$aU", "$$aV", "$$aW", "$$aX", "$$aY", "$$aZ",
                "$$ba", "$$bb", "$$bc", "$$bd", "$$be", "$$bf", "$$bg", "$$bh", "$$bi", "$$bj", "$$bk", "$$bl", "$$bm",
                "$$bn", "$$bo", "$$bp", "$$bq", "$$br", "$$bs", "$$bt", "$$bu", "$$bv", "$$bw", "$$bx", "$$by", "$$bz",
                "$$bA", "$$bB", "$$bC", "$$bD", "$$bE", "$$bF", "$$bG", "$$bH", "$$bI", "$$bJ", "$$bK", "$$bL", "$$bM",
                "$$bN", "$$bO", "$$bP", "$$bQ", "$$bR", "$$bS", "$$bT", "$$bU", "$$bV", "$$bW", "$$bX", "$$bY", "$$bZ",
                "$$ca",
            ],
        ),
        (
            &["$foo"],
            &[
                "$$a", "$$b", "$$c", "$$d", "$$e", "$$f", "$$g", "$$h", "$$i", "$$j", "$$k", "$$l", "$$m",
                "$$n", "$$o", "$$p", "$$q", "$$r", "$$s", "$$t", "$$u", "$$v", "$$w", "$$x", "$$y", "$$z",
                "$$A", "$$B", "$$C", "$$D", "$$E", "$$F", "$$G", "$$H", "$$I", "$$J", "$$K", "$$L", "$$M",
                "$$N", "$$O", "$$P", "$$Q", "$$R", "$$S", "$$T", "$$U", "$$V", "$$W", "$$X", "$$Y", "$$Z",
                "$$aa", "$$ab", "$$ac", "$$ad", "$$ae", "$$af", "$$ag", "$$ah", "$$ai", "$$aj", "$$ak", "$$al", "$$am",
                "$$an", "$$ao", "$$ap", "$$aq", "$$ar", "$$as", "$$at", "$$au", "$$av", "$$aw", "$$ax", "$$ay", "$$az",
                "$$aA", "$$aB", "$$aC", "$$aD", "$$aE", "$$aF", "$$aG", "$$aH", "$$aI", "$$aJ", "$$aK", "$$aL", "$$aM",
                "$$aN", "$$aO", "$$aP", "$$aQ", "$$aR", "$$aS", "$$aT", "$$aU", "$$aV", "$$aW", "$$aX", "$$aY", "$$aZ",
                "$$ba", "$$bb", "$$bc", "$$bd", "$$be", "$$bf", "$$bg", "$$bh", "$$bi", "$$bj", "$$bk", "$$bl", "$$bm",
                "$$bn", "$$bo", "$$bp", "$$bq", "$$br", "$$bs", "$$bt", "$$bu", "$$bv", "$$bw", "$$bx", "$$by", "$$bz",
                "$$bA", "$$bB", "$$bC", "$$bD", "$$bE", "$$bF", "$$bG", "$$bH", "$$bI", "$$bJ", "$$bK", "$$bL", "$$bM",
                "$$bN", "$$bO", "$$bP", "$$bQ", "$$bR", "$$bS", "$$bT", "$$bU", "$$bV", "$$bW", "$$bX", "$$bY", "$$bZ",
                "$$ca",
            ],
        ),
        (
            &["$$$"],
            &[
                "$$$$a", "$$$$b", "$$$$c", "$$$$d", "$$$$e", "$$$$f", "$$$$g", "$$$$h", "$$$$i", "$$$$j", "$$$$k", "$$$$l", "$$$$m",
                "$$$$n", "$$$$o", "$$$$p", "$$$$q", "$$$$r", "$$$$s", "$$$$t", "$$$$u", "$$$$v", "$$$$w", "$$$$x", "$$$$y", "$$$$z",
                "$$$$A", "$$$$B", "$$$$C", "$$$$D", "$$$$E", "$$$$F", "$$$$G", "$$$$H", "$$$$I", "$$$$J", "$$$$K", "$$$$L", "$$$$M",
                "$$$$N", "$$$$O", "$$$$P", "$$$$Q", "$$$$R", "$$$$S", "$$$$T", "$$$$U", "$$$$V", "$$$$W", "$$$$X", "$$$$Y", "$$$$Z",
                "$$$$aa", "$$$$ab", "$$$$ac", "$$$$ad", "$$$$ae", "$$$$af", "$$$$ag", "$$$$ah", "$$$$ai", "$$$$aj", "$$$$ak", "$$$$al", "$$$$am",
                "$$$$an", "$$$$ao", "$$$$ap", "$$$$aq", "$$$$ar", "$$$$as", "$$$$at", "$$$$au", "$$$$av", "$$$$aw", "$$$$ax", "$$$$ay", "$$$$az",
                "$$$$aA", "$$$$aB", "$$$$aC", "$$$$aD", "$$$$aE", "$$$$aF", "$$$$aG", "$$$$aH", "$$$$aI", "$$$$aJ", "$$$$aK", "$$$$aL", "$$$$aM",
                "$$$$aN", "$$$$aO", "$$$$aP", "$$$$aQ", "$$$$aR", "$$$$aS", "$$$$aT", "$$$$aU", "$$$$aV", "$$$$aW", "$$$$aX", "$$$$aY", "$$$$aZ",
                "$$$$ba", "$$$$bb", "$$$$bc", "$$$$bd", "$$$$be", "$$$$bf", "$$$$bg", "$$$$bh", "$$$$bi", "$$$$bj", "$$$$bk", "$$$$bl", "$$$$bm",
                "$$$$bn", "$$$$bo", "$$$$bp", "$$$$bq", "$$$$br", "$$$$bs", "$$$$bt", "$$$$bu", "$$$$bv", "$$$$bw", "$$$$bx", "$$$$by", "$$$$bz",
                "$$$$bA", "$$$$bB", "$$$$bC", "$$$$bD", "$$$$bE", "$$$$bF", "$$$$bG", "$$$$bH", "$$$$bI", "$$$$bJ", "$$$$bK", "$$$$bL", "$$$$bM",
                "$$$$bN", "$$$$bO", "$$$$bP", "$$$$bQ", "$$$$bR", "$$$$bS", "$$$$bT", "$$$$bU", "$$$$bV", "$$$$bW", "$$$$bX", "$$$$bY", "$$$$bZ",
                "$$$$ca",
            ],
        ),
        (
            &["$$$foo"],
            &[
                "$$$$a", "$$$$b", "$$$$c", "$$$$d", "$$$$e", "$$$$f", "$$$$g", "$$$$h", "$$$$i", "$$$$j", "$$$$k", "$$$$l", "$$$$m",
                "$$$$n", "$$$$o", "$$$$p", "$$$$q", "$$$$r", "$$$$s", "$$$$t", "$$$$u", "$$$$v", "$$$$w", "$$$$x", "$$$$y", "$$$$z",
                "$$$$A", "$$$$B", "$$$$C", "$$$$D", "$$$$E", "$$$$F", "$$$$G", "$$$$H", "$$$$I", "$$$$J", "$$$$K", "$$$$L", "$$$$M",
                "$$$$N", "$$$$O", "$$$$P", "$$$$Q", "$$$$R", "$$$$S", "$$$$T", "$$$$U", "$$$$V", "$$$$W", "$$$$X", "$$$$Y", "$$$$Z",
                "$$$$aa", "$$$$ab", "$$$$ac", "$$$$ad", "$$$$ae", "$$$$af", "$$$$ag", "$$$$ah", "$$$$ai", "$$$$aj", "$$$$ak", "$$$$al", "$$$$am",
                "$$$$an", "$$$$ao", "$$$$ap", "$$$$aq", "$$$$ar", "$$$$as", "$$$$at", "$$$$au", "$$$$av", "$$$$aw", "$$$$ax", "$$$$ay", "$$$$az",
                "$$$$aA", "$$$$aB", "$$$$aC", "$$$$aD", "$$$$aE", "$$$$aF", "$$$$aG", "$$$$aH", "$$$$aI", "$$$$aJ", "$$$$aK", "$$$$aL", "$$$$aM",
                "$$$$aN", "$$$$aO", "$$$$aP", "$$$$aQ", "$$$$aR", "$$$$aS", "$$$$aT", "$$$$aU", "$$$$aV", "$$$$aW", "$$$$aX", "$$$$aY", "$$$$aZ",
                "$$$$ba", "$$$$bb", "$$$$bc", "$$$$bd", "$$$$be", "$$$$bf", "$$$$bg", "$$$$bh", "$$$$bi", "$$$$bj", "$$$$bk", "$$$$bl", "$$$$bm",
                "$$$$bn", "$$$$bo", "$$$$bp", "$$$$bq", "$$$$br", "$$$$bs", "$$$$bt", "$$$$bu", "$$$$bv", "$$$$bw", "$$$$bx", "$$$$by", "$$$$bz",
                "$$$$bA", "$$$$bB", "$$$$bC", "$$$$bD", "$$$$bE", "$$$$bF", "$$$$bG", "$$$$bH", "$$$$bI", "$$$$bJ", "$$$$bK", "$$$$bL", "$$$$bM",
                "$$$$bN", "$$$$bO", "$$$$bP", "$$$$bQ", "$$$$bR", "$$$$bS", "$$$$bT", "$$$$bU", "$$$$bV", "$$$$bW", "$$$$bX", "$$$$bY", "$$$$bZ",
                "$$$$ca",
            ],
        ),
        (
            &["$$$foo", "$a"],
            &[
                "$$$$a", "$$$$b", "$$$$c", "$$$$d", "$$$$e", "$$$$f", "$$$$g", "$$$$h", "$$$$i", "$$$$j", "$$$$k", "$$$$l", "$$$$m",
                "$$$$n", "$$$$o", "$$$$p", "$$$$q", "$$$$r", "$$$$s", "$$$$t", "$$$$u", "$$$$v", "$$$$w", "$$$$x", "$$$$y", "$$$$z",
                "$$$$A", "$$$$B", "$$$$C", "$$$$D", "$$$$E", "$$$$F", "$$$$G", "$$$$H", "$$$$I", "$$$$J", "$$$$K", "$$$$L", "$$$$M",
                "$$$$N", "$$$$O", "$$$$P", "$$$$Q", "$$$$R", "$$$$S", "$$$$T", "$$$$U", "$$$$V", "$$$$W", "$$$$X", "$$$$Y", "$$$$Z",
                "$$$$aa", "$$$$ab", "$$$$ac", "$$$$ad", "$$$$ae", "$$$$af", "$$$$ag", "$$$$ah", "$$$$ai", "$$$$aj", "$$$$ak", "$$$$al", "$$$$am",
                "$$$$an", "$$$$ao", "$$$$ap", "$$$$aq", "$$$$ar", "$$$$as", "$$$$at", "$$$$au", "$$$$av", "$$$$aw", "$$$$ax", "$$$$ay", "$$$$az",
                "$$$$aA", "$$$$aB", "$$$$aC", "$$$$aD", "$$$$aE", "$$$$aF", "$$$$aG", "$$$$aH", "$$$$aI", "$$$$aJ", "$$$$aK", "$$$$aL", "$$$$aM",
                "$$$$aN", "$$$$aO", "$$$$aP", "$$$$aQ", "$$$$aR", "$$$$aS", "$$$$aT", "$$$$aU", "$$$$aV", "$$$$aW", "$$$$aX", "$$$$aY", "$$$$aZ",
                "$$$$ba", "$$$$bb", "$$$$bc", "$$$$bd", "$$$$be", "$$$$bf", "$$$$bg", "$$$$bh", "$$$$bi", "$$$$bj", "$$$$bk", "$$$$bl", "$$$$bm",
                "$$$$bn", "$$$$bo", "$$$$bp", "$$$$bq", "$$$$br", "$$$$bs", "$$$$bt", "$$$$bu", "$$$$bv", "$$$$bw", "$$$$bx", "$$$$by", "$$$$bz",
                "$$$$bA", "$$$$bB", "$$$$bC", "$$$$bD", "$$$$bE", "$$$$bF", "$$$$bG", "$$$$bH", "$$$$bI", "$$$$bJ", "$$$$bK", "$$$$bL", "$$$$bM",
                "$$$$bN", "$$$$bO", "$$$$bP", "$$$$bQ", "$$$$bR", "$$$$bS", "$$$$bT", "$$$$bU", "$$$$bV", "$$$$bW", "$$$$bX", "$$$$bY", "$$$$bZ",
                "$$$$ca",
            ],
        ),
    ];

    let allocator = Allocator::default();
    for &(used_names, created) in cases {
        let mut scoping = Scoping::default();
        for &name in used_names {
            scoping.create_symbol(SPAN, name, SymbolFlags::empty(), ScopeId::new(0), NodeId::DUMMY);
        }

        let mut generator = FastUidGenerator::new(&scoping, &allocator);
        for &expected_uid in created {
            assert_eq!(generator.create(), expected_uid);
        }
    }
}

#[cfg(test)]
#[test]
fn debug_uids() {
    // (&[ initial, ... ], &[ (name, expected_uid), ... ])
    #[expect(clippy::type_complexity)]
    let cases: &[(&[&str], &[(&str, &str)])] = &[
        (&[], &[("foo", "_foo"), ("foo", "_foo2"), ("foo", "_foo3")]),
        (
            &["foo", "foo0", "foo1", "foo2", "foo10", "_bar"],
            &[("foo", "_foo"), ("foo", "_foo2"), ("foo", "_foo3")],
        ),
        (
            &["_foo0", "_foo1", "__foo0", "____foo1", "_foo01", "_foo012345", "_foo000000"],
            &[("foo", "_foo"), ("foo", "_foo2"), ("foo", "_foo3")],
        ),
        (&[], &[("_foo", "_foo"), ("__foo", "_foo2"), ("_____foo", "_foo3")]),
        (&[], &[("_foo123", "_foo"), ("__foo456", "_foo2"), ("_____foo789", "_foo3")]),
        (&["_foo"], &[("foo", "_foo2"), ("foo", "_foo3"), ("foo", "_foo4")]),
        (&["_foo3"], &[("foo", "_foo4"), ("foo", "_foo5"), ("foo", "_foo6")]),
        (&["__foo"], &[("foo", "__foo2"), ("foo", "__foo3"), ("foo", "__foo4")]),
        (&["__foo8"], &[("foo", "__foo9"), ("foo", "__foo10"), ("foo", "__foo11")]),
        (&["_foo999", "____foo"], &[("foo", "____foo2"), ("foo", "____foo3"), ("foo", "____foo4")]),
        (
            &["_foo4294967293"],
            &[
                ("foo", "_foo4294967294"),
                ("foo", "_foo4294967295"),
                ("foo", "__foo2"),
                ("foo", "__foo3"),
            ],
        ),
        (
            &["___foo4294967293"],
            &[
                ("foo", "___foo4294967294"),
                ("foo", "___foo4294967295"),
                ("foo", "____foo2"),
                ("foo", "____foo3"),
            ],
        ),
        (&[], &[("_", "_"), ("_", "_2"), ("_", "_3")]),
        (
            &["_0", "_1", "__0", "____1", "_01", "_012345", "_000000"],
            &[("_", "_"), ("_", "_2"), ("_", "_3")],
        ),
        (&[], &[("___", "_"), ("_____", "_2"), ("_____", "_3")]),
        (&["_"], &[("_", "_2"), ("_", "_3"), ("_", "_4")]),
        (&["_4"], &[("_", "_5"), ("_", "_6"), ("_", "_7")]),
        (&["___"], &[("_", "___2"), ("_", "___3"), ("_", "___4")]),
        (&["___99"], &[("_", "___100"), ("_", "___101"), ("_", "___102")]),
        (&["_"], &[("_123", "_2"), ("__456", "_3"), ("___789", "_4")]),
    ];

    let allocator = Allocator::default();
    for &(used_names, created) in cases {
        let mut generator =
            DebugUidGenerator { names: FxHashMap::default(), allocator: &allocator };
        for &used_name in used_names {
            generator.add(used_name);
        }

        for &(name, uid) in created {
            assert_eq!(generator.create(name), uid);
        }
    }
}
