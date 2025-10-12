use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::AssignmentExpression;
use oxc_span::Span;

use crate::lexer::Kind;

pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, AssignmentExpression<'a>>,

    /// Trailing comma spans for `ArrayExpression`.
    /// Used for error reporting.
    /// Keyed by start span of `ArrayExpression`.
    /// Valued by position of the trailing_comma.
    pub trailing_commas: FxHashMap<u32, Span>,

    list_capacity_hints: ListCapacityHints,
}

impl ParserState<'_> {
    pub fn new() -> Self {
        Self {
            not_parenthesized_arrow: FxHashSet::default(),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            list_capacity_hints: ListCapacityHints::default(),
        }
    }

    pub const fn list_capacity_hint(&self, close: Kind) -> usize {
        self.list_capacity_hints.hint(close)
    }

    pub fn update_list_capacity_hint(&mut self, close: Kind, capacity: usize) {
        self.list_capacity_hints.update(close, capacity);
    }
}

#[derive(Default, Clone, Copy)]
struct ListCapacityHints {
    r_curly: usize,
    r_brack: usize,
    r_paren: usize,
    r_angle: usize,
    other: usize,
}

impl ListCapacityHints {
    const MAX_HINT: usize = 1024;

    const fn hint(&self, close: Kind) -> usize {
        match close {
            Kind::RCurly => self.r_curly,
            Kind::RBrack => self.r_brack,
            Kind::RParen => self.r_paren,
            Kind::RAngle => self.r_angle,
            _ => self.other,
        }
    }

    fn update(&mut self, close: Kind, capacity: usize) {
        let slot = match close {
            Kind::RCurly => &mut self.r_curly,
            Kind::RBrack => &mut self.r_brack,
            Kind::RParen => &mut self.r_paren,
            Kind::RAngle => &mut self.r_angle,
            _ => &mut self.other,
        };
        *slot = Self::next_hint(*slot, capacity);
    }

    fn next_hint(current: usize, capacity: usize) -> usize {
        let capacity = capacity.min(Self::MAX_HINT);
        if capacity == 0 {
            return current / 2;
        }
        if current == 0 || capacity > current { capacity } else { current }
    }
}
