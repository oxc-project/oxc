//! Bounded walks: the scan back from a query to an anchor, the jump plan it records for the
//! walk, and the entry points every question goes through.

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use super::anchor::{Anchor, anchor_at, paren_anchor};
use super::*;
use crate::pipeline::disambiguate::{LOCAL_WALK_BUDGET, WALK_SCAN_CAP};

/// Scan back to the anchor of a query, recording the groups the walk skips; None past the cap.
fn scan(tokens: &Tokens, jumps: &mut Vec<Jump>, from: usize, steps: &mut u32) -> Option<Anchor> {
    jumps.clear();
    let mut q = tokens.prev_sig(from);
    loop {
        let Some(p) = q else {
            return Some(Anchor::Stmt(0));
        };
        *steps += 1;
        if *steps > WALK_SCAN_CAP {
            return None;
        }
        let k = tokens.base_kind(p);
        if k >= OP_KIND_BASE {
            match tokens.src[p] {
                b')' | b']' | b'}' => {
                    let o = tokens.match_delim_back(p)?;
                    jumps.push(Jump { at: o as u32, to: p as u32 });
                    q = tokens.prev_sig(o);
                    continue;
                }
                b'(' => {
                    if let Some(anchor) = paren_anchor(tokens, p) {
                        return Some(anchor);
                    }
                    // A paren that may sit in a type: the walk reaches it from further back.
                }
                _ => {}
            }
        } else if k == tk!(Ident) {
            if let (_, Some(anchor)) = anchor_at(tokens, p) {
                return Some(anchor);
            }
        } else if matches_tk!(k, TemplateTail | TemplateMiddle) {
            // A template: skip back to its head.
            let head = tokens.template_head(p, steps, WALK_SCAN_CAP)?;
            jumps.push(Jump { at: head as u32, to: p as u32 });
            q = tokens.prev_sig(head);
            continue;
        }
        q = tokens.prev_sig(p);
    }
}

/// Read a bounded walk scanned back from from and walked to to; None if the full walk must answer.
fn local_walk<R>(
    tokens: &Tokens,
    walks: &mut Walks,
    from: usize,
    to: usize,
    read: impl FnOnce(&Walk) -> R,
) -> Option<R> {
    if !walks.shortcuts() || walks.spent > LOCAL_WALK_BUDGET as usize + to / 32 {
        return None;
    }
    let mut steps = 0;
    let anchor = scan(tokens, &mut walks.local.jumps, from, &mut steps);
    walks.spent += steps as usize;
    let w = &mut walks.local;
    w.start(tokens, anchor?);
    w.advance(tokens, to);
    (!w.seed_lost).then(|| read(w))
}

/// Context at the token starting at `pos` (not through it).
pub(crate) fn before(tokens: &Tokens, walks: &mut Walks, pos: usize) -> Site {
    if let Some(s) = local_walk(tokens, walks, pos, pos, Walk::site) {
        return s;
    }
    walks.full_to(tokens, pos).site()
}

/// Open < lists a > run at pos would close.
pub(crate) fn angles_before(tokens: &Tokens, walks: &mut Walks, pos: usize) -> usize {
    if let Some(n) = local_walk(tokens, walks, pos, pos, Walk::open_angles) {
        return n;
    }
    walks.full_to(tokens, pos).open_angles()
}

/// What the significant token at `pos` leaves behind.
pub(crate) fn after(tokens: &Tokens, walks: &mut Walks, pos: usize) -> After {
    // The scan takes in the token, skipping a closer's group whole; a word here is never an anchor.
    let from = if tokens.base_kind(pos) == tk!(Ident) { pos } else { pos + 1 };
    if let Some(a) = local_walk(tokens, walks, from, pos + 1, Walk::classify_after) {
        return a;
    }
    walks.full_to(tokens, pos).after_token(tokens, pos)
}

impl Walk {
    /// Seed the walk at the anchor, putting the scan's jumps (nearest first) in source order.
    fn start(&mut self, tokens: &Tokens, anchor: Anchor) {
        self.jumps.reverse();
        let (Anchor::Stmt(at) | Anchor::Expr(at)) = anchor;
        self.reset(tokens.module);
        self.walked_to = at;
        self.prev_end = at;
        // An operand anchor is certain only inside the construct it opens.
        let expr = matches!(anchor, Anchor::Expr(_));
        if expr {
            self.expect = Expect::Operand;
        }
        self.seed_depth = self.frames.len() + usize::from(expr);
    }
}
