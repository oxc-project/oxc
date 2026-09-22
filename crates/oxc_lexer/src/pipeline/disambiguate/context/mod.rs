use super::common::Tokens;
use super::type_context::{lt_run_opens_type_args, type_args_at};

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum After {
    Value,
    Operand,
    /// Ends a declaration nothing can continue: / is a regex.
    EndsDecl,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Site {
    /// Inside a type: annotation, alias, type argument list, type literal.
    pub in_type: bool,
    /// An operand may start here, in expression context.
    pub operand: bool,
    /// A < here opens the type-parameter list of a declaration or member.
    pub type_params: bool,
    /// Open < lists a > run here would close.
    pub angles: usize,
}

pub(crate) struct Walks {
    full: Walk,
    local: Walk,
}

impl Walks {
    pub(crate) fn new() -> Walks {
        Walks { full: Walk::new(), local: Walk::new() }
    }

    pub(crate) fn restart(&mut self, module: bool) {
        self.full.reset(module);
        self.local.seed_lost = true;
    }
}

/// [after] on the full walk: yield / await need the enclosing functions.
pub(super) fn after_scoped(tokens: &Tokens, walks: &mut Walks, pos: usize) -> After {
    let w = &mut walks.full;
    if w.last_query.0 == pos {
        return w.last_query.1;
    }
    let inside_last = pos < w.walked_to && pos >= w.last_start;
    if w.walked_to > pos && !inside_last {
        w.reset(tokens.module);
    }
    w.advance(tokens, pos);
    // A query inside the last token (the tail of a fused >>>) gets the state after it.
    let a = if w.walked_to > pos { w.classify_after() } else { w.after_token(tokens, pos) };
    w.last_query = (pos, a);
    a
}
