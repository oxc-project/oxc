//! Perfect hash table mapping multi-byte operators (e.g. `==`, `>>>`) to their [`TokenKind`].
//!
//! A candidate is the next 4 bytes of source plus a length to try (2, 3 or 4).
//!
//! There are 2 separate [`OpTable`] tables for 2-byte and 3-byte operators.
//! There is only one 4-byte operator (`>>>=`), so it is handled separately.
//!
//! [`OpTable::lookup`] and code in `coalesce` pack a candidate into a `u32` key:
//! - Bottom 3 bytes: The candidate's first 3 bytes, with the 3rd zeroed if the length is 2.
//! - Top byte: 0.
//!
//! [`OpTableData::slot`] hashes the key by multiplying it by the multiplier and taking
//! the top [`OpTableData::BITS`] bits of the product as the slot index.
//!
//! [`OpTableData`] holds one `u32` per slot, built at compile time from [`OPMAP_OPS`]:
//! - Bottom 3 bytes: Operator's first 3 bytes, with the 3rd byte 0 for a 2-byte operator.
//! - Top byte: Operator's [`TokenKind`].
//!
//! Empty slots hold 0.
//!
//! So a lookup is a multiply, a load, and a comparison of the key and bottom bytes of slot value.
//! If they match, the top byte of the slot value is the [`TokenKind`].
//! [`OpTable::pack`] returns the slot value, for callers which do the comparison themselves.
//! [`OpTable::lookup`] does the whole lookup.
//! [`opmap_longest`] checks for 4-byte, then 3-byte, then 2-byte operators in turn.
//!
//! Hash multipliers for both tables are hard-coded. `test_perfect_hash` test checks
//! that they produce no collisions. If a change to the operator list breaks that,
//! the test's failure message gives a replacement.

use crate::token::{TokenKind, tk};

use crate::pipeline::bytes::is_digit;

/// Number of entries in table for 2-byte operators indexed by hash.
const OP2_HASH_TABLE_SIZE: usize = 32;

/// Multiplier for the 2-byte operator perfect hash.
const OP2_MUL: u32 = 0x058B_B283;

/// Number of entries in table for 3-byte operators indexed by hash.
const OP3_HASH_TABLE_SIZE: usize = 16;

/// Multiplier for the 3-byte operator perfect hash.
const OP3_MUL: u32 = 0x010E_2F79;

/// An operator and its corresponding [`TokenKind`].
struct OpDef {
    txt: &'static [u8],
    kind: TokenKind,
}

impl OpDef {
    /// Create new [`OpDef`].
    const fn new(txt: &'static str, kind: TokenKind) -> Self {
        assert!(txt.len() == 2 || txt.len() == 3, "`txt` must be 2 or 3 bytes long");
        Self { txt: txt.as_bytes(), kind }
    }

    /// Get length of this operator as a `u32`.
    const fn len(&self) -> u32 {
        self.txt.len() as u32
    }

    /// Get a `[u8; 4]` containing all the bytes of this operator and 0 for any remaining bytes.
    const fn bytes(&self) -> [u8; 4] {
        first_4_bytes(self.txt)
    }

    /// Get hash key for this operator.
    const fn key(&self) -> u32 {
        u32::from_le_bytes(self.bytes())
    }
}

/// All 2-byte and 3-byte operators.
///
/// Only exists at build time and in tests, not referenced from any runtime code.
static OPMAP_OPS: [OpDef; 32] = [
    OpDef::new("<=", TokenKind::Le),
    OpDef::new(">=", TokenKind::Ge),
    OpDef::new("==", TokenKind::EqEq),
    OpDef::new("!=", TokenKind::BangEq),
    OpDef::new("===", TokenKind::EqEqEq),
    OpDef::new("!==", TokenKind::BangEqEq),
    OpDef::new("**", TokenKind::StarStar),
    OpDef::new("++", TokenKind::PlusPlus),
    OpDef::new("--", TokenKind::MinusMinus),
    OpDef::new("<<", TokenKind::LShift),
    OpDef::new(">>", TokenKind::RShift),
    OpDef::new(">>>", TokenKind::URShift),
    OpDef::new("&&", TokenKind::AmpAmp),
    OpDef::new("||", TokenKind::PipePipe),
    OpDef::new("??", TokenKind::Nullish),
    OpDef::new("?.", TokenKind::OptionalChain),
    OpDef::new("=>", TokenKind::Arrow),
    OpDef::new("+=", TokenKind::PlusEq),
    OpDef::new("-=", TokenKind::MinusEq),
    OpDef::new("*=", TokenKind::StarEq),
    OpDef::new("%=", TokenKind::PercentEq),
    OpDef::new("<<=", TokenKind::LShiftEq),
    OpDef::new(">>=", TokenKind::RShiftEq),
    OpDef::new("&=", TokenKind::AmpEq),
    OpDef::new("|=", TokenKind::PipeEq),
    OpDef::new("^=", TokenKind::CaretEq),
    OpDef::new("&&=", TokenKind::AmpAmpEq),
    OpDef::new("||=", TokenKind::PipePipeEq),
    OpDef::new("??=", TokenKind::NullishEq),
    OpDef::new("**=", TokenKind::StarStarEq),
    OpDef::new("...", TokenKind::Ellipsis),
    OpDef::new("/=", TokenKind::SlashEq),
];

