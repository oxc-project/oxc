//! Token

use std::{fmt, mem, ptr::NonNull};

use oxc_span::Span;

use super::kind::Kind;

// Bit layout for `u128`:
// - Bits 0-31 (32 bits): `start` (`u32`)
// - Bits 32-63 (32 bits): `end` (`u32`)
// - Bits 64-71 (8 bits): `kind` (`Kind`)
// - Bits 72-79 (8 bits): `is_on_new_line` (`bool`)
// - Bits 80-87 (8 bits): `escaped` (`bool`)
// - Bits 88-95 (8 bits): `has_separator` (`bool`)
// - Bits 96-127 (32 bits): unused

const START_SHIFT: usize = 0;
const END_SHIFT: usize = 32;
const KIND_SHIFT: usize = 64;
const IS_ON_NEW_LINE_SHIFT: usize = 72;
const ESCAPED_SHIFT: usize = 80;
const HAS_SEPARATOR_SHIFT: usize = 88;

const START_MASK: u128 = 0xFFFF_FFFF; // 32 bits
const END_MASK: u128 = 0xFFFF_FFFF; // 32 bits
const KIND_MASK: u128 = 0xFF; // 8 bits
#[expect(dead_code)]
const BOOL_MASK: u128 = 0xFF; // 8 bits

const _: () = {
    const fn is_valid_shift<T>(shift: usize) -> bool {
        let align_bits = align_of::<T>() * 8;
        shift.is_multiple_of(align_bits) && shift < u128::BITS as usize
    }

    // Check `u32` fields are aligned on 32 and in bounds, so can be read/written via pointers
    assert!(is_valid_shift::<u32>(START_SHIFT));
    assert!(is_valid_shift::<u32>(END_SHIFT));

    // Check `Kind` is 1 byte, and `KIND_SHIFT` is aligned on 8 and in bounds, so can be read/written via pointers
    assert!(size_of::<Kind>() == 1);
    assert!(align_of::<Kind>() == 1);
    assert!(is_valid_shift::<Kind>(KIND_SHIFT));

    // Check flags fields are aligned on 8 and in bounds, so can be read/written via pointers
    assert!(is_valid_shift::<bool>(IS_ON_NEW_LINE_SHIFT));
    assert!(is_valid_shift::<bool>(ESCAPED_SHIFT));
    assert!(is_valid_shift::<bool>(HAS_SEPARATOR_SHIFT));
};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Token(u128);

impl Default for Token {
    #[inline]
    fn default() -> Self {
        // `Kind::default()` is `Kind::Eof`. So `0` is equivalent to:
        // start: 0,
        // end: 0,
        // kind: Kind::default(),
        // is_on_new_line: false,
        // escaped: false,
        // has_separator: false,
        const _: () = assert!(Kind::Eof as u8 == 0);
        Self(0)
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("kind", &self.kind())
            .field("start", &self.start())
            .field("end", &self.end())
            .field("is_on_new_line", &self.is_on_new_line())
            .field("escaped", &self.escaped())
            .field("has_separator", &self.has_separator())
            .finish()
    }
}

impl Token {
    #[inline]
    pub(super) fn new(kind: Kind, start: u32, end: u32, pending: PendingToken) -> Self {
        debug_assert!(end >= start);
        Self(
            (u128::from(start) << START_SHIFT)
                | (u128::from(end) << END_SHIFT)
                | ((kind as u128) << KIND_SHIFT)
                | (u128::from(pending.0 & PendingToken::TOKEN_FLAGS) << IS_ON_NEW_LINE_SHIFT),
        )
    }
}

/// Flags of the token being lexed, and trivia state since the previous token, packed in one `u32`
/// so the lexer reads and resets all per-token state with one load and one store.
///
/// The low 3 bytes are `is_on_new_line`, `escaped` and `has_separator`, in the same layout as in
/// [`Token`]. The other bits are trivia state, used by `TriviaBuilder`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct PendingToken(u32);

impl PendingToken {
    const IS_ON_NEW_LINE: u32 = 1;
    const ESCAPED: u32 = 1 << (ESCAPED_SHIFT - IS_ON_NEW_LINE_SHIFT);
    const HAS_SEPARATOR: u32 = 1 << (HAS_SEPARATOR_SHIFT - IS_ON_NEW_LINE_SHIFT);
    const TOKEN_FLAGS: u32 = Self::IS_ON_NEW_LINE | Self::ESCAPED | Self::HAS_SEPARATOR;
    /// Saw a newline since the last token.
    const SAW_NEWLINE: u32 = 1 << 24;
    /// Saw a newline since the last comment or token.
    const SAW_NEWLINE_FOR_COMMENT: u32 = 1 << 25;
    /// Comments have been added which are not yet attached to a token.
    const HAS_PENDING_COMMENTS: u32 = 1 << 26;

