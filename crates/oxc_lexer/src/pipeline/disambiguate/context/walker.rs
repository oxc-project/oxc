use super::*;

#[derive(Clone, Copy)]
pub(super) enum Jump {
    /// at opens a balanced group whose closer is at to: the walk resumes at the closer.
    Skip { at: u32, to: u32 },
    /// The walk resumes at to only if it read the < at at as a list opener.
    Angle { at: u32, to: u32 },
    /// The last ;, , and } boundary (0: none) before the query in the frame at opens.
    Sep { at: u32, semi: u32, comma: u32, brace: u32 },
    /// A continued walk's resume points, usable once its virtual frames are dropped.
    Resume { semi: u32, brace: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Expect {
    /// A statement may start here, so an operand may too.
    Statement,
    /// An operand may start here, a statement may not: after an operator, (, =, return.
    Operand,
    /// A value just ended: an operator may follow, an operand may not.
    Operator,
}

impl Walk {
    #[inline]
    pub(super) fn operand_allowed(&self) -> bool {
        self.expect != Expect::Operator
    }

    #[inline]
    pub(super) fn at_stmt_start(&self) -> bool {
        self.expect == Expect::Statement
    }
}

pub(super) struct Walk {
    /// Frame count at the anchor (0: the full walk); valid while that frame is on the stack.
    pub(super) seed_depth: usize,
    /// The anchor frame was popped or a closer was unbalanced: the state is a guess from here.
    pub(super) seed_lost: bool,
    pub(super) jumps: Vec<Jump>,
    pub(super) next_jump: usize,
    pub(super) frames: Vec<Frame>,
    /// Next unprocessed byte position: every token start below it has been walked.
    pub(super) walked_to: usize,
    pub(super) expect: Expect,
    pub(super) prev_end: usize,
    /// A numeric literal ended exactly at prev_end, so a . there continues it.
    pub(super) prev_num: bool,
    /// Previous token was . / ?.: the next word is a property name.
    pub(super) after_dot: bool,
    /// Keyword code of the previous significant token, 0 if none.
    pub(super) prev_kw: u8,
    /// Previous token was =>, with the async-ness of the arrow.
    pub(super) prev_arrow: bool,
    pub(super) arrow_async: bool,
    /// Previous token closed a Group ()), and whether async preceded it.
    pub(super) closed_group: bool,
    pub(super) closed_group_async: bool,
    pub(super) closed_params: bool,
    /// The previous significant token was async (same line as this one).
    pub(super) prev_async: bool,
    pub(super) export_default: bool,
    /// Decorator at statement level / operand level (0 none).
    pub(super) decorator: u8,
    pub(super) for_await: bool,
    /// A closing JSX tag is being skipped until its tk!(JsxTagEnd).
    pub(super) jsx_closing: bool,
    /// The last token completed a statement nothing can continue (break label, import "x").
    pub(super) stmt_done: bool,
    /// Last after_token query, so a site asking twice gets one answer.
    pub(super) last_query: (usize, After),
    /// Start of the last token; a query inside it (the last > of >>>) sees the state after it.
    pub(super) last_start: usize,
    /// After x as T or a type-argument list TypeScript never tries type arguments: < compares.
    pub(super) no_type_args: bool,
}

impl Walk {
    pub(super) fn new() -> Walk {
        Walk {
            seed_depth: 0,
            seed_lost: false,
            jumps: Vec::with_capacity(64),
            next_jump: 0,
            frames: Vec::with_capacity(64),
            walked_to: 0,
            expect: Expect::Statement,
            prev_end: 0,
            prev_num: false,
            after_dot: false,
            prev_kw: 0,
            prev_arrow: false,
            arrow_async: false,
            closed_group: false,
            closed_group_async: false,
            closed_params: false,
            prev_async: false,
            export_default: false,
            decorator: 0,
            for_await: false,
            jsx_closing: false,
            stmt_done: false,
            last_query: (usize::MAX, After::Operand),
            last_start: 0,
            no_type_args: false,
        }
    }

    pub(super) fn reset(&mut self, module: bool) {
        self.frames.clear();
        self.frames.push(Frame {
            kind: FrameKind::Root,
            is_generator: false,
            is_async: module,
            strict: module,
            reserved: false,
            is_value: false,
            decl: false,
            atom: false,
            inner: false,
            state: 0,
            reg: 0,
            mods: 0,
            decl_binding: false,
            head: 0,
            open_questions: 0,
            prologue: 1,
        });
        self.walked_to = 0;
        self.expect = Expect::Statement;
        self.prev_end = 0;
        self.prev_num = false;
        self.after_dot = false;
        self.prev_kw = 0;
        self.prev_arrow = false;
        self.arrow_async = false;
        self.closed_group = false;
        self.closed_group_async = false;
        self.closed_params = false;
        self.prev_async = false;
        self.export_default = false;
        self.decorator = 0;
        self.for_await = false;
        self.jsx_closing = false;
        self.stmt_done = false;
        self.last_query = (usize::MAX, After::Operand);
        self.last_start = 0;
        self.no_type_args = false;
        self.seed_depth = 0;
        self.seed_lost = false;
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

    pub(super) fn stmt_frame(&self) -> usize {
        let mut i = self.frames.len() - 1;
        loop {
            match self.frames[i].kind {
                FrameKind::Concise
                | FrameKind::TypeRegion
                | FrameKind::Angle
                | FrameKind::FnHead
                | FrameKind::ClassHead => {}
                _ => return i,
            }
            if i == 0 {
                return 0;
            }
            i -= 1;
        }
    }

    pub(super) fn decl_frame(&self) -> Option<usize> {
        let i = self.stmt_frame();
        if is_stmt_holder(self.frames[i].kind) { Some(i) } else { None }
    }

    pub(super) fn stmt_reg(&self) -> u8 {
        let i = self.stmt_frame();
        if is_stmt_holder(self.frames[i].kind) { self.frames[i].reg } else { S_NONE }
    }

    pub(super) fn set_stmt_reg(&mut self, v: u8) {
        let i = self.stmt_frame();
        if is_stmt_holder(self.frames[i].kind) {
            self.frames[i].reg = v;
        }
    }

    /// Innermost function-like scope: where yield / await look up their keyword-ness.
    pub(super) fn scope(&self) -> &Frame {
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

    pub(super) fn in_type(&self) -> bool {
        matches!(
            self.top_kind(),
            FrameKind::TypeRegion
                | FrameKind::Angle
                | FrameKind::TypeParen
                | FrameKind::TypeBracket
                | FrameKind::TypeLit
        ) || (self.top_kind() == FrameKind::Sub && self.top().decl)
    }

    pub(super) fn region_index(&self) -> Option<usize> {
        let mut i = self.frames.len() - 1;
        loop {
            match self.frames[i].kind {
                FrameKind::TypeRegion => return Some(i),
                FrameKind::Concise
                | FrameKind::Angle
                | FrameKind::FnHead
                | FrameKind::ClassHead => {}
                _ => return None,
            }
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }

    pub(super) fn pop_concise(&mut self) {
        while self.top_kind() == FrameKind::Concise {
            self.pop();
        }
    }

    pub(super) fn pop_virtual(&mut self) {
        while matches!(
            self.top_kind(),
            FrameKind::Concise
                | FrameKind::TypeRegion
                | FrameKind::Angle
                | FrameKind::FnHead
                | FrameKind::ClassHead
        ) {
            self.pop();
        }
    }

    /// Pops through the nearest bracket frame if its kind is in kinds; any other is a barrier.
    pub(super) fn pop_to(&mut self, kinds: &[FrameKind]) -> Option<Frame> {
        let mut i = self.frames.len();
        while i > 1 {
            i -= 1;
            let k = self.frames[i].kind;
            if kinds.contains(&k) {
                let f = self.frames[i];
                self.frames.truncate(i);
                if i < self.seed_depth {
                    self.seed_lost = true;
                }
                return Some(f);
            }
            if !matches!(
                k,
                FrameKind::Concise
                    | FrameKind::TypeRegion
                    | FrameKind::Angle
                    | FrameKind::FnHead
                    | FrameKind::ClassHead
            ) {
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

    pub(super) fn operand_done(&mut self) {
        self.set_operand();
        self.clear_prev();
    }

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

    pub(super) fn jump(&mut self, pos: usize, end: usize, limit: usize) -> usize {
        while self.next_jump < self.jumps.len() {
            let j = self.jumps[self.next_jump];
            let at = match j {
                Jump::Skip { at, .. } | Jump::Angle { at, .. } | Jump::Sep { at, .. } => {
                    at as usize
                }
                Jump::Resume { .. } => {
                    self.next_jump += 1;
                    continue;
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
                Jump::Resume { .. } => 0,
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

    pub(super) fn resume(&mut self, semi: usize, brace: usize, limit: usize) {
        let mut i = self.frames.len() - 1;
        while matches!(
            self.frames[i].kind,
            FrameKind::Concise
                | FrameKind::TypeRegion
                | FrameKind::Angle
                | FrameKind::FnHead
                | FrameKind::ClassHead
        ) && i > 0
        {
            i -= 1;
        }
        let k = self.frames[i].kind;
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
                Jump::Resume { .. } => 0,
            };
            if at >= to {
                break;
            }
            self.next_jump += 1;
        }
    }

    pub(super) fn resume_member(&mut self) {
        if self.top_kind() == FrameKind::ClassBody {
            let f = self.top_mut();
            f.state = M_KEY_POS;
            f.mods = 0;
            self.operand_done();
        } else {
            self.end_statement();
            self.clear_prev();
        }
    }

    /// Only where the separator resets the frame: ; for statements, , for members and elements.
    pub(super) fn sep_allowed(&self, sep: u8) -> bool {
        Self::sep_allowed_in(self.top_kind(), sep)
    }

    pub(super) fn sep_allowed_in(k: FrameKind, sep: u8) -> bool {
        if sep == b'}' {
            return matches!(
                k,
                FrameKind::Root
                    | FrameKind::Block
                    | FrameKind::FnBody
                    | FrameKind::ArrowBody
                    | FrameKind::StaticBlock
                    | FrameKind::ClassBody
                    | FrameKind::TypeLit
            );
        }
        if sep == b';' {
            matches!(
                k,
                FrameKind::Root
                    | FrameKind::Block
                    | FrameKind::FnBody
                    | FrameKind::ArrowBody
                    | FrameKind::StaticBlock
                    | FrameKind::ClassBody
                    | FrameKind::TypeLit
                    | FrameKind::Head
            )
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

    pub(super) fn after_token(&mut self, tokens: &Tokens, pos: usize) -> After {
        debug_assert!(pos >= self.walked_to);
        let end = self.step(tokens, pos);
        self.walked_to = end.max(self.walked_to);
        self.classify_after()
    }

    pub(super) fn classify_after(&self) -> After {
        if self.stmt_done {
            return After::EndsDecl;
        }
        // A declaration-type region whose last token completed a type.
        if let Some(i) = self.region_index() {
            let r = &self.frames[i];
            if r.decl && r.atom && matches!(r.state, R_INLINE | R_STMT | R_INTERFACE) {
                // Parameter annotations never precede a regex; only statement regions matter.
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
        // A bodiless function signature: function f(a) then a line break.
        if self.top_kind() == FrameKind::FnHead && self.closed_params {
            return After::EndsDecl;
        }
        // let x with nothing after the binding.
        if let Some(si) = self.decl_frame() {
            let sf = &self.frames[si];
            if sf.state == D_BOUND && si == self.frames.len() - 1 && sf.kind != FrameKind::Head {
                return After::EndsDecl;
            }
        }
        if self.operand_allowed() { After::Operand } else { After::Value }
    }

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

    pub(super) fn type_params_expected(&self) -> bool {
        match self.top_kind() {
            FrameKind::FnHead | FrameKind::ClassHead => true,
            FrameKind::Object | FrameKind::ClassBody => self.top().state == M_KEY_SEEN,
            _ => self.stmt_reg() == S_TYPE_NAME,
        }
    }

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
        // The first value in a for ( head is its binding.
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

    pub(super) fn type_atom(&mut self) {
        if let Some(i) = self.region_index() {
            let r = &mut self.frames[i];
            r.atom = true;
            r.inner = false;
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

    pub(super) fn member_done(&mut self) {
        if matches!(self.top_kind(), FrameKind::ClassBody | FrameKind::Object) {
            let f = self.top_mut();
            if f.state == M_KEY_SEEN {
                f.state = M_KEY_POS;
                f.mods = 0;
            }
        }
    }
}
