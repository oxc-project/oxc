//! The walk: its frame stack and flags, the operations the token steps use on them, the
//! jump plan a bounded walk follows, and what a query reads off the state.

use super::*;

/// A jump a bounded walk takes after stepping the token at `at`.
#[derive(Clone, Copy)]
pub(super) enum Jump {
    /// `at` opens a balanced group whose closer is at `to`: the walk resumes at the closer.
    Skip { at: u32, to: u32 },
    /// `at` is a `<` balanced by the `>` at `to`: the walk resumes there if it read the `<` as a
    /// list opener (a comparison walks on).
    Angle { at: u32, to: u32 },
    /// `at` opens a frame (or is an anchor inside one) holding the last `;` and `,` at `semi` /
    /// `comma` and the last `}` boundary at `brace` (0: none) before the query: the walk resumes
    /// at the nearest one its frame kind allows. A brace boundary is the token after a `}` that
    /// must start a statement or member, so the frame is reset as a `;` would.
    Sep { at: u32, semi: u32, comma: u32, brace: u32 },
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
    #[inline]
    pub(super) fn operand_allowed(&self) -> bool {
        self.expect != Expect::Operator
    }

    /// May a statement start here?
    #[inline]
    pub(super) fn at_stmt_start(&self) -> bool {
        self.expect == Expect::Statement
    }
}

#[derive(Default)]
pub(super) struct Walk {
    /// Frame count right after a bounded walk started at its anchor (0: the full walk); the walk
    /// stays valid while that frame is on the stack.
    pub(super) seed_depth: usize,
    /// A bounded walk popped its anchor frame (or met an unbalanced closer): its state is a guess
    /// from here on.
    pub(super) seed_lost: bool,
    /// Jumps of the current bounded walk, in source order, and the next one to consider.
    pub(super) jumps: Vec<Jump>,
    pub(super) gts: Vec<u32>,
    pub(super) next_jump: usize,
    pub(super) frames: Vec<Frame>,
    /// Next unprocessed byte position: every token start below it has been walked.
    pub(super) walked_to: usize,
    /// What may come next.
    pub(super) expect: Expect,
    pub(super) prev_end: usize,
    /// The previous significant token was a numeric literal ending exactly at `prev_end` (so a `.`
    /// there continues the number).
    pub(super) prev_num: bool,
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
    /// Previous token closed a Params frame.
    pub(super) closed_params: bool,
    /// The previous significant token was `async` (same line as this one).
    pub(super) prev_async: bool,
    /// `export default` was just seen.
    pub(super) export_default: bool,
    /// Decorator at statement level / operand level (0 none).
    pub(super) decorator: u8,
    /// A closing JSX tag is being skipped until its tk!(JsxTagEnd).
    pub(super) jsx_closing: bool,
    /// Set by the last processed token when the statement it completed cannot be continued by
    /// anything (`break label`, module specifier).
    pub(super) stmt_done: bool,
    /// Last `after_token` query, so a site asking twice gets one answer.
    pub(super) last_query: Option<(usize, After)>,
    /// Start of the last processed token (a query inside it, e.g. at the last `>` of a fused `>>>`,
    /// reports the state after it).
    pub(super) last_start: usize,
    /// The previous token ended an `as` type or closed a type-argument list; TypeScript never tries
    /// type arguments after those, so a `<` here compares.
    pub(super) no_type_args: bool,
}

impl Walk {
    pub(super) fn new() -> Walk {
        Walk {
            jumps: Vec::with_capacity(64),
            gts: Vec::with_capacity(8),
            frames: Vec::with_capacity(64),
            ..Walk::default()
        }
    }

    pub(super) fn reset(&mut self, module: bool) {
        let frames = std::mem::take(&mut self.frames);
        let jumps = std::mem::take(&mut self.jumps);
        let gts = std::mem::take(&mut self.gts);
        *self = Walk { frames, jumps, gts, ..Walk::default() };
        self.frames.clear();
        self.frames.push(Frame {
            kind: FrameKind::Root,
            is_async: module,
            strict: module,
            prologue: true,
            ..Frame::default()
        });
    }

