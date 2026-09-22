use crate::token::{OP_KIND_BASE, tk};

use super::anchor::{Anchor, anchor_at, brace_boundary, paren_anchor};
use super::*;
use crate::pipeline::disambiguate::WALK_SCAN_CAP;

pub(super) struct Scan {
    pub(super) anchor: Anchor,
    /// Unmatched < on the query's own level: the type lists a > run there could close.
    pub(super) angles: u32,
}

/// Per level: the last ;, , and } boundary, and the > closers waiting for a <.
#[derive(Default)]
struct Level {
    semi: Option<usize>,
    comma: Option<usize>,
    brace: Option<usize>,
    closers: Vec<u32>,
}

impl Level {
    fn enter(&mut self) {
        self.semi = None;
        self.comma = None;
        self.brace = None;
        self.closers.clear();
    }

    fn inside_list(&mut self) {
        self.semi = None;
        self.comma = None;
        self.brace = None;
    }

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

    fn record_resume(&self, w: &mut Walk) {
        if self.semi.is_some() || self.brace.is_some() {
            w.jumps.push(Jump::Resume {
                semi: self.semi.map_or(0, |s| s as u32),
                brace: self.brace.map_or(0, |b| b as u32),
            });
        }
    }
}

/// cont: where the current bounded walk stopped; reaching it means the walk continues.
pub(super) fn scan(
    tokens: &Tokens,
    w: &mut Walk,
    from: usize,
    cont: Option<usize>,
) -> Option<Scan> {
    w.jumps.clear();
    let mut steps = 0u32;
    // Brackets opened (going back) between the query and the position scanned.
    let mut level = 0u32;
    let mut lv = Level { closers: Vec::with_capacity(8), ..Level::default() };
    // Unmatched < found on the query's own level.
    let mut unmatched_lt = 0u32;
    let mut q = tokens.prev_sig(from);
    loop {
        let Some(p) = q else {
            lv.record(w, 0);
            return Some(Scan { anchor: Anchor::Stmt(0), angles: unmatched_lt });
        };
        if cont.is_some_and(|c| p < c) {
            lv.record_resume(w);
            return Some(Scan { anchor: Anchor::Continue, angles: unmatched_lt });
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
                        lv.record_resume(w);
                        return Some(Scan { anchor: Anchor::Continue, angles: unmatched_lt });
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
                // A separator inside a balanced <...> before the query is not the level's.
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
                    Anchor::Continue => p,
                };
                lv.record(w, at);
                return Some(Scan { anchor, angles: unmatched_lt });
            }
            // A , after class / interface may be in its heritage list, not the frame's.
            if matches!(kw, K_CLASS | K_INTERFACE) && !tokens.property_name(p) {
                lv.comma = None;
                lv.brace = None;
            }
        } else if k == tk!(TemplateTail) || k == tk!(TemplateMiddle) {
            // Skip back to the template head; a middle also opens the substitution the query is in.
            let mut depth = 1i32;
            let mut t = tokens.prev_sig(p);
            let head = loop {
                let Some(tp) = t else {
                    return None;
                };
                steps += 1;
                if steps > WALK_SCAN_CAP {
                    return None;
                }
                match tokens.base_kind(tp) {
                    tk!(TemplateTail) => depth += 1,
                    tk!(TemplateHead) => {
                        depth -= 1;
                        if depth == 0 {
                            break tp;
                        }
                    }
                    _ => {}
                }
                t = tokens.prev_sig(tp);
            };
            if cont.is_some_and(|c| head < c) {
                lv.record_resume(w);
                return Some(Scan { anchor: Anchor::Continue, angles: unmatched_lt });
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

pub(super) fn local_walk<R>(
    tokens: &Tokens,
    walks: &mut Walks,
    pos: usize,
    from: usize,
    f: impl FnOnce(&mut Walk, &Tokens) -> R,
) -> Option<R> {
    let w = &mut walks.local;
    let cont = (w.seed_depth != 0 && !w.seed_lost && w.walked_to <= from).then_some(w.walked_to);
    let s = scan(tokens, w, from, cont)?;
    w.start(tokens.module, s.anchor, pos, from);
    w.advance(tokens, pos);
    if w.seed_lost {
        return None;
    }
    Some(f(w, tokens))
}

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

pub(crate) fn angles_before(tokens: &Tokens, walks: &mut Walks, pos: usize) -> usize {
    let local = {
        let w = &mut walks.local;
        let cont = (w.seed_depth != 0 && !w.seed_lost && w.walked_to <= pos).then_some(w.walked_to);
        match scan(tokens, w, pos, cont) {
            None => None,
            Some(s) if s.angles == 0 && s.anchor != Anchor::Continue => Some(0),
            Some(s) => {
                w.start(tokens.module, s.anchor, pos, pos);
                w.advance(tokens, pos);
                if w.seed_lost { None } else { Some(w.open_angles()) }
            }
        }
    };
    local.unwrap_or_else(|| before(tokens, walks, pos).angles)
}

pub(crate) fn after(tokens: &Tokens, walks: &mut Walks, pos: usize) -> After {
    after_from(tokens, walks, pos, pos)
}

pub(crate) fn after_from(tokens: &Tokens, walks: &mut Walks, pos: usize, from: usize) -> After {
    if let Some(a) = local_walk(tokens, walks, pos, from, |w, tokens| {
        if w.walked_to > pos { w.classify_after() } else { w.after_token(tokens, pos) }
    }) {
        return a;
    }
    after_scoped(tokens, walks, pos)
}

impl Walk {
    pub(super) fn start(&mut self, module: bool, anchor: Anchor, pos: usize, from: usize) {
        self.jumps.reverse();
        if from < pos {
            self.jumps.push(Jump::Skip { at: from as u32, to: pos as u32 });
        }
        self.next_jump = 0;
        match anchor {
            Anchor::Continue => {
                if let Some(Jump::Resume { semi, brace }) = self.jumps.first().copied() {
                    self.jumps.remove(0);
                    self.resume(semi as usize, brace as usize, pos);
                }
            }
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