    pub(super) const NONE: Self = Self(0);
    pub(super) const START_OF_FILE: Self =
        Self(Self::IS_ON_NEW_LINE | Self::SAW_NEWLINE | Self::SAW_NEWLINE_FOR_COMMENT);

    #[inline]
    pub(super) fn is_on_new_line(self) -> bool {
        self.0 & Self::IS_ON_NEW_LINE != 0
    }

    #[inline]
    pub(super) fn set_is_on_new_line(&mut self) {
        self.0 |= Self::IS_ON_NEW_LINE;
    }

    #[inline]
    pub(super) fn set_escaped(&mut self) {
        self.0 |= Self::ESCAPED;
    }

    #[inline]
    pub(super) fn set_has_separator(&mut self) {
        self.0 |= Self::HAS_SEPARATOR;
    }

    #[inline]
    pub(super) fn saw_newline(self) -> bool {
        self.0 & Self::SAW_NEWLINE != 0
    }

    #[inline]
    pub(super) fn saw_newline_for_comment(self) -> bool {
        self.0 & Self::SAW_NEWLINE_FOR_COMMENT != 0
    }

    #[inline]
    pub(super) fn set_saw_newline(&mut self) {
        self.0 |= Self::SAW_NEWLINE | Self::SAW_NEWLINE_FOR_COMMENT;
    }

    #[inline]
    pub(super) fn clear_saw_newline_for_comment(&mut self) {
        self.0 &= !Self::SAW_NEWLINE_FOR_COMMENT;
    }

    #[inline]
    pub(super) fn has_pending_comments(self) -> bool {
        self.0 & Self::HAS_PENDING_COMMENTS != 0
    }

    #[inline]
    pub(super) fn set_has_pending_comments(&mut self, value: bool) {
        if value {
            self.0 |= Self::HAS_PENDING_COMMENTS;
        } else {
            self.0 &= !Self::HAS_PENDING_COMMENTS;
        }
    }
}

// Getters and setters.
//
// Prior to Rust 1.95.0, `set` methods used safe bitwise operations.
// This regressed heavily in Rust 1.95.0 due to an LLVM bug:
// https://github.com/oxc-project/oxc/pull/21509
// https://github.com/rust-lang/rust/issues/155422
//
// To obtain the same tight assembly as before on Rust 1.95.0, we now use unsafe pointer manipulation
// to directly write the "fields" of `Token`.
// The original implementations are kept in comments, in case we want to revert to them once the LLVM bug is fixed.
impl Token {
    #[inline]
    pub fn span(&self) -> Span {
        Span::new(self.start(), self.end())
    }

    // `set_span` is only exposed as public API when `mutate_tokens` feature is enabled.
    // Otherwise, it is only accessible within `lexer` module.
    #[cfg(feature = "mutate_tokens")]
    #[inline]
    pub fn set_span(&mut self, span: Span) {
        self.set_span_impl(span);
    }

    #[cfg(not(feature = "mutate_tokens"))]
    #[inline]
    #[allow(dead_code, clippy::allow_attributes)]
    pub(super) fn set_span(&mut self, span: Span) {
        self.set_span_impl(span);
    }

    #[inline]
    fn set_span_impl(&mut self, span: Span) {
        // On little-endian systems, `start` and `end` fields in `Span` are in same order as in `Token`,
        // so compiler boils this down to just a `u64` write of the `Span` into the first 8 bytes of the `Token`
        // https://godbolt.org/z/bdY5ccad6
        self.set_start(span.start);
        self.set_end(span.end);
    }

    #[inline]
    pub fn start(&self) -> u32 {
        ((self.0 >> START_SHIFT) & START_MASK) as u32
    }

    #[inline]
    pub(super) fn set_start(&mut self, start: u32) {
        /*
        // Original version. Perf regressed in Rust 1.95.0.
        self.0 &= !(START_MASK << START_SHIFT); // Clear current `start` bits
        self.0 |= u128::from(start) << START_SHIFT;
        */

        // SAFETY: `START_SHIFT` is a valid `u32` field position in `Token`
        unsafe { self.write_u32(START_SHIFT, start) };
    }

    #[inline]
    pub fn end(&self) -> u32 {
        ((self.0 >> END_SHIFT) & END_MASK) as u32
    }

    #[inline]
    pub(super) fn set_end(&mut self, end: u32) {
        let start = self.start();
        debug_assert!(end >= start, "Token end ({end}) cannot be less than start ({start})");

        /*
        // Original version. Perf regressed in Rust 1.95.0.
        self.0 &= !(END_MASK << END_SHIFT); // Clear current `end` bits
        self.0 |= u128::from(end) << END_SHIFT;
        */

        // SAFETY: `END_SHIFT` is a valid `u32` field position in `Token`
        unsafe { self.write_u32(END_SHIFT, end) };
    }