    #[inline]
    pub(super) fn top(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    #[inline]
    pub(super) fn top_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    #[inline]
    pub(super) fn top_kind(&self) -> FrameKind {
        self.top().kind
    }

    pub(super) fn push(&mut self, kind: FrameKind) -> &mut Frame {
        let f = self.top().child(kind);
        self.frames.push(f);
        self.frames.last_mut().unwrap()
    }

    pub(super) fn pop(&mut self) -> Frame {
        if self.frames.len() > 1 {
            let f = self.frames.pop().unwrap();
            if self.frames.len() < self.seed_depth {
                self.seed_lost = true;
            }
            f
        } else {
            *self.top()
        }
    }

    /// A closer with no frame to close: past here a bounded walk guesses.
    pub(super) fn unbalanced(&mut self) {
        if self.seed_depth != 0 {
            self.seed_lost = true;
        }
    }

    pub(super) fn unbalanced_close(&mut self) {
        self.unbalanced();
        self.after_statement();
        self.clear_prev();
    }

    /// Index of the innermost frame that owns statements / declarations.
    pub(super) fn stmt_frame(&self) -> usize {
        let mut i = self.frames.len() - 1;
        loop {
            if !self.frames[i].kind.is_virtual() {
                return i;
            }
            if i == 0 {
                return 0;
            }
            i -= 1;
        }
    }

    /// The frame whose declarator state (`let x`, `for (let x`) governs the next token, if the
    /// innermost real frame can hold one.
    pub(super) fn decl_frame(&self) -> Option<usize> {
        let i = self.stmt_frame();
        if self.frames[i].kind.is_stmt_holder() { Some(i) } else { None }
    }

    /// Statement register of the innermost statement holder (S_NONE inside expression frames).
    pub(super) fn stmt_reg(&self) -> u8 {
        let i = self.stmt_frame();
        if self.frames[i].kind.is_stmt_holder() { self.frames[i].reg } else { S_NONE }
    }

    pub(super) fn top_declarator(&self) -> u8 {
        let f = self.top();
        if f.kind.is_stmt_holder() { f.state } else { D_NONE }
    }

    pub(super) fn top_reg(&self) -> u8 {
        let f = self.top();
        if f.kind.is_stmt_holder() { f.reg } else { S_NONE }
    }

    pub(super) fn set_stmt_reg(&mut self, v: u8) {
        let i = self.stmt_frame();
        if self.frames[i].kind.is_stmt_holder() {
            self.frames[i].reg = v;
        }
    }

    /// Innermost function-like scope: where `yield` / `await` look up their keyword-ness.
    fn scope(&self) -> &Frame {
        let mut i = self.frames.len() - 1;
        loop {
            match self.frames[i].kind {
                FrameKind::Root
                | FrameKind::FnBody
                | FrameKind::ArrowBody
                | FrameKind::Concise
                | FrameKind::ClassBody
                | FrameKind::StaticBlock
                | FrameKind::Params => return &self.frames[i],
                _ => {}
            }
            if i == 0 {
                return &self.frames[0];
            }
            i -= 1;
        }
    }

    /// Is the top of the stack (ignoring nothing) a type context?
    pub(super) fn in_type(&self) -> bool {
        let k = self.top_kind();
        k == FrameKind::TypeRegion || k.is_type_group() || (k == FrameKind::Sub && self.top().decl)
    }

    /// The nearest TypeRegion above the nearest bracket frame, if any.
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
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }

    /// Pop concise arrow bodies sitting on top of the stack.
    pub(super) fn pop_concise(&mut self) {
        while self.top_kind() == FrameKind::Concise {
            self.pop();
        }
    }

    /// End every virtual frame above the nearest bracket frame (used by closers and separators).
    pub(super) fn pop_virtual(&mut self) {
        while self.top_kind().is_virtual() {
            self.pop();
        }
    }

    /// Pop through the nearest bracket frame when wanted accepts its kind; any other is a barrier.
    pub(super) fn pop_to(&mut self, wanted: impl Fn(FrameKind) -> bool) -> Option<Frame> {
        let mut i = self.frames.len();
        while i > 1 {
            i -= 1;
            let k = self.frames[i].kind;
            if wanted(k) {
                let f = self.frames[i];
                self.frames.truncate(i);
                if i < self.seed_depth {
                    self.seed_lost = true;
                }
                return Some(f);
            }
            if !k.is_virtual() {
                return None;
            }
        }
        None
    }

    pub(super) fn after_statement(&mut self) {
        self.expect = Expect::Statement;
        let i = self.stmt_frame();
        let f = &mut self.frames[i];
        f.state = D_NONE;
        f.reg = S_NONE;
        f.head = 0;
        f.open_questions = 0;
        self.export_default = false;
        self.decorator = 0;
    }

