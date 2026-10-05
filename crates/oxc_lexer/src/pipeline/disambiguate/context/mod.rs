use super::common::Tokens;
use super::type_context::{keyword_type, lt_run_split, type_args_at};

mod anchor;
mod frame;
mod jsx;
mod punct;
mod scan;
mod step;
mod types;
mod walker;
mod words;

use frame::*;
use walker::{Expect, Jump, Walk};

pub(super) use scan::{after, angles_before, before};

#[cfg(test)]
mod tests;

/// What the token at a site leaves behind, for the `/` right after it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum After {
    /// The token ends a value: `/` is division.
    Value,
    /// An operand may start: `/` is a regex.
    Operand,
    /// The token ends a declaration nothing can continue (an annotation, a declarator without
    /// initializer, a bodiless signature, `break label`, a module specifier): `/` is a regex.
    EndsDecl,
}

/// Context at the (not yet processed) token starting at a position.
#[derive(Clone, Copy, Debug)]
pub(super) struct Site {
    /// Inside a type: annotation, alias, type argument list, type literal.
    pub in_type: bool,
    /// An operand may start here, in expression context.
    pub operand: bool,
    /// A `<` here opens the type-parameter list of a declaration or member.
    pub type_params: bool,
}

/// The two walks a lex keeps: the memoized walk from the start of the source, and the bounded
/// walk started at the anchor nearest the last query.
pub(crate) struct Walks {
    full: Walk,
    local: Walk,
    /// Scan steps the bounded walks took since the last restart.
    spent: usize,
    /// Tests: every question goes to the full walk, with no anchors or shortcuts.
    #[cfg(test)]
    pub(crate) full_walk_only: bool,
    /// Tests: a > run goes to the walk without the rules that settle it from nearby tokens.
    #[cfg(test)]
    pub(crate) no_run_rules: bool,
}

impl Walks {
    pub(crate) fn new() -> Walks {
        Walks {
            full: Walk::new(),
            local: Walk::new(),
            spent: 0,
            #[cfg(test)]
            full_walk_only: false,
            #[cfg(test)]
            no_run_rules: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn shortcuts(&self) -> bool {
        !self.full_walk_only
    }

    #[cfg(not(test))]
    pub(crate) fn shortcuts(&self) -> bool {
        true
    }

    #[cfg(test)]
    pub(crate) fn run_rules(&self) -> bool {
        !self.full_walk_only && !self.no_run_rules
    }

    #[cfg(not(test))]
    pub(crate) fn run_rules(&self) -> bool {
        true
    }

    /// A new lex, or a new pass over it by a stage that sees other token kinds: the full walk
    /// starts over and the bounded walks get a new budget.
    pub(crate) fn restart(&mut self, module: bool) {
        self.full.reset(module);
        self.spent = 0;
    }

    /// The full walk advanced to pos, restarted if it has passed the token there.
    fn full_to(&mut self, tokens: &Tokens, pos: usize) -> &mut Walk {
        let w = &mut self.full;
        let inside_last = pos < w.walked_to && pos >= w.last_start;
        if w.walked_to > pos && !inside_last {
            w.reset(tokens.module);
        }
        w.advance(tokens, pos);
        w
    }
}
