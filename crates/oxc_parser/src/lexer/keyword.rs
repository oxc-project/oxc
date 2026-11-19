//! Classify a scanned identifier as a keyword or [`Kind::Ident`].
//!
//! This lookup lets all ASCII letters that can start a keyword share one identifier handler,
//! reducing the number of indirect-call targets in byte dispatch. It accepts either a source
//! slice or the decoded spelling of an escaped identifier; scanning and decoding happen elsewhere.
//!
//! Keywords are 2 to 11 bytes long. For those lengths, overlapping, in-bounds loads pack the
//! complete spelling into an eight-byte prefix and a three-byte suffix, with the length stored
//! alongside the suffix. A multiply-and-shift hash selects one entry in a 512-entry (8 KiB)
//! static table. Comparing the full spelling and length rejects non-keywords that hash to the
//! same slot. Lookup needs no allocation, padded input, or collision probing.
//!
//! The table is built at compile time from [`Kind::VARIANTS`], using [`Kind::is_any_keyword`]
//! and [`Kind::to_str`] as the source of keyword spellings. The hash gives each current keyword
//! a distinct slot; compile-time assertions reject collisions or unsupported keyword lengths.
//! Adding a keyword may require a new hash multiplier and an update to the byte-handler table
//! if its first letter currently bypasses keyword lookup.

use super::Kind;

#[derive(Clone, Copy)]
struct Entry {
    prefix: u64,
    suffix_and_len: u32,
    kind: Kind,
}

// Collision-free for the current keywords. Table construction rejects collisions
// when keywords change; choose a new multiplier if needed. Update the byte-handler
// table if a new keyword starts with a letter currently routed to IDT.
const HASH_SEED: u64 = 17_825_894_737_173_976_417;

// Only nine hash bits are retained, so the result fits usize on every target.
const fn slot(prefix: u64, suffix_and_len: u32) -> usize {
    ((prefix ^ ((suffix_and_len as u64) << 21)).wrapping_mul(HASH_SEED) >> 55) as usize
}

// Each 16-byte entry stores the complete keyword and its length. Hash matches alone
// do not establish equality; non-keywords which share a slot are rejected below.
static TABLE: [Entry; 512] = {
    let mut table = [Entry { prefix: 0, suffix_and_len: 0, kind: Kind::Ident }; 512];
    let mut index = 0;
    while index < Kind::VARIANTS.len() {
        let kind = Kind::VARIANTS[index];
        index += 1;
        if !kind.is_any_keyword() {
            continue;
        }
        let bytes = kind.to_str().as_bytes();
        assert!(bytes.len() >= 2 && bytes.len() <= 11);
        let mut prefix = [0; 8];
        let mut suffix_and_len = [0; 4];
        // The length check above proves this fits in a byte.
        #[expect(clippy::cast_possible_truncation)]
        let len = bytes.len() as u8;
        suffix_and_len[3] = len;
        let mut byte_index = 0;
        while byte_index < bytes.len() {
            if byte_index < 8 {
                prefix[byte_index] = bytes[byte_index];
            } else {
                suffix_and_len[byte_index - 8] = bytes[byte_index];
            }
            byte_index += 1;
        }
        let prefix = u64::from_le_bytes(prefix);
        let suffix_and_len = u32::from_le_bytes(suffix_and_len);
        let slot = slot(prefix, suffix_and_len);
        assert!(matches!(table[slot].kind, Kind::Ident), "keyword hash collision");
        table[slot] = Entry { prefix, suffix_and_len, kind };
    }
    table
};

#[expect(clippy::inline_always)]
#[inline(always)]
pub(super) fn match_keyword(bytes: &[u8]) -> Kind {
    let len = bytes.len();
    if !(2..12).contains(&len) {
        return Kind::Ident;
    }

    // Overlapping loads assemble the identifier without padding or reading beyond
    // its slice. Decoded escaped identifiers need not have any trailing source bytes.
    // SAFETY: The length check proves 2 <= len <= 11. Each branch reads the first
    // 2/4/8 bytes and the last 2/4 bytes, all within the slice. Unaligned loads do not
    // require the identifier to start at an aligned address.
    let (prefix, suffix) = unsafe {
        let ptr = bytes.as_ptr();
        if len <= 4 {
            let first = u64::from(u16::from_le(ptr.cast::<u16>().read_unaligned()));
            let last = u64::from(u16::from_le(ptr.add(len - 2).cast::<u16>().read_unaligned()));
            (first | (last << ((len - 2) * 8)), 0)
        } else if len <= 8 {
            let first = u64::from(u32::from_le(ptr.cast::<u32>().read_unaligned()));
            let last = u64::from(u32::from_le(ptr.add(len - 4).cast::<u32>().read_unaligned()));
            (first | (last << ((len - 4) * 8)), 0)
        } else {
            let first = u64::from_le(ptr.cast::<u64>().read_unaligned());
            let last = u32::from_le(ptr.add(len - 4).cast::<u32>().read_unaligned());
            (first, last >> ((12 - len) * 8))
        }
    };
    // The length check above proves len fits in u32.
    #[expect(clippy::cast_possible_truncation)]
    let suffix_and_len = suffix | ((len as u32) << 24);
    let entry = TABLE[slot(prefix, suffix_and_len)];
    if entry.prefix == prefix && entry.suffix_and_len == suffix_and_len {
        entry.kind
    } else {
        Kind::Ident
    }
}
