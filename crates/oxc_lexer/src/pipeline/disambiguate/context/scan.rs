//! Bounded walks: the scan back from a query to an anchor, the jump plan it records for the
//! walk, and the entry points every question goes through.

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use super::anchor::{Anchor, anchor_at, paren_anchor};
use super::*;
use crate::pipeline::disambiguate::{LOCAL_WALK_BUDGET, WALK_SCAN_CAP};

struct Scan {
    anchor: Anchor,
    /// Unmatched `<` on the query's own level: the type lists a `>` run there could close.
    angles: u32,
    steps: u32,
}

/// Scan back to the anchor of a query, recording the groups the walk skips; None past the cap.
fn scan(tokens: &Tokens, jumps: &mut Vec<Jump>, from: usize) -> Option<Scan> {
    jumps.clear();
    let mut steps = 0u32;
    // Brackets opened (going back) between the query and the position scanned.
    let mut level = 0u32;
    // Closers seen on the level scanned that still wait for their opener.
    let mut pending = 0u32;
    // Unmatched `<` found on the query's own level.
    let mut angles = 0u32;
    let mut q = tokens.prev_sig(from);
    loop {
        let Some(p) = q else {
            return Some(Scan { anchor: Anchor::Stmt(0), angles, steps });
        };
        steps += 1;
        if steps > WALK_SCAN_CAP {
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
                        return Some(Scan { anchor, angles, steps });
                    }
                    // A paren that may sit in a type: the walk reaches it from further back.
                    level += 1;
                    pending = 0;
                }
                b'[' | b'{' => {
                    level += 1;
                    pending = 0;
                }
                b'>' if !(p > 0 && tokens.src[p - 1] == b'=') => {
                    pending += tokens.run_len(p, b'>') as u32;
                }
                b'<' if tokens.src[p + 1] != b'=' => {
                    for _ in 0..tokens.run_len(p, b'<') {
                        if pending > 0 {
                            pending -= 1;
                        } else if level == 0 {
                            angles += 1;
                        }
                    }
                }
                _ => {}
            }
        } else if k == tk!(Ident) {
            if let (_, Some(anchor)) = anchor_at(tokens, p) {
                return Some(Scan { anchor, angles, steps });
            }
        } else if matches_tk!(k, TemplateTail | TemplateMiddle) {
            // A template: skip back to its head. A middle also opens the substitution the query is
            // in, so the level changes there.
            let head = tokens.template_head(p, &mut steps, WALK_SCAN_CAP)?;
            if k == tk!(TemplateMiddle) {
                level += 1;
                pending = 0;
            }
            jumps.push(Jump { at: head as u32, to: p as u32 });
            q = tokens.prev_sig(head);
            continue;
        } else if k == tk!(TemplateHead) {
            // The substitution the query is in.
            level += 1;
            pending = 0;
        }
        q = tokens.prev_sig(p);
    }
}

/// The scan for a bounded walk to the token at pos, or None once the full walk must answer.
fn local_scan(tokens: &Tokens, walks: &mut Walks, pos: usize, from: usize) -> Option<Scan> {
    if !walks.shortcuts() || walks.spent > LOCAL_WALK_BUDGET as usize + pos / 32 {
        return None;
    }
    let s = scan(tokens, &mut walks.local.jumps, from);
    walks.spent += s.as_ref().map_or(WALK_SCAN_CAP, |s| s.steps) as usize;
    s
}

/// Run `f` on a bounded walk for the token at `pos`, scanning back from `from` (the matching
/// opener of a closer at `pos`, else `pos`). None when no anchor lies within the scan cap or the
/// walk lost its footing.
fn local_walk<R>(
    tokens: &Tokens,
    walks: &mut Walks,
    pos: usize,
    from: usize,
    f: impl FnOnce(&mut Walk, &Tokens) -> R,
) -> Option<R> {
    let s = local_scan(tokens, walks, pos, from)?;
    let w = &mut walks.local;
    w.start(tokens, s.anchor, pos, from);
    w.advance(tokens, pos);
    let r = f(w, tokens);
    (!w.seed_lost).then_some(r)
}

/// Context at the token starting at `pos` (not through it).
pub(crate) fn before(tokens: &Tokens, walks: &mut Walks, pos: usize) -> Site {
    if let Some(s) = local_walk(tokens, walks, pos, pos, |w, _| w.site()) {
        return s;
    }
    walks.full_to(tokens, pos).site()
}

/// Open `<` lists a `>` run at `pos` would close. When the scan back to the anchor meets no open
/// `<` on the run's level there is nothing to close, and no walk is needed.
pub(crate) fn angles_before(tokens: &Tokens, walks: &mut Walks, pos: usize) -> usize {
    if let Some(s) = local_scan(tokens, walks, pos, pos) {
        if s.angles == 0 {
            return 0;
        }
        let w = &mut walks.local;
        w.start(tokens, s.anchor, pos, pos);
        w.advance(tokens, pos);
        if !w.seed_lost {
            return w.open_angles();
        }
    }
    walks.full_to(tokens, pos).open_angles()
}

/// What the significant token at `pos` leaves behind.
pub(crate) fn after(tokens: &Tokens, walks: &mut Walks, pos: usize) -> After {
    // A closer or template tail: the bounded walk starts at its opener and skips the group.
    let k = tokens.base_kind(pos);
    let from = if k >= OP_KIND_BASE && matches!(tokens.src[pos], b')' | b']' | b'}') {
        tokens.match_delim_back(pos)
    } else if k == tk!(TemplateTail) {
        tokens.template_head(pos, &mut 0, WALK_SCAN_CAP)
    } else {
        None
    };
    let from = from.unwrap_or(pos);
    if let Some(a) = local_walk(tokens, walks, pos, from, |w, tokens| w.after_token(tokens, pos)) {
        return a;
    }
    walks.full_to(tokens, pos).after_token(tokens, pos)
}

impl Walk {
    /// Seed the walk at `anchor` for a query at `pos` whose closer group opens at `from`, with the
    /// scan's jumps in `jumps` (nearest first).
    fn start(&mut self, tokens: &Tokens, anchor: Anchor, pos: usize, from: usize) {
        self.jumps.reverse();
        if from < pos {
            self.jumps.push(Jump { at: from as u32, to: pos as u32 });
        }
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
        self.seed_at = at;
        self.seed_lost = false;
    }
}
