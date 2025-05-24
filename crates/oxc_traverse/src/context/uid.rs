use std::str;

use itoa::Buffer as ItoaBuffer;
use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, StringBuilder as ArenaStringBuilder};
use oxc_semantic::Scoping;
use oxc_span::Atom;

/// Unique identifier generator.
///
/// When initialized with [`UidGenerator::new`], creates a catalog of all symbols and unresolved references
/// in the AST which begin with `_`.
///
/// [`UidGenerator::create`] uses that catalog to generate a unique identifier which does not clash with
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
///
/// # Potential improvements
///
/// TODO(improve-on-babel):
///
/// UID generation is fairly expensive, because of the amount of string hashing required.
///
/// [`UidGenerator::new`] iterates through every binding and unresolved reference in the entire AST,
/// and builds a hashmap of symbols which could clash with UIDs.
/// Once that's built, [`UidGenerator::create`] has to do at a hashmap lookup when generating each UID.
/// Hashing strings is a fairly expensive operation.
///
/// We could improve this in one of 3 ways:
///
/// ## 1. Build the hashmap in `SemanticBuilder`
///
/// Instead of iterating through all symbols again here.
///
/// ## 2. Use a simpler algorithm
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
/// The resulting UIDs would still be fairly readable.
///
/// This is a different method from Babel, and unfortunately produces UID names
/// which differ from Babel for some of its test cases.
///
/// ## 3. Even simpler algorithm, but produces hard-to-read code
///
/// If output is being minified anyway, use a method which produces less debuggable output,
/// but is even simpler:
///
/// * During initial semantic pass, check for any existing identifiers starting with `_`.
/// * Find the highest number of leading `_`s for any existing symbol.
/// * Generate UIDs with a counter starting at 0, prefixed with number of `_`s one greater than
///   what was found in AST.
///
/// i.e. if source contains identifiers `_foo` and `__bar`, create UIDs names `___0`, `___1`,
/// `___2` etc. They'll all be unique within the program.
pub struct UidGenerator<'a> {
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

impl<'a> UidGenerator<'a> {
    /// Create [`UidGenerator`].
    pub(super) fn new(scoping: &Scoping, allocator: &'a Allocator) -> Self {
        let mut generator = Self { names: FxHashMap::default(), allocator };

        for name in scoping.symbol_names() {
            generator.add(name);
        }
        for &name in scoping.root_unresolved_references().keys() {
            generator.add(name);
        }

        generator
    }

    /// Add a record to [`UidGenerator`].
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
    /// Please see docs for [`UidGenerator`] for further info.
    pub(super) fn create(&mut self, name: &str) -> Atom<'a> {
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

            if uid_name.underscore_count == 1 {
                Atom::from_strs_array_in(["_", base, digits], self.allocator)
            } else {
                let mut uid = ArenaStringBuilder::with_capacity_in(
                    uid_name.underscore_count as usize + base.len() + digits.len(),
                    self.allocator,
                );
                uid.push_ascii_byte_repeat(b'_', uid_name.underscore_count as usize);
                uid.push_str(base);
                uid.push_str(digits);
                Atom::from(uid)
            }
        } else {
            let uid = Atom::from_strs_array_in(["_", base], self.allocator);
            // SAFETY: String starts with `_`, so trimming off that byte leaves a valid UTF-8 string
            let base = unsafe { uid.as_str().get_unchecked(1..) };
            self.names.insert(base, UidName { underscore_count: 1, postfix: 1 });
            uid
        }
    }
}

#[cfg(test)]
#[test]
fn uids() {
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
        let mut generator = UidGenerator { names: FxHashMap::default(), allocator: &allocator };
        for &used_name in used_names {
            generator.add(used_name);
        }

        for &(name, uid) in created {
            assert_eq!(generator.create(name), uid);
        }
    }
}
