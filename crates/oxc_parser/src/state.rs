use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::AssignmentExpression;
use oxc_span::Span;

pub struct ParserState<'a> {
    not_parenthesized_arrow: Option<FxHashSet<u32>>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    cover_initialized_name: Option<FxHashMap<u32, AssignmentExpression<'a>>>,

    /// Trailing comma spans for `ArrayExpression`.
    /// Used for error reporting.
    /// Keyed by start span of `ArrayExpression`.
    /// Valued by position of the trailing_comma.
    trailing_commas: Option<FxHashMap<u32, Span>>,
}

impl<'a> ParserState<'a> {
    pub fn new() -> Self {
        Self { not_parenthesized_arrow: None, cover_initialized_name: None, trailing_commas: None }
    }

    /// Get or create the not_parenthesized_arrow set
    #[inline]
    pub fn not_parenthesized_arrow(&mut self) -> &mut FxHashSet<u32> {
        self.not_parenthesized_arrow.get_or_insert_with(FxHashSet::default)
    }

    /// Check if position is in not_parenthesized_arrow set
    #[inline]
    pub fn contains_not_parenthesized_arrow(&self, pos: &u32) -> bool {
        self.not_parenthesized_arrow.as_ref().map_or(false, |set| set.contains(pos))
    }

    /// Get or create the cover_initialized_name map
    #[inline]
    pub fn cover_initialized_name(&mut self) -> &mut FxHashMap<u32, AssignmentExpression<'a>> {
        self.cover_initialized_name.get_or_insert_with(FxHashMap::default)
    }

    /// Get reference to cover_initialized_name map if it exists
    #[inline]
    pub fn cover_initialized_name_ref(&self) -> Option<&FxHashMap<u32, AssignmentExpression<'a>>> {
        self.cover_initialized_name.as_ref()
    }

    /// Get or create the trailing_commas map
    #[inline]
    pub fn trailing_commas(&mut self) -> &mut FxHashMap<u32, Span> {
        self.trailing_commas.get_or_insert_with(FxHashMap::default)
    }

    /// Get reference to trailing_commas map if it exists
    #[inline]
    pub fn trailing_commas_ref(&self) -> Option<&FxHashMap<u32, Span>> {
        self.trailing_commas.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_initialization() {
        // Create a new parser state
        let mut state = ParserState::new();
        
        // Verify all collections start as None (not allocated)
        assert!(state.cover_initialized_name_ref().is_none());
        assert!(state.trailing_commas_ref().is_none());
        assert!(!state.contains_not_parenthesized_arrow(&42));
        
        // Access collections to trigger lazy initialization
        let _trailing_commas = state.trailing_commas();
        let _not_parenthesized = state.not_parenthesized_arrow();
        let _cover_init = state.cover_initialized_name();
        
        // Verify collections are now allocated
        assert!(state.cover_initialized_name_ref().is_some());
        assert!(state.trailing_commas_ref().is_some());
        
        // Verify they're empty but allocated
        assert_eq!(state.cover_initialized_name_ref().unwrap().len(), 0);
        assert_eq!(state.trailing_commas_ref().unwrap().len(), 0);
    }
}
