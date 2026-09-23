//! Forward context walk.
//!
//! The hard regex-vs-division questions ("did this `}` close a value?", "is this `yield` a
//! keyword?", "does a type annotation end here?") are questions about the parser's state, which is
//! a pushdown state: a stack of bracket frames tagged with what they opened, a few scope flags, and
//! whether an operand may start next. That state is a function of the token prefix, so it is
//! computed forward and memoized instead of reconstructed backward at every site.
//!
//! The walk is lazy and reads the carved bitmaps as carve left them: keywords are still
//! identifiers, multi-byte operators are still one token start per byte, and `misc_pre` has already
//! split Unicode whitespace. A query walks from a nearby anchor whose context is certain (see
//! [`anchor`]), skipping the groups and members the scan back to it recorded; the memoized walk
//! from the start of the source answers when no anchor is in reach.
//!
//! There are 9 files:
//!
//! - [`frame`]: what the walk remembers per frame, and the keyword codes it reads off the source.
//! - [`walker`]: the walk's state and stack operations, the jump plan, and what a query reads.
//! - [`step`]: stepping one token: the line-break rules, literals, and the dispatch below.
//! - [`words`]: words: contextual keywords, statement keywords, names and member keys.
//! - [`punct`]: punctuation in expression context: operators, arrows, separators, brackets.
//! - [`types`]: punctuation inside a type: regions, angle lists, where a type ends.
//! - [`jsx`]: inside a JSX tag or element.
//! - [`anchor`]: the tokens where a bounded walk may start.
//! - [`scan`]: the scan back to an anchor, and the entry points every question goes through.

use super::common::Tokens;
use super::type_context::{lt_run_split, type_args_at};

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

pub(super) use scan::{after, after_from, angles_before, before};

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
    /// Open `<` lists a `>` run here would close.
    pub angles: usize,
}

/// The two walks a lex keeps: the memoized walk from the start of the source, and the bounded
/// walk started at the anchor nearest the last query.
pub(crate) struct Walks {
    full: Walk,
    local: Walk,
}

impl Walks {
    pub(crate) fn new() -> Walks {
        Walks { full: Walk::new(), local: Walk::new() }
    }

    /// A new lex, or a new pass over it by a stage that sees other token kinds: the full walk
    /// starts over and the bounded walk forgets its anchor.
    pub(crate) fn restart(&mut self, module: bool) {
        self.full.reset(module);
        self.local.seed_lost = true;
    }
}

/// [`after`] on the full walk from the start of the source: needed when the answer depends on the
/// enclosing functions (`yield` / `await`).
pub(super) fn after_scoped(tokens: &Tokens, walks: &mut Walks, pos: usize) -> After {
    let w = &mut walks.full;
    if let Some((p, a)) = w.last_query
        && p == pos
    {
        return a;
    }
    let inside_last = pos < w.walked_to && pos >= w.last_start;
    if w.walked_to > pos && !inside_last {
        w.reset(tokens.module);
    }
    w.advance(tokens, pos);
    // A query inside the token just processed (the tail of a fused operator run such as `>>>`)
    // is answered by the state after it.
    let a = if w.walked_to > pos { w.classify_after() } else { w.after_token(tokens, pos) };
    w.last_query = Some((pos, a));
    a
}
