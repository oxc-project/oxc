//! Bounded walks: the scan back from a query to an anchor, the jump plan it records for the
//! walk, and the entry points every question goes through.
//!
//! A walk starts at an anchor: a token whose context is certain from its neighbours alone (see
//! [`anchor`]). The scan back to the anchor records the balanced groups it crosses and the last
//! separator inside each open bracket, so the walk skips group interiors and, once a bracket's
//! frame kind is known, resumes at the separator. Nothing is guessed: what the walk did not see
//! was either inside a skipped group, whose closer depends only on how it opened, or before a
//! separator, which resets the frame.
//!
//! [`anchor`]: super::anchor

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use super::anchor::{Anchor, anchor_at, brace_boundary, paren_anchor};
use super::*;
use crate::pipeline::disambiguate::WALK_SCAN_CAP;

struct Scan {
    pub(super) anchor: Anchor,
    /// Unmatched `<` on the query's own level: the type lists a `>` run there could close.
    pub(super) angles: u32,
}

/// What the scan has seen on the level it is in, nearest to the query first: the last `;`, `,`
/// and `}` boundary, where the walk may resume once the level's frame kind is known, and the `>`
/// closers waiting for their `<` (`u32::MAX`: one of a run, which the walk cannot jump to).
struct Level<'a> {
    semi: Option<usize>,
    comma: Option<usize>,
    brace: Option<usize>,
    closers: &'a mut Vec<u32>,
}

impl Level<'_> {
    /// Nothing seen yet on a level the scan enters through its opener.
    fn enter(&mut self) {
        self.inside_list();
        self.closers.clear();
    }

    /// An open `<` around everything seen so far on the level: the separators inside it are not
    /// the level's.
    fn inside_list(&mut self) {
        self.semi = None;
        self.comma = None;
        self.brace = None;
    }

    /// Record the separators and brace boundary found on the level for the opener (or anchor)
    /// at `at`.
    fn record(&self, w: &mut Walk, at: usize) {
        if self.semi.is_some() || self.comma.is_some() || self.brace.is_some() {
            w.jumps.push(Jump::Sep {
                at: at as u32,
                semi: self.semi.map_or(0, |s| s as u32),
                comma: self.comma.map_or(0, |c| c as u32),
                brace: self.brace.map_or(0, |b| b as u32),
            });
        }
    }

    /// The walk continues inside this level: the points where it may resume.
    fn resume_point(&self) -> Anchor {
        Anchor::Continue {
            semi: self.semi.map_or(0, |s| s as u32),
            brace: self.brace.map_or(0, |b| b as u32),
        }
    }
}

/// Scan back from `from` (exclusive) to the anchor of a query, recording the walk's jumps in
/// `w.jumps` (nearest first). `cont` is where the current bounded walk stopped: reaching it, or a
/// group holding it, means the walk continues from there. None past the scan cap.
fn scan(tokens: &Tokens, w: &mut Walk, from: usize, cont: Option<usize>) -> Option<Scan> {
    let mut gts = std::mem::take(&mut w.gts);
    gts.clear();
    let scan = scan_with(tokens, w, from, cont, &mut gts);
    w.gts = gts;
    scan
}

fn scan_with(
    tokens: &Tokens,
    w: &mut Walk,
    from: usize,
    cont: Option<usize>,
    gts: &mut Vec<u32>,
) -> Option<Scan> {
    w.jumps.clear();
    let mut steps = 0u32;
    // Brackets opened (going back) between the query and the position scanned.
    let mut level = 0u32;
    let mut lv = Level { semi: None, comma: None, brace: None, closers: gts };
    // Unmatched `<` found on the query's own level.
    let mut unmatched_lt = 0u32;
    let mut q = tokens.prev_sig(from);
    loop {
        let Some(p) = q else {
            lv.record(w, 0);
            return Some(Scan { anchor: Anchor::Stmt(0), angles: unmatched_lt });
        };
        if cont.is_some_and(|c| p < c) {
            return Some(Scan { anchor: lv.resume_point(), angles: unmatched_lt });
        }
        steps += 1;
        if steps > WALK_SCAN_CAP {
            return None;
        }
        let k = tokens.base_kind(p);
        if k >= OP_KIND_BASE {
            let c = tokens.src[p];
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    let o = tokens.match_delim_back(p, open, c)?;
                    if cont.is_some_and(|c| o < c) {
                        return Some(Scan { anchor: lv.resume_point(), angles: unmatched_lt });
                    }
                    if c == b'}' && lv.brace.is_none() && lv.closers.is_empty() {
                        lv.brace = brace_boundary(tokens, p);
                    }
                    w.jumps.push(Jump::Skip { at: o as u32, to: p as u32 });
                    q = tokens.prev_sig(o);
                    continue;
                }
                b'(' => {
                    lv.record(w, p);
                    if let Some(anchor) = paren_anchor(tokens, p) {
                        return Some(Scan { anchor, angles: unmatched_lt });
                    }
                    // A paren that may sit in a type: the walk reaches it from further back.
                    level += 1;
                    lv.enter();
                }
                b'[' | b'{' => {
                    lv.record(w, p);
                    level += 1;
                    lv.enter();
                }
                // A separator inside a balanced `<...>` before the query is not the level's.
                b';' => {
                    if lv.semi.is_none() && lv.closers.is_empty() {
                        lv.semi = Some(p);
                    }
                }
                b',' => {
                    if lv.comma.is_none() && lv.closers.is_empty() {
                        lv.comma = Some(p);
                    }
                }
                b'>' => {
                    if !(p > 0 && tokens.src[p - 1] == b'=') {
                        let n = tokens.run_len(p, b'>');
                        if n == 1 {
                            lv.closers.push(p as u32);
                        } else {
                            lv.closers.extend(core::iter::repeat_n(u32::MAX, n as usize));
                        }
                    }
                }
                b'<' if tokens.src[p + 1] != b'=' => {
                    let n = tokens.run_len(p, b'<');
                    for _ in 0..n {
                        match lv.closers.pop() {
                            Some(g) => {
                                if n == 1 && g != u32::MAX {
                                    w.jumps.push(Jump::Angle { at: p as u32, to: g });
                                }
                            }
                            None => {
                                if level == 0 {
                                    unmatched_lt += 1;
                                }
                                lv.inside_list();
                            }
                        }
                    }
                }
                _ => {}
            }
        } else if k == tk!(Ident) {
            let (kw, anchor) = anchor_at(tokens, p);
            if let Some(anchor) = anchor {
                let at = match anchor {
                    Anchor::Stmt(a) | Anchor::Expr(a) => a,
                    Anchor::Continue { .. } => p,
                };
                lv.record(w, at);
                return Some(Scan { anchor, angles: unmatched_lt });
            }
            // A `,` after a class or interface keyword may sit in its heritage list, which belongs
            // to the head, not to the frame around it.
            if matches_tk!(kw, KwClass | KwInterface) && !tokens.property_name(p) {
                lv.comma = None;
                lv.brace = None;
            }
        } else if matches_tk!(k, TemplateTail | TemplateMiddle) {
            // A template: skip back to its head. A middle also opens the substitution the query is
            // in, so the level changes there.
            let head = tokens.template_head(p, &mut steps, WALK_SCAN_CAP)?;
            if cont.is_some_and(|c| head < c) {
                return Some(Scan { anchor: lv.resume_point(), angles: unmatched_lt });
            }
            if k == tk!(TemplateMiddle) {
                lv.record(w, p);
                level += 1;
                lv.enter();
            }
            w.jumps.push(Jump::Skip { at: head as u32, to: p as u32 });
            q = tokens.prev_sig(head);
            continue;
        } else if k == tk!(TemplateHead) {
            // The substitution the query is in.
            lv.record(w, p);
            level += 1;
            lv.enter();
        }
        q = tokens.prev_sig(p);
    }
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
    let w = &mut walks.local;
    let s = w.local_scan(tokens, from)?;
    w.run_local(tokens, s, pos, from).then(|| f(w, tokens))
}