/// Details of the single 4-byte operator (`>>>=`).
const FOUR_BYTE_OP_BYTES: [u8; 4] = *b">>>=";
const FOUR_BYTE_OP_KIND: TokenKind = TokenKind::URShiftEq;

/// Returns `true` if a byte is one of the operator characters `=!<>+-*&|^%?.`.
///
/// Note `/` is excluded - callers handle it separately.
#[inline(always)]
pub(super) const fn is_op_char(c: u8) -> bool {
    const OPCH_LO: [u8; 16] = [0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 10, 3, 7, 2];
    const OPCH_HI: [u8; 16] = [0, 0, 1, 2, 0, 4, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];

    (OPCH_LO[(c & 15) as usize] & OPCH_HI[(c >> 4) as usize]) != 0
}

/// Hash table for 2-byte operators.
///
/// Aligned on a 128-byte boundary, so whole table sits in a 128-byte cache line
/// on Apple Silicon, and a pair of 64-byte cache lines on `x86_64`.
static OP2_TABLE_DATA: OpTableData<OP2_HASH_TABLE_SIZE, Align128> = OpTableData::new(2, OP2_MUL);
const OP2_TABLE: OpTable<OP2_HASH_TABLE_SIZE, Align128> = OpTable::new(&OP2_TABLE_DATA, 2, OP2_MUL);

/// Hash table for 3-byte operators.
///
/// Aligned on a 64-byte boundary, so whole table sits in a 64-byte cache line.
static OP3_TABLE_DATA: OpTableData<OP3_HASH_TABLE_SIZE, Align64> = OpTableData::new(3, OP3_MUL);
const OP3_TABLE: OpTable<OP3_HASH_TABLE_SIZE, Align64> = OpTable::new(&OP3_TABLE_DATA, 3, OP3_MUL);

/// ZST aligned on 64.
#[repr(align(64))]
struct Align64;

/// ZST aligned on 128.
#[repr(align(128))]
struct Align128;

/// Operator hash table.
struct OpTable<const TABLE_SIZE: usize, Align: 'static> {
    /// Lookup table data.
    data: &'static OpTableData<TABLE_SIZE, Align>,
    /// Length of operators in this table (2 or 3).
    len: u32,
    /// Hash multiplier.
    mul: u32,
}

impl<const TABLE_SIZE: usize, Align> OpTable<TABLE_SIZE, Align> {
    /// Create an operator perfect hash table for operators of length `len`,
    /// using multiplier `mul`.
    const fn new(data: &'static OpTableData<TABLE_SIZE, Align>, len: u32, mul: u32) -> Self {
        Self { data, len, mul }
    }

    /// Hash `key` and get the packed value for an operator from this hash table.
    ///
    /// `key` must contain:
    /// - Bytes 0-1: First 2 bytes of source
    /// - Byte  2  : 3rd byte of source for 3-byte operators, or 0 for 2-byte operators
    /// - Byte  3  : 0
    ///
    /// Returned value contains:
    /// - Bytes 0-1: First 2 bytes of operator
    /// - Byte  2  : 3rd byte of operator for 3-byte operators, or 0 for 2-byte operators
    /// - Byte  3  : [`TokenKind`] of the operator as a `u8`
    ///
    /// If no match, returns 0 (i.e. operator bytes `\0\0\0`, `TokenKind` byte 0).
    ///
    /// Hashmap collisions are possible, so caller must confirm the match with:
    /// - `(pack & 0xFFFF) == key` for 2-byte operators.
    /// - `(pack & 0xFF_FFFF) == key` for 3-byte operators.
    #[inline(always)]
    fn pack(&self, key: u32) -> u32 {
        let slot = self.data.slot(key, self.mul);
        self.data.values[slot]
    }

    /// Check if `bytes` starts with an operator from this hash table.
    ///
    /// * If an operator is found, returns the [`TokenKind`] of the operator as a `u32`.
    /// * Otherwise, returns 0.
    #[inline(always)]
    fn lookup(&self, bytes: [u8; 4]) -> u32 {
        let mask = if self.len == 2 { 0xFFFF } else { 0xFF_FFFF };
        let key = u32::from_le_bytes(bytes) & mask;
        let pack = self.pack(key);
        // If bottom 3 bytes of `key` are all 0, it's possible that `key` hashes to an empty slot,
        // so `pack == 0`. In that case `(pack & mask) == key` so `pack >> 24` is returned.
        // But for empty slots, `pack >> 24` is also 0, so 0 is returned either way.
        if (pack & mask) == key { pack >> 24 } else { 0 }
    }
}

