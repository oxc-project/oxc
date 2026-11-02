//! The walk: its frame stack and flags, the operations the token steps use on them, the
//! jump plan a bounded walk follows, and what a query reads off the state.

use crate::cpu::cpu_version;

use super::*;

/// A balanced group a bounded walk skips, from its opener to its closer.
#[derive(Clone, Copy)]
pub(super) struct Jump {
    pub(super) at: u32,
    pub(super) to: u32,
}

/// What may come next at the walk's position.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) enum Expect {
    /// A statement may start here, so an operand may too.
    #[default]
    Statement,
    /// An operand may start here, a statement may not: after an operator, `(`, `=`, `return`.
    Operand,
    /// A value just ended: an operator may follow, an operand may not.
    Operator,
}

impl Walk {
    /// May an operand start here?
    #[cpu_version(Generic)]
    #[inline]
    pub(super) fn operand_allowed(&self) -> bool {
        self.expect != Expect::Operator
    }

    /// May a statement start here?
    #[cpu_version(Generic)]
    #[inline]
    pub(super) fn at_stmt_start(&self) -> bool {
        self.expect == Expect::Statement
    }
}

#[derive(Default)]
pub(super) struct Walk {
    /// Frame count right after a bounded walk started at its anchor (0: the full walk); the walk
    /// stays valid while that frame is on the stack.
    /// An operand anchor counts one more: the construct it opens, past which the walk would guess.
    pub(super) seed_depth: usize,
    /// A bounded walk popped its anchor frame (or met an unbalanced closer): its state is a guess
    /// from here on.
    pub(super) seed_lost: bool,
    /// Jumps of the current bounded walk, in source order, and the next one to consider.
    pub(super) jumps: Vec<Jump>,
    pub(super) next_jump: usize,
    pub(super) frames: Vec<Frame>,
    /// Next unprocessed byte position: every token start below it has been walked.
    pub(super) walked_to: usize,
    /// What may come next.
    pub(super) expect: Expect,
    pub(super) prev_end: usize,
    /// Previous token was `.` / `?.`: the next word is a property name.
    pub(super) after_dot: bool,
    /// Keyword code of the previous significant token, 0 if none.
    pub(super) prev_kw: u8,
    /// Previous token was `=>`, with the async-ness of the arrow.
    pub(super) prev_arrow: bool,
    pub(super) arrow_async: bool,
    /// Previous token closed a Group (`)`), and whether `async` preceded it.
    pub(super) closed_group: bool,
    pub(super) closed_group_async: bool,
    /// `export default` was just seen.
    pub(super) export_default: bool,
    /// Decorator at statement level / operand level (0 none).
    pub(super) decorator: u8,
    /// A closing JSX tag is being skipped until its tk!(JsxTagEnd).
    pub(super) jsx_closing: bool,
    /// Set by the last processed token when the statement it completed cannot be continued by
    /// anything (`break label`, module specifier).
    pub(super) stmt_done: bool,
    /// Start of the last processed token (a query inside it, e.g. at the last `>` of a fused `>>>`,
    /// reports the state after it).
    pub(super) last_start: usize,
    /// The previous token ended an `as` type or closed a type-argument list; TypeScript never tries
    /// type arguments after those, so a `<` here compares.
    pub(super) no_type_args: bool,
}

impl Walk {
    #[cpu_version(Generic)]
    pub(super) fn new() -> Walk {
        Walk { jumps: Vec::with_capacity(64), frames: Vec::with_capacity(64), ..Walk::default() }
    }

    #[cpu_multiversion]
    pub(super) fn reset(&mut self, module: bool, _cpu: impl Cpu) {
        let frames = std::mem::take(&mut self.frames);
        let jumps = std::mem::take(&mut self.jumps);
        *self = Walk { frames, jumps, ..Walk::default() };
        self.frames.clear();
        self.frames.push(Frame { kind: FrameKind::Root, is_async: module, ..Frame::default() });
    }