    /// Statement boundary reached (`;`, ASI, block end).
    pub(super) fn end_statement(&mut self) {
        // Bodiless signatures and open type regions end with the statement.
        self.pop_virtual();
        self.after_statement();
    }

    pub(super) fn set_value(&mut self) {
        self.expect = Expect::Operator;
    }

    pub(super) fn set_operand(&mut self) {
        self.expect = Expect::Operand;
    }

    /// An operator or keyword after which an operand may start.
    pub(super) fn operand_done(&mut self) {
        self.set_operand();
        self.clear_prev();
    }

    /// A keyword that expects an operand, remembered for the next token.
    pub(super) fn keyword(&mut self, kw: u8) {
        self.operand_done();
        self.prev_kw = kw;
    }

    pub(super) fn advance(&mut self, tokens: &Tokens, limit: usize) {
        let mut pos = self.walked_to;
        loop {
            pos = tokens.next_start(pos);
            if pos >= limit || pos >= tokens.n {
                break;
            }
            let end = self.step(tokens, pos);
            pos = self.jump(pos, end, limit);
        }
        self.walked_to = pos.max(self.walked_to);
    }

    /// After stepping the token at `pos` (ending at `end`): the next position to walk, following a
    /// planned jump when the token opens a group or a frame that allows one. A jump lands on a
    /// closer or separator, so no line break is reported before it.
    fn jump(&mut self, pos: usize, end: usize, limit: usize) -> usize {
        while self.next_jump < self.jumps.len() {
            let j = self.jumps[self.next_jump];
            let at = match j {
                Jump::Skip { at, .. } | Jump::Angle { at, .. } | Jump::Sep { at, .. } => {
                    at as usize
                }
            };
            if at > pos {
                break;
            }
            self.next_jump += 1;
            if at < pos {
                continue;
            }
            let to = match j {
                Jump::Skip { to, .. } => to as usize,
                Jump::Angle { to, .. } => {
                    if self.top_kind() == FrameKind::Angle {
                        to as usize
                    } else {
                        0
                    }
                }
                Jump::Sep { semi, comma, brace, .. } => {
                    let semi = if semi != 0 && self.sep_allowed(b';') { semi as usize } else { 0 };
                    let comma =
                        if comma != 0 && self.sep_allowed(b',') { comma as usize } else { 0 };
                    let brace =
                        if brace != 0 && self.sep_allowed(b'}') { brace as usize } else { 0 };
                    let to = semi.max(comma).max(brace);
                    if to == brace && to > pos && to <= limit {
                        self.resume_member();
                    }
                    to
                }
            };
            if to > pos && to <= limit {
                self.prev_end = to;
                return to;
            }
        }
        end
    }

    /// A continued walk: jump to the nearest of the `;` at `semi` and the brace boundary at
    /// `brace` (0: none) before `limit` that its innermost bracket frame allows.
    pub(super) fn resume(&mut self, semi: usize, brace: usize, limit: usize) {
        let k = self.frames[self.stmt_frame()].kind;
        let semi = if semi != 0 && Self::sep_allowed_in(k, b';') { semi } else { 0 };
        let brace = if brace != 0 && Self::sep_allowed_in(k, b'}') { brace } else { 0 };
        let to = semi.max(brace);
        if to <= self.walked_to || to > limit {
            return;
        }
        self.pop_virtual();
        if to == brace {
            self.resume_member();
        }
        self.walked_to = to;
        self.prev_end = to;
        while self.next_jump < self.jumps.len() {
            let at = match self.jumps[self.next_jump] {
                Jump::Skip { at, .. } | Jump::Angle { at, .. } | Jump::Sep { at, .. } => {
                    at as usize
                }
            };
            if at >= to {
                break;
            }
            self.next_jump += 1;
        }
    }

    /// The state at a member or statement start after a `}` in the innermost frame: what its `;`
    /// would leave.
    fn resume_member(&mut self) {
        if self.top_kind() == FrameKind::ClassBody {
            self.top_mut().next_member();
            self.operand_done();
        } else {
            self.end_statement();
            self.clear_prev();
        }
    }

    /// Can the walk resume at a `;` / `,` of the innermost frame, or after a `}` boundary in it?
    /// Only where the separator resets the frame: statements after `;`, members and elements after
    /// `,`, statements and members after a body or a nested literal.
    fn sep_allowed(&self, sep: u8) -> bool {
        Self::sep_allowed_in(self.top_kind(), sep)
    }