    #[inline]
    pub fn kind(&self) -> Kind {
        // SAFETY: `Kind` is `#[repr(u8)]`. Only `Token::default` and `Token::set_kind` set these bits,
        // and they set them to the `u8` value of an existing `Kind`.
        // So transmuting these bits back to `Kind` must produce a valid `Kind`.
        unsafe { mem::transmute::<u8, Kind>(((self.0 >> KIND_SHIFT) & KIND_MASK) as u8) }
    }

    // `set_kind` is only exposed as public API when `mutate_tokens` feature is enabled.
    // Otherwise, it is only accessible within `lexer` module.
    #[cfg(feature = "mutate_tokens")]
    #[inline]
    pub fn set_kind(&mut self, kind: Kind) {
        self.set_kind_impl(kind);
    }

    #[cfg(not(feature = "mutate_tokens"))]
    #[inline]
    pub(super) fn set_kind(&mut self, kind: Kind) {
        self.set_kind_impl(kind);
    }

    #[inline]
    fn set_kind_impl(&mut self, kind: Kind) {
        /*
        // Original version. Perf regressed in Rust 1.95.0.
        self.0 &= !(KIND_MASK << KIND_SHIFT); // Clear current `kind` bits
        self.0 |= u128::from(kind as u8) << KIND_SHIFT;
        */

        const OFFSET: usize =
            if cfg!(target_endian = "little") { KIND_SHIFT / 8 } else { 15 - (KIND_SHIFT / 8) };
        // SAFETY: `Kind` is `#[repr(u8)]`, so writing one byte at `OFFSET` overwrites only the `kind` byte
        // without touching adjacent fields. These bits always represent a valid `Kind`.
        // `Token` is borrowed mutably, so the write is unaliased.
        unsafe { *NonNull::from(self).cast::<Kind>().add(OFFSET).as_mut() = kind };
    }

    /// Checks if this token appears at the start of a new line.
    ///
    /// Returns `true` if the token was preceded by a line terminator during lexical analysis.
    /// This information is crucial for automatic semicolon insertion (ASI) and other
    /// JavaScript parsing rules that depend on line boundaries.
    #[inline]
    pub fn is_on_new_line(&self) -> bool {
        // Use a pointer read rather than arithmetic as it produces less instructions.
        // SAFETY: 8 bits starting at `IS_ON_NEW_LINE_SHIFT` are only set in `Token::default` and
        // `Token::new`. Both only set these bits to 0 or 1, so valid to read as a `bool`.
        unsafe { self.read_bool(IS_ON_NEW_LINE_SHIFT) }
    }

    #[inline]
    pub fn escaped(&self) -> bool {
        // Use a pointer read rather than arithmetic as it produces less instructions.
        // SAFETY: 8 bits starting at `ESCAPED_SHIFT` are only set in `Token::default` and
        // `Token::new`. Both only set these bits to 0 or 1, so valid to read as a `bool`.
        unsafe { self.read_bool(ESCAPED_SHIFT) }
    }

    #[inline]
    pub fn has_separator(&self) -> bool {
        // Use a pointer read rather than arithmetic as it produces less instructions.
        // SAFETY: 8 bits starting at `HAS_SEPARATOR_SHIFT` are only set in `Token::default` and
        // `Token::new`. Both only set these bits to 0 or 1, so valid to read as a `bool`.
        unsafe { self.read_bool(HAS_SEPARATOR_SHIFT) }
    }

    /// Read `bool` from 8 bits starting at bit position `shift`.
    ///
    /// # SAFETY
    ///
    /// `shift` must be the location of a valid boolean "field" in [`Token`]
    /// e.g. `ESCAPED_SHIFT`. The caller must guarantee that the 8 bits at
    /// `shift` contain only 0 or 1, making it safe to read as a `bool`.
    ///
    /// # Performance analysis
    ///
    /// This method uses unsafe pointer arithmetic to directly read a boolean value
    /// from the token's 128-bit representation. This approach is deliberately chosen
    /// for performance optimization on hot paths.
    ///
    /// This unsafe pointer arithmetic approach generates only 1 CPU instruction:
    /// ```asm
    /// movzx   eax, byte ptr [rdi + 9]  ; Load byte at offset
    /// ```
    ///
    /// Compared to the safe bit-shift alternative:
    /// ```ignore
    /// (token.0 >> shift) & 1 != 0
    /// ```
    ///
    /// ```asm
    /// movzx   eax, byte ptr [rdi + 9]  ; Load byte at offset
    /// and     al, 1                    ; Mask to lower bit only
    /// ```
    ///
    /// <https://godbolt.org/z/7xxrP348P>
    ///
    /// This optimization was retained after careful benchmarking (see PR #13788),
    /// where the single instruction difference on hot paths justified keeping
    /// the unsafe implementation.
    #[expect(clippy::inline_always)]
    #[inline(always)] // So `shift` is statically known
    unsafe fn read_bool(&self, shift: usize) -> bool {
        // Byte offset depends on endianness of the system
        let offset = if cfg!(target_endian = "little") { shift / 8 } else { 15 - (shift / 8) };
        // SAFETY: Caller guarantees `shift` points to valid `bool`.
        // This method borrows `Token`, so valid to read field via a reference - can't be aliased.
        unsafe {
            let field_ptr = NonNull::from_ref(self).cast::<bool>().add(offset);
            debug_assert!(field_ptr.cast::<u8>().read() <= 1);
            *field_ptr.as_ref()
        }
    }

