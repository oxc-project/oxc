//! Token

use bitflags::bitflags;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use super::{cold_branch, kind::Kind};

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    struct TokenFlags: u8 {
        /// Indicates the token is on a newline
        const IsOnNewLine = 1 << 0;

        /// True if the identifier / string / template kinds has escaped strings.
        /// The escaped strings are saved in [Lexer::escaped_strings] and [Lexer::escaped_templates] by
        /// [Token::start].
        ///
        /// [Lexer::escaped_strings]: [super::Lexer::escaped_strings]
        /// [Lexer::escaped_templates]: [super::Lexer::escaped_templates]
        const Escaped = 1 << 1;

        /// True if for numeric literal tokens that contain separator characters (`_`).
        ///
        /// Numeric literals are defined in Section 12.9.3 of the ECMAScript
        /// standard and include [`Kind::Decimal`], [`Kind::Binary`],
        /// [`Kind::Octal`], [`Kind::Hex`], etc.
        const HasSeparator = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Token {
    /// Token Kind
    pub kind: Kind,

    /// Start offset in source
    pub start: u32,

    /// Length of the token
    ///
    /// [u16::MAX] is stored here if the token's length is greater than `u16::MAX`
    /// (which is rare but can happen with large strings).
    /// Actual ends of long tokens are stored in `lexer.long_token_ends`.
    pub len: u16,

    flags: TokenFlags,
}

impl Token {
    pub(super) fn new_on_new_line() -> Self {
        Self { flags: TokenFlags::IsOnNewLine, ..Self::default() }
    }

    #[inline]
    pub fn span(&self, long_ends: &FxHashMap<u32, u32>) -> Span {
        Span::new(self.start, self.end(long_ends))
    }

    #[inline]
    pub fn set_end(&mut self, end: u32, long_ends: &mut FxHashMap<u32, u32>) {
        if let Ok(len) = u16::try_from(end - self.start) {
            self.len = len;
        } else {
            cold_branch(|| {
                self.len = u16::MAX;
                long_ends.insert(self.start, end);
            });
        }
    }

    #[inline]
    pub fn end(&self, long_ends: &FxHashMap<u32, u32>) -> u32 {
        #[allow(clippy::if_not_else)]
        if self.len != u16::MAX {
            self.start + u32::from(self.len)
        } else {
            cold_branch(|| {
                long_ends.get(&self.start).copied().unwrap_or_else(||
                    // The token's length happens to be exact `u16::MAX`
                    self.start + u32::from(u16::MAX))
            })
        }
    }

    #[inline]
    pub fn escaped(&self) -> bool {
        self.flags.contains(TokenFlags::Escaped)
    }
    #[inline]
    pub fn set_escaped(&mut self) {
        self.flags.insert(TokenFlags::Escaped);
    }

    #[inline]
    pub fn is_on_new_line(&self) -> bool {
        self.flags.contains(TokenFlags::IsOnNewLine)
    }

    #[inline]
    pub fn set_is_on_new_line(&mut self) {
        self.flags.insert(TokenFlags::IsOnNewLine);
    }

    #[inline]
    pub fn has_separator(&self) -> bool {
        let has_separator = self.flags.contains(TokenFlags::HasSeparator);
        debug_assert!(!has_separator || self.kind.is_number());
        has_separator
    }

    pub(crate) fn set_has_separator(&mut self) {
        debug_assert!(
            !self.flags.contains(TokenFlags::HasSeparator)
                || self.kind.is_number()
                || self.kind == Kind::default()
        );
        self.flags.insert(TokenFlags::HasSeparator);
    }
}

#[cfg(test)]
mod size_asserts {
    use super::Token;
    const _: () = assert!(std::mem::size_of::<Token>() == 8);
}