    fn sep_allowed_in(k: FrameKind, sep: u8) -> bool {
        if sep == b'}' {
            return k.is_stmt_holder() || matches!(k, FrameKind::ClassBody | FrameKind::TypeLit);
        }
        if sep == b';' {
            k.is_stmt_holder()
                || matches!(k, FrameKind::ClassBody | FrameKind::TypeLit | FrameKind::Head)
        } else {
            matches!(
                k,
                FrameKind::Object
                    | FrameKind::Array
                    | FrameKind::Call
                    | FrameKind::Params
                    | FrameKind::Group
                    | FrameKind::TypeLit
                    | FrameKind::EnumBody
                    | FrameKind::ModuleSpec
                    | FrameKind::Pattern
                    | FrameKind::ArrayPattern
                    | FrameKind::Index
                    | FrameKind::TypeParen
                    | FrameKind::TypeBracket
                    | FrameKind::Head
                    | FrameKind::Sub
            )
        }
    }

    /// Process the single token at `pos` (which must be the next unprocessed token) and report what
    /// it leaves behind.
    pub(super) fn after_token(&mut self, tokens: &Tokens, pos: usize) -> After {
        debug_assert!(pos >= self.walked_to);
        let end = self.step(tokens, pos);
        self.walked_to = end.max(self.walked_to);
        self.classify_after()
    }

    /// The state left behind by the last processed token.
    pub(super) fn classify_after(&self) -> After {
        if self.stmt_done {
            return After::EndsDecl;
        }
        // A declaration-type region whose last token completed a type.
        if let Some(i) = self.region_index() {
            let r = &self.frames[i];
            if r.decl && r.atom && matches!(r.state, R_INLINE | R_STMT | R_INTERFACE) {
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
        if self.top_kind() == FrameKind::FnHead && self.closed_params {
            return After::EndsDecl;
        }
        // `let x` with nothing after the binding.
        if self.top_declarator() == D_BOUND {
            return After::EndsDecl;
        }
        if self.operand_allowed() { After::Operand } else { After::Value }
    }

    /// Is the identifier `yield` at the (unprocessed) token `pos` a keyword?
    pub(super) fn yield_is_keyword(&self) -> bool {
        let s = self.scope();
        !s.field_init() && (s.is_generator || s.strict || s.reserved)
    }

    pub(super) fn await_is_keyword(&self) -> bool {
        let s = self.scope();
        !s.field_init() && (s.is_async || s.reserved)
    }

    pub(super) fn site(&self) -> Site {
        let in_type = self.in_type();
        Site {
            in_type,
            operand: self.operand_allowed() && !in_type && !self.stmt_done,
            type_params: self.type_params_expected(),
            angles: self.open_angles(),
        }
    }

    /// Would a `<` at the next token open a type-parameter list of a declaration head or member?
    pub(super) fn type_params_expected(&self) -> bool {
        match self.top_kind() {
            FrameKind::FnHead | FrameKind::ClassHead => true,
            FrameKind::Object | FrameKind::ClassBody => self.top().state == M_KEY_SEEN,
            _ => self.stmt_reg() == S_TYPE_NAME,
        }
    }

    /// Consecutive open `<` lists at the top of the stack.
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

    pub(super) fn value_done(&mut self) {
        self.set_value();
        self.clear_prev();
        // The first value in a `for (` head is its binding.
        if self.top_kind() == FrameKind::Head && self.top().state == F_START {
            self.top_mut().state = F_BOUND;
        }
    }

    pub(super) fn clear_prev(&mut self) {
        self.no_type_args = false;
        self.after_dot = false;
        self.prev_kw = 0;
        self.prev_arrow = false;
        self.arrow_async = false;
        self.closed_group = false;
        self.closed_params = false;
        self.prev_async = false;
        self.prev_num = false;
    }

    pub(super) fn type_atom(&mut self, inner: bool) {
        if let Some(i) = self.region_index() {
            let r = &mut self.frames[i];
            r.atom = true;
            r.inner = inner;
        }
        self.set_value();
        self.clear_prev();
    }

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
    pub(super) fn member_done(&mut self) {
        if matches!(self.top_kind(), FrameKind::ClassBody | FrameKind::Object) {
            let f = self.top_mut();
            if f.state == M_KEY_SEEN {
                f.next_member();
            }
        }
    }
}