    /// Write `u32` to the 32 bits starting at bit position `shift`.
    ///
    /// # SAFETY
    ///
    /// `shift` must be the location of a valid `u32` "field" in [`Token`] i.e. `START_SHIFT` or `END_SHIFT`.
    ///
    /// # Performance analysis
    ///
    /// Writing the whole `u32` via a pointer avoids a read-modify-write of the underlying `u128`.
    /// LLVM stopped folding the safe `self.0 &= !mask; self.0 |= val << shift` pattern
    /// into a single store as of rustc 1.95.0 - see <https://github.com/rust-lang/rust/issues/155422>.
    /// This implementation produces `mov dword ptr [rdi + N], esi` on both affected and unaffected Rust versions.
    #[expect(clippy::inline_always)]
    #[inline(always)] // So `shift` is statically known
    unsafe fn write_u32(&mut self, shift: usize, value: u32) {
        // `Token` is 16 bytes = 4 `u32`s wide. Offset in `u32` units depends on endianness.
        let offset = if cfg!(target_endian = "little") { shift / 32 } else { 3 - (shift / 32) };
        // SAFETY: Caller guarantees `shift` points to a valid `u32` field (`start` or `end`).
        // `Token` is `#[repr(transparent)]` over `u128`, so casting `NonNull<Token>` to `NonNull<u32>`
        // is going from stricter to looser alignment. `Token` is borrowed mutably, so the
        // write is unaliased - `as_mut` produces a `&mut u32` with `noalias` metadata for LLVM.
        unsafe { *NonNull::from(self).cast::<u32>().add(offset).as_mut() = value };
    }
}

#[cfg(test)]
mod test {
    use super::{Kind, PendingToken, Span, Token};

    // Test size of `Token`
    const _: () = assert!(size_of::<Token>() == 16);

    // Test default token values
    #[test]
    fn default_token_values() {
        let token = Token::default();
        assert_eq!(token.start(), 0);
        assert_eq!(token.end(), 0);
        assert_eq!(token.kind(), Kind::Eof); // Kind::default() is Eof
        assert!(!token.is_on_new_line());
        assert!(!token.escaped());
        assert!(!token.has_separator());
    }

    #[test]
    fn new_token() {
        let token = Token::new(Kind::Ident, 3, 7, PendingToken::NONE);
        assert_eq!(token.kind(), Kind::Ident);
        assert_eq!(token.span(), Span::new(3, 7));
        assert!(!token.is_on_new_line());
        assert!(!token.escaped());
        assert!(!token.has_separator());

        // Trivia state is not copied into `Token`
        let token = Token::new(Kind::Eof, 0, 0, PendingToken::START_OF_FILE);
        assert!(token.is_on_new_line());
        assert!(!token.escaped());
        assert!(!token.has_separator());

        let mut pending = PendingToken::NONE;
        pending.set_escaped();
        pending.set_has_separator();
        pending.set_has_pending_comments(true);
        let mut token = Token::new(Kind::Str, 30, 33, pending);
        assert!(!token.is_on_new_line());
        assert!(token.escaped());
        assert!(token.has_separator());

        // Setters do not affect other fields
        token.set_span(Span::new(10, 15));
        token.set_kind(Kind::Ident);
        assert_eq!(token.span(), Span::new(10, 15));
        assert_eq!(token.kind(), Kind::Ident);
        assert!(!token.is_on_new_line());
        assert!(token.escaped());
        assert!(token.has_separator());
    }

    #[test]
    fn pending_token_trivia_state() {
        let mut pending = PendingToken::NONE;
        pending.set_saw_newline();
        assert!(pending.saw_newline());
        assert!(pending.saw_newline_for_comment());
        pending.clear_saw_newline_for_comment();
        assert!(pending.saw_newline());
        assert!(!pending.saw_newline_for_comment());
        pending.set_has_pending_comments(true);
        assert!(pending.has_pending_comments());
        pending.set_has_pending_comments(false);
        assert!(!pending.has_pending_comments());
        assert!(!pending.is_on_new_line());
    }
}
