//! Astro-specific lexer functionality
//!
//! Handles the frontmatter delimiter `---` and HTML body with JSX expressions.

use super::Lexer;

impl Lexer<'_> {
    /// Set the lexer position for Astro parsing.
    /// This is used to skip to a specific offset in the source.
    pub(crate) fn set_position_for_astro(&mut self, offset: u32) {
        let source_start = self.source.whole().as_ptr();
        // SAFETY: offset is within bounds of the source text
        let new_ptr = unsafe { source_start.add(offset as usize) };
        // SAFETY: Creating a SourcePosition from a valid offset within the source
        let new_pos = unsafe { super::source::SourcePosition::new(new_ptr) };
        self.source.set_position(new_pos);
    }
}