    #[cpu_version(Generic)]
    #[inline]
    pub(super) fn top(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    #[cpu_version(Generic)]
    #[inline]
    pub(super) fn top_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    #[cpu_version(Generic)]
    #[inline]
    pub(super) fn top_kind(&self) -> FrameKind {
        self.top().kind
    }

    #[cpu_version(Generic)]
    pub(super) fn push(&mut self, kind: FrameKind) -> &mut Frame {
        self.frames.push(Frame { kind, ..Frame::default() });
        self.frames.last_mut().unwrap()
    }

    #[cpu_version(Generic)]
    #[inline(always)]
    pub(super) fn pop(&mut self) -> Frame {
        if self.frames.len() > 1 { self.frames.pop().unwrap() } else { *self.top() }
    }

    /// A closer with no frame to close: past here a bounded walk guesses.
    #[cpu_version(Generic)]
    pub(super) fn unbalanced(&mut self) {
        if self.seed_depth != 0 {
            self.seed_lost = true;
        }
    }

    #[cpu_version(Generic)]
    pub(super) fn unbalanced_close(&mut self) {
        self.unbalanced();
        self.after_statement();
        self.clear_prev();
    }

    /// Index of the innermost frame that owns statements / declarations.
    #[cpu_version(Generic)]
    pub(super) fn stmt_frame(&self) -> usize {
        let mut i = self.frames.len() - 1;
        while self.frames[i].kind.is_virtual() {
            i -= 1;
        }
        i
    }

    /// Statement register of the innermost statement holder (None inside expression frames).
    #[cpu_version(Generic)]
    pub(super) fn stmt_reg(&self) -> StmtReg {
        self.frames[self.stmt_frame()].reg
    }

    #[cpu_version(Generic)]
    pub(super) fn set_stmt_reg(&mut self, v: StmtReg) {
        let i = self.stmt_frame();
        if self.frames[i].kind.is_stmt_holder() {
            self.frames[i].reg = v;
        }
    }

    /// Innermost function-like scope: where `yield` / `await` look up their keyword-ness.
    #[cpu_version(Generic)]
    fn scope(&self) -> usize {
        let mut i = self.frames.len() - 1;
        loop {
            let f = &self.frames[i];
            match f.kind {
                FrameKind::Root
                | FrameKind::FnBody
                | FrameKind::ArrowBody
                | FrameKind::Concise
                | FrameKind::Params => return i,
                FrameKind::ClassBody if f.field_init() => return i,
                _ => i -= 1,
            }
        }
    }

    /// Is the top of the stack (ignoring nothing) a type context?
    #[cpu_version(Generic)]
    pub(super) fn in_type(&self) -> bool {
        let k = self.top_kind();
        k == FrameKind::TypeRegion || k.is_type_group() || (k == FrameKind::Sub && self.top().decl)
    }

    /// The nearest TypeRegion above the nearest bracket frame, if any.
    #[cpu_version(Generic)]
    pub(super) fn region_index(&self) -> Option<usize> {
        let mut i = self.frames.len() - 1;
        loop {
            let k = self.frames[i].kind;
            if k == FrameKind::TypeRegion {
                return Some(i);
            }
            if !k.is_virtual() {
                return None;
            }
            i -= 1;
        }
    }

    /// Pop concise arrow bodies sitting on top of the stack.
    #[cpu_version(Generic)]
    #[inline(always)]
    pub(super) fn pop_concise(&mut self) {
        while self.top_kind() == FrameKind::Concise {
            self.pop();
        }
    }

    /// End every virtual frame above the nearest bracket frame (used by closers and separators).
    #[cpu_version(Generic)]
    pub(super) fn pop_virtual(&mut self) {
        while self.top_kind().is_virtual() {
            self.pop();
        }
    }

    /// Pop through the nearest bracket frame when wanted accepts its kind; any other is a barrier.
    #[cpu_version(Generic)]
    pub(super) fn pop_to(&mut self, wanted: impl Fn(FrameKind) -> bool) -> Option<Frame> {
        let mut i = self.frames.len();
        while i > 1 {
            i -= 1;
            let k = self.frames[i].kind;
            if wanted(k) {
                let f = self.frames[i];
                self.frames.truncate(i);
                return Some(f);
            }
            if !k.is_virtual() {
                return None;
            }
        }
        None
    }

    #[cpu_version(Generic)]
    pub(super) fn after_statement(&mut self) {
        self.expect = Expect::Statement;
        let i = self.stmt_frame();
        let f = &mut self.frames[i];
        f.declarator = Declarator::None;
        f.reg = StmtReg::None;
        f.open_questions = 0;
        self.export_default = false;
        self.decorator = 0;
    }

    /// Statement boundary reached (`;`, ASI, block end).
    #[cpu_version(Generic)]
    pub(super) fn end_statement(&mut self) {
        // Bodiless signatures and open type regions end with the statement.
        self.pop_virtual();
        self.after_statement();
    }

    #[cpu_version(Generic)]
    pub(super) fn set_value(&mut self) {
        self.expect = Expect::Operator;
    }

    #[cpu_version(Generic)]
    pub(super) fn set_operand(&mut self) {
        self.expect = Expect::Operand;
    }

    /// An operator or keyword after which an operand may start.
    #[cpu_version(Generic)]
    pub(super) fn operand_done(&mut self) {
        self.set_operand();
        self.clear_prev();
    }

    /// A keyword that expects an operand, remembered for the next token.
    #[cpu_version(Generic)]
    pub(super) fn keyword(&mut self, kw: u8) {
        self.operand_done();
        self.prev_kw = kw;
    }

    #[cpu_version(Generic)]
    pub(super) fn advance(&mut self, tokens: &Tokens, limit: usize) {
        let cpu = BASELINE;

        let mut pos = self.walked_to;
        loop {
            pos = tokens.next_start(pos, cpu);
            if pos >= limit || pos >= tokens.n {
                break;
            }
            let end = self.step(tokens, pos);
            if self.frames.len() < self.seed_depth {
                self.seed_lost = true;
            }
            pos = self.jump(pos, end, limit);
        }
        self.walked_to = pos.max(self.walked_to);
    }

    /// After stepping the token at `pos` (ending at `end`): the next position to walk, following a
    /// planned jump when the token opens a group or a frame that allows one. A jump lands on a
    /// closer or separator, so no line break is reported before it.
    #[cpu_version(Generic)]
    fn jump(&mut self, pos: usize, end: usize, limit: usize) -> usize {
        while let Some(&Jump { at, to }) = self.jumps.get(self.next_jump) {
            let (at, to) = (at as usize, to as usize);
            if at > pos {
                break;
            }
            self.next_jump += 1;
            if at == pos && to <= limit {
                self.prev_end = to;
                return to;
            }
        }
        end
    }

    /// What the token at pos leaves behind: it is stepped unless it is the tail of the last one.
    #[cpu_version(Generic)]
    pub(super) fn after_token(&mut self, tokens: &Tokens, pos: usize) -> After {
        if pos >= self.walked_to {
            let end = self.step(tokens, pos);
            self.walked_to = end.max(self.walked_to);
        }
        self.classify_after()
    }

    /// The state left behind by the last processed token.
    #[cpu_version(Generic)]
    pub(super) fn classify_after(&self) -> After {
        if self.stmt_done {
            return After::EndsDecl;
        }
        // A declaration-type region whose last token completed a type.
        if let Some(i) = self.region_index() {
            let r = &self.frames[i];
            if r.atom && matches!(r.region, RegionRule::Inline | RegionRule::Stmt) {
                // Parameter annotations (`(a: T` then `/`) are never followed by a regex; only
                // statement-level regions matter.
                let below = if i > 0 { self.frames[i - 1].kind } else { FrameKind::Root };
                if !matches!(
                    below,
                    FrameKind::Params
                        | FrameKind::Group
                        | FrameKind::Call
                        | FrameKind::Angle
                        | FrameKind::TypeParen
                ) {
                    return After::EndsDecl;
                }
            }
        }
        // A bodiless function signature: `function f(a)` then a line break.
        if self.top_kind() == FrameKind::FnHead && !self.operand_allowed() {
            return After::EndsDecl;
        }
        // `let x` with nothing after the binding.
        if self.top().declarator == Declarator::Bound {
            return After::EndsDecl;
        }
        if self.operand_allowed() { After::Operand } else { After::Value }
    }

    /// Is yield or await a keyword here? A bounded walk that cannot tell loses its footing.
    #[cpu_version(Generic)]
    pub(super) fn scoped_keyword(&mut self, generator: bool) -> bool {
        let i = self.scope();
        if i == 0 && self.seed_depth != 0 {
            self.seed_lost = true;
        }
        let s = &self.frames[i];
        !s.field_init() && if generator { s.is_generator } else { s.is_async }
    }

    #[cpu_version(Generic)]
    pub(super) fn site(&self) -> Site {
        let in_type = self.in_type();
        Site {
            in_type,
            operand: self.operand_allowed() && !in_type && !self.stmt_done,
            type_params: self.type_params_expected(),
        }
    }

    /// Would a `<` at the next token open a type-parameter list of a declaration head or member?
    #[cpu_version(Generic)]
    pub(super) fn type_params_expected(&self) -> bool {
        match self.top_kind() {
            FrameKind::FnHead | FrameKind::ClassHead => true,
            FrameKind::Object | FrameKind::ClassBody => self.top().member == Member::KeySeen,
            _ => self.stmt_reg() == StmtReg::TypeName,
        }
    }

    /// Consecutive open `<` lists at the top of the stack.
    #[cpu_version(Generic)]
    pub(super) fn open_angles(&self) -> usize {
        let mut i = self.frames.len();
        let mut k = 0;
        while i > 1 {
            i -= 1;
            if self.frames[i].kind == FrameKind::Angle {
                k += 1;
            } else {
                break;
            }
        }
        k
    }

    #[cpu_version(Generic)]
    pub(super) fn value_done(&mut self) {
        self.set_value();
        self.clear_prev();
        let f = self.top_mut();
        if f.declarator == Declarator::Binding {
            f.declarator = Declarator::Bound;
        }
    }

    #[cpu_version(Generic)]
    pub(super) fn clear_prev(&mut self) {
        self.no_type_args = false;
        self.after_dot = false;
        self.prev_kw = 0;
        self.prev_arrow = false;
        self.arrow_async = false;
        self.closed_group = false;
    }

    #[cpu_version(Generic)]
    pub(super) fn type_atom(&mut self, inner: bool) {
        if let Some(i) = self.region_index() {
            let r = &mut self.frames[i];
            r.atom = true;
            r.inner = inner;
        }
        self.value_done();
        self.no_type_args = true;
    }

    #[cpu_version(Generic)]
    pub(super) fn type_operator(&mut self) {
        if let Some(i) = self.region_index() {
            let r = &mut self.frames[i];
            r.atom = false;
            r.inner = false;
        }
        self.operand_done();
    }

    /// After a method / accessor body or nested object closed inside a member container, the next
    /// member key may follow.
    #[cpu_version(Generic)]
    pub(super) fn member_done(&mut self) {
        if matches!(self.top_kind(), FrameKind::ClassBody | FrameKind::Object) {
            let f = self.top_mut();
            if f.member == Member::KeySeen {
                f.next_member();
            }
        }
    }
}