/// Operator hash table data.
///
/// This has to be a separate type from [`OpTable`] as we want this stored in a `static`
/// whereas [`OpTable`] should just be a `const`, so it doesn't bloat the binary.
struct OpTableData<const TABLE_SIZE: usize, Align> {
    values: [u32; TABLE_SIZE],
    _align: [Align; 0],
}

impl<const TABLE_SIZE: usize, Align> OpTableData<TABLE_SIZE, Align> {
    /// Number of bits in hash for this table.
    const BITS: usize = TABLE_SIZE.trailing_zeros() as usize;

    /// Create the data for an operator perfect hash table for operators of length `len`,
    /// using multiplier `mul`.
    const fn new(len: u32, mul: u32) -> Self {
        assert!(TABLE_SIZE.is_power_of_two());

        let mut table = Self { values: [0; TABLE_SIZE], _align: [] };

        let mut i = 0;
        while i < OPMAP_OPS.len() {
            let op_def = &OPMAP_OPS[i];
            if op_def.len() == len {
                let key = op_def.key();
                let slot = table.slot(key, mul);
                table.values[slot] = key | ((op_def.kind as u32) << 24);
            }
            i += 1;
        }

        table
    }

    /// Get hash of `key`, which is the slot index into the table,
    /// using `mul` as the hash map multiplier.
    ///
    /// Returns a `usize` but it is guaranteed to be less than `TABLE_SIZE`.
    #[inline(always)]
    const fn slot(&self, key: u32, mul: u32) -> usize {
        (key.wrapping_mul(mul) >> (32 - Self::BITS)) as usize
    }
}

/// Hash `key` and get the packed value for a 2-byte operator from the hash table.
///
/// `key` must contain:
/// - Bytes 0-1: 2 bytes of source
/// - Bytes 2-3: 0
///
/// Returned value contains:
/// - Bytes 0-1: 2 bytes of operator
/// - Byte  2  : 0
/// - Byte  3  : [`TokenKind`] of the operator as a `u8`
///
/// If no match, returns 0 (i.e. operator bytes `\0\0`, `TokenKind` byte 0).
///
/// Hashmap collisions are possible, so caller must additionally check that
/// `(pack & 0xFFFF) == key` to confirm a match.
#[inline(always)]
pub(super) fn opmap_pack2(key: u32) -> u32 {
    OP2_TABLE.pack(key)
}

/// Hash `key` and get the packed value for a 3-byte operator from the hash table.
///
/// `key` must contain:
/// - Bytes 0-2: 3 bytes of source
/// - Byte  3  : 0
///
/// Returned value contains:
/// - Bytes 0-2: 3 bytes of operator
/// - Byte  3  : [`TokenKind`] of the operator as a `u8`
///
/// If no match, returns 0 (i.e. operator bytes `\0\0\0`, `TokenKind` byte 0).
///
/// Hashmap collisions are possible, so caller must additionally check that
/// `(pack & 0xFF_FFFF) == key` to confirm a match.
#[inline(always)]
pub(super) fn opmap_pack3(key: u32) -> u32 {
    OP3_TABLE.pack(key)
}

/// Check if up to 4 bytes of source contains a 2-byte, 3-byte, or 4-byte operator.
///
/// Searches for longest operator first, starting at `min(max_len, 4)` length.
///
/// `max_len` should be 2 or more to find an operator.
/// If `max_len` is < 2, and `bytes` contains padding `\0` bytes from after end of source
/// in last 3 or 4 bytes, no matching operator can be found.
///
/// If an operator is found, returns a tuple `(kind, len)` where:
/// - `kind` is the [`TokenKind`] of the operator as a `u32`
/// - `len` is the length of the found operator in bytes
///
/// If no operator is found, returns 0 as `kind`, and 1 as `len`.
///
/// `?.` followed by a digit is rejected as a match.
#[inline(always)]
pub(super) fn opmap_longest(bytes: [u8; 4], max_len: u32) -> (/* kind*/ u32, /* len */ u32) {
    if max_len >= 4 && bytes == FOUR_BYTE_OP_BYTES {
        return (FOUR_BYTE_OP_KIND as u32, 4);
    }

    if max_len >= 3 {
        let kind = OP3_TABLE.lookup(bytes);
        if kind != 0 {
            return (kind, 3);
        }
    }

    let kind = OP2_TABLE.lookup(bytes);
    if kind != 0 && !(kind == tk!(OptionalChain) as u32 && is_digit(bytes[2])) {
        return (kind, 2);
    }

    (0, 1)
}