/// Context at the token starting at `pos` (not through it).
pub(crate) fn before(tokens: &Tokens, walks: &mut Walks, pos: usize) -> Site {
    if let Some(s) = local_walk(tokens, walks, pos, pos, |w, _| w.site()) {
        return s;
    }
    let w = &mut walks.full;
    if w.walked_to > pos {
        w.reset(tokens.module);
    }
    w.advance(tokens, pos);
    w.site()
}

/// Open `<` lists a `>` run at `pos` would close. When the scan back to the anchor meets no open
/// `<` on the run's level there is nothing to close, and no walk is needed.
pub(crate) fn angles_before(tokens: &Tokens, walks: &mut Walks, pos: usize) -> usize {
    let local = {
        let w = &mut walks.local;
        match w.local_scan(tokens, pos) {
            None => None,
            Some(s) if s.angles == 0 && !matches!(s.anchor, Anchor::Continue { .. }) => Some(0),
            Some(s) => w.run_local(tokens, s, pos, pos).then(|| w.open_angles()),
        }
    };
    local.unwrap_or_else(|| before(tokens, walks, pos).angles)
}

/// What the significant token at `pos` leaves behind.
pub(crate) fn after(tokens: &Tokens, walks: &mut Walks, pos: usize) -> After {
    after_from(tokens, walks, pos, pos)
}

/// Like [`after`], with the bounded walk scanning back from `from` (the matching opener of a
/// closer at `pos`): the walk reaches the opener, skips the group and steps the closer.
pub(crate) fn after_from(tokens: &Tokens, walks: &mut Walks, pos: usize, from: usize) -> After {
    if let Some(a) = local_walk(tokens, walks, pos, from, |w, tokens| {
        if w.walked_to > pos { w.classify_after() } else { w.after_token(tokens, pos) }
    }) {
        return a;
    }
    after_scoped(tokens, walks, pos)
}

impl Walk {
    fn local_scan(&mut self, tokens: &Tokens, from: usize) -> Option<Scan> {
        let cont = (self.seed_depth != 0 && !self.seed_lost && self.walked_to <= from)
            .then_some(self.walked_to);
        scan(tokens, self, from, cont)
    }

    fn run_local(&mut self, tokens: &Tokens, s: Scan, pos: usize, from: usize) -> bool {
        self.start(tokens.module, s.anchor, pos, from);
        self.advance(tokens, pos);
        !self.seed_lost
    }

    /// Seed the walk at `anchor` for a query at `pos` whose closer group opens at `from`, with the
    /// scan's jumps in `jumps` (nearest first).
    pub(super) fn start(&mut self, module: bool, anchor: Anchor, pos: usize, from: usize) {
        self.jumps.reverse();
        if from < pos {
            self.jumps.push(Jump::Skip { at: from as u32, to: pos as u32 });
        }
        self.next_jump = 0;
        match anchor {
            Anchor::Continue { semi, brace } => self.resume(semi as usize, brace as usize, pos),
            Anchor::Stmt(at) | Anchor::Expr(at) => {
                self.reset(module);
                self.walked_to = at;
                self.prev_end = at;
                if matches!(anchor, Anchor::Expr(_)) {
                    self.expect = Expect::Operand;
                }
                self.seed_depth = self.frames.len();
                self.seed_lost = false;
            }
        }
    }
}