/// Get a `[u8; 4]` containing all the bytes of `txt` and 0 for any remaining bytes.
const fn first_4_bytes(txt: &[u8]) -> [u8; 4] {
    let mut bytes = [0; 4];
    let mut i = 0;
    while i < txt.len() {
        bytes[i] = txt[i];
        i += 1;
    }
    bytes
}

#[cfg(test)]
mod tests {
    use crate::token::{OP_KIND_BASE, OP_KIND_MAX};

    use super::*;

    #[test]
    fn test_is_op_char() {
        const OPCHARS: &[u8] = b"=!<>+-*&|^%?.";

        let mut in_set = [false; 256];
        for &q in OPCHARS {
            in_set[q as usize] = true;
        }

        for c in 0..256usize {
            assert!(is_op_char(c as u8) == in_set[c], "OPCH_LO/HI wrong at byte {c:#04x}");
        }
    }

    #[test]
    fn test_op_defs_token_kind_range() {
        for (i, op_def) in OPMAP_OPS.iter().enumerate() {
            let kind = op_def.kind as u8;
            assert!(kind >= OP_KIND_BASE && kind <= OP_KIND_MAX, "OpDef {i}: `kind` out of range");
        }
    }

    #[test]
    fn test_op_defs_unique_token_kinds() {
        let mut seen = [false; 256];
        for op_def in &OPMAP_OPS {
            let kind = op_def.kind;
            assert!(!seen[kind as usize], "duplicate `TokenKind`: {kind}");
            seen[kind as usize] = true;
        }
    }

    #[test]
    fn test_op_defs_no_key_collisions() {
        for (index1, op_def1) in OPMAP_OPS.iter().enumerate() {
            for (index2, op_def2) in OPMAP_OPS.iter().enumerate().skip(index1 + 1) {
                assert!(op_def1.key() != op_def2.key(), "OpDef {index1}, {index2}: key collision");
            }
        }
    }

    #[test]
    fn test_optable_lookup_correct_kinds() {
        for (i, op_def) in OPMAP_OPS.iter().enumerate() {
            let bytes = op_def.bytes();
            let lookup_kind =
                if op_def.len() == 2 { OP2_TABLE.lookup(bytes) } else { OP3_TABLE.lookup(bytes) };
            assert!(lookup_kind == op_def.kind as u32, "OpDef {i}: `lookup` produced wrong kind");
        }
    }

    #[test]
    fn test_optable_lookup_returns_zero_on_no_match() {
        let cases = [
            // Not operators
            "..", "=/", "<<<", "&&&", "?..",
            // 3-byte candidate whose 1st 2 bytes are a 2-byte operator, and 3rd byte is `\0`.
            // `key` is identical for these and the 2-byte operators.
            // `lookup` needs to ensure they aren't misidentified as matching.
            "==\0", "<<\0", "||\0",
            // All bytes are 0, so `key` is 0. The check against `pack` passes if `key` hashes
            // to an empty slot. `lookup` must still return 0.
            "\0\0", "\0\0\0",
        ];
        for txt in cases {
            let bytes = first_4_bytes(txt.as_bytes());
            let kind =
                if txt.len() == 2 { OP2_TABLE.lookup(bytes) } else { OP3_TABLE.lookup(bytes) };
            assert!(kind == 0, "`lookup` should return 0 for {txt:?}");
        }
    }

    #[test]
    fn test_perfect_hash() {
        check_table(&OP2_TABLE);
        check_table(&OP3_TABLE);
    }

    fn check_table<const TABLE_SIZE: usize, Align>(table: &OpTable<TABLE_SIZE, Align>) {
        let len = table.len;
        let operator_count = OPMAP_OPS.iter().filter(|op_def| op_def.len() == len).count();

        // If table has no collisions, all good
        if is_collision_free(table.data, operator_count) {
            return;
        }

        // There was a collision - find a new multiplier which has no collisions
        let mut mul = (1u32 << 24) | 1;
        while mul < (1u32 << 28) {
            let table_data = OpTableData::<TABLE_SIZE, Align>::new(len, mul);
            if is_collision_free(&table_data, operator_count) {
                panic!(
                    "Current value for `OP{len}_MUL` produces collisions. Set it to 0x{:04X}_{:04X}.",
                    mul >> 16,
                    mul & 0xFFFF
                );
            }
            mul += 2;
        }

        panic!(
            "Current value for `OP{len}_MUL` produces collisions. Could not find another value."
        );
    }

    fn is_collision_free<const TABLE_SIZE: usize, Align>(
        table_data: &OpTableData<TABLE_SIZE, Align>,
        operator_count: usize,
    ) -> bool {
        let filled_slots_count = table_data.values.iter().filter(|&&pack| pack != 0).count();
        filled_slots_count == operator_count
    }
}
