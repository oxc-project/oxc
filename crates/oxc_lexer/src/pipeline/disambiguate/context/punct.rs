//! Punctuation in expression context: operators, arrows, separators, and the brackets that
//! open and close frames.

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use crate::pipeline::operators::is_op_char;

use super::*;

impl Walk {
    pub(super) fn step_op(&mut self, tokens: &Tokens, pos: usize, newline: bool) -> usize {
        let c = tokens.src[pos];
        let mut len = if is_op_char(c) || c == b'/' { tokens.op_len(pos) } else { 1 };
        let c1 = tokens.src[pos + 1];

        // Type context: brackets and separators belong to the type.
        if self.in_type() {
            // `>` closes one Angle per byte; `<<` opens two.
            if c == b'>' && !(c1 == b'=' && self.top_kind() != FrameKind::Angle) {
                len = 1;
            }
            if c == b'<' && c1 == b'<' {
                len = 1;
            }
            return self.type_op(pos, c, len);
        }

        let postfix = !self.operand_allowed() && !newline;
        match c {
            b'{' => self.open_brace(),
            b'}' => self.close_brace(),
            b'(' => self.open_paren(),
            b')' => self.close_paren(),
            b'[' => self.open_bracket(),
            b']' => self.close_bracket(),
            b';' => self.semicolon(),
            b',' => self.comma(),
            b':' => self.colon(tokens),
            b'?' if len == 1 => self.question(tokens, pos),
            // Optional chaining or nullish coalescing.
            b'?' => {
                self.operand_done();
                self.after_dot = c1 == b'.';
            }
            // A spread: what follows is a value, not a member key.
            b'.' if len == 3 => {
                if self.top_kind() == FrameKind::Object {
                    self.top_mut().state = M_VALUE;
                }
                self.operand_done();
            }
            // The dot of a number such as 1. continues the literal.
            b'.' if tokens.numeric_dot(pos) => self.set_value(),
            b'.' => {
                self.operand_done();
                self.after_dot = true;
            }
            b'=' if len == 2 && c1 == b'>' => self.arrow(tokens, pos),
            b'=' if len == 1 => self.assign(),
            // A postfix non-null assertion, or a definite assignment.
            b'!' if len == 1 && tokens.ts && postfix => self.value_done(),
            // A postfix increment or decrement keeps the value.
            b'+' | b'-' if len == 2 && c1 == c && postfix => self.value_done(),
            b'*' if len == 1 && self.top_kind() == FrameKind::FnHead => {
                self.top_mut().is_generator = true;
                self.value_done();
                self.prev_kw = tk!(KwFunction);
            }
            b'*' if len == 1
                && matches!(self.top_kind(), FrameKind::Object | FrameKind::ClassBody)
                && self.top().state == M_KEY_POS =>
            {
                self.top_mut().mods |= MOD_GEN;
                self.operand_done();
            }
            // The star of yield*.
            b'*' if len == 1 && self.prev_kw == tk!(KwYield) => self.set_operand(),
            b'<' if len == 1 => self.less_than(tokens, pos),
            // Two openers, not a shift.
            b'<' if c1 == b'<'
                && len == 2
                && tokens.ts
                && !self.operand_allowed()
                && !self.no_type_args
                && lt_run_split(tokens, pos) =>
            {
                self.less_than(tokens, pos);
                return pos + 1;
            }
            b'@' => {
                if self.decorator == 0 {
                    // A decorator after export or export default decorates a declaration.
                    let decl = self.at_stmt_start()
                        || !self.operand_allowed()
                        || self.prev_kw == tk!(KwExport)
                        || self.export_default;
                    self.decorator = if decl { 1 } else { 2 };
                }
                self.operand_done();
            }
            // Every other operator expects an operand.
            _ => self.operand_done(),
        }
        pos + len
    }

    pub(super) fn arrow(&mut self, tokens: &Tokens, pos: usize) {
        let is_async = if self.closed_group { self.closed_group_async } else { self.arrow_async };
        // Concise body unless `{` follows.
        let nx = tokens.peek(pos + 2);
        let block = nx.kind >= OP_KIND_BASE && nx.byte == b'{';
        if !block {
            self.push(FrameKind::Concise).is_async = is_async;
        }
        self.operand_done();
        self.prev_arrow = true;
        self.arrow_async = is_async;
    }

    pub(super) fn assign(&mut self) {
        if matches!(self.stmt_reg(), S_TYPE_NAME | S_IMPORT) {
            // `type X =`: the alias type.
            self.set_stmt_reg(S_NONE);
            self.open_region(R_STMT);
            return;
        }
        if self.top_declarator() == D_BOUND {
            self.top_mut().state = D_INIT;
        }
        if self.top_kind() == FrameKind::ClassBody {
            self.top_mut().state = M_VALUE;
        }
        self.operand_done();
    }

    pub(super) fn question(&mut self, tokens: &Tokens, pos: usize) {
        // Optional marker (`a?: T`, `a?,`, `a?)`) vs conditional.
        let nx = tokens.peek(pos + 1);
        let member = self.top_kind() == FrameKind::ClassBody;
        let optional = nx.kind >= OP_KIND_BASE
            && (matches!(nx.byte, b':' | b',' | b')' | b']' | b'>')
                || (member && matches!(nx.byte, b'(' | b'<')));
        if optional && !self.operand_allowed() {
            // Keep the member / parameter state.
            self.clear_prev();
            return;
        }
        self.top_mut().open_questions += 1;
        self.operand_done();
    }

    fn colon(&mut self, tokens: &Tokens) {
        // A concise arrow body without a pending `?` of its own ends at a `:` (the `?` belongs to
        // the frame below).
        while self.top_kind() == FrameKind::Concise && self.top().open_questions == 0 {
            self.pop();
        }
        let top = self.top_kind();
        // Ternary with a pending `?` on this frame.
        if self.top().open_questions > 0 {
            self.top_mut().open_questions -= 1;
            self.operand_done();
            return;
        }
        match top {
            FrameKind::Object => {
                self.top_mut().state = M_VALUE;
                self.operand_done();
                return;
            }
            // A member, parameter or index signature annotation.
            FrameKind::ClassBody | FrameKind::Params | FrameKind::ComputedKey => {
                if tokens.ts {
                    self.open_region(R_INLINE);
                } else {
                    self.operand_done();
                }
                return;
            }
            FrameKind::Group | FrameKind::Call => {
                if tokens.ts && !self.operand_allowed() {
                    // Arrow parameter annotation.
                    self.open_region(R_INLINE);
                } else {
                    self.operand_done();
                }
                return;
            }
            FrameKind::FnHead => {
                // Return type.
                self.open_region(R_INLINE);
                return;
            }
            _ => {}
        }
        if self.stmt_reg() == S_CASE {
            self.after_statement();
            self.clear_prev();
            return;
        }
        if tokens.ts && self.top_declarator() == D_BOUND {
            // Declarator type annotation.
            self.open_region(R_INLINE);
            return;
        }
        if tokens.ts && self.closed_group && top != FrameKind::Head {
            // `(a): T =>` arrow return type; remember the group's `async`.
            let is_async = self.closed_group_async;
            self.open_region(R_ARROW_RET);
            self.top_mut().mods = if is_async { MOD_ASYNC } else { 0 };
            return;
        }
        if top.is_stmt_holder() {
            // Any other colon among statements ends a label.
            self.after_statement();
            self.clear_prev();
            return;
        }
        self.operand_done();
    }

    pub(super) fn semicolon(&mut self) {
        self.pop_concise();
        match self.top_kind() {
            FrameKind::Head => {
                let f = self.top_mut();
                f.state = F_NO_OF;
                f.open_questions = 0;
                let si = self.stmt_frame();
                self.frames[si].state = D_NONE;
                self.operand_done();
            }
            FrameKind::ClassBody => {
                self.top_mut().next_member();
                self.operand_done();
            }
            _ => {
                self.end_statement();
                self.clear_prev();
            }
        }
    }

    pub(super) fn comma(&mut self) {
        self.pop_concise();
        let top = self.top_kind();
        match top {
            FrameKind::Object | FrameKind::ClassBody => {
                let f = self.top_mut();
                f.next_member();
                f.open_questions = 0;
            }
            _ => {
                if self.top_declarator() != D_NONE {
                    self.top_mut().state = D_BINDING;
                }
                self.top_mut().open_questions = 0;
            }
        }
        self.operand_done();
    }

    pub(super) fn open_brace(&mut self) {
        let top = *self.top();
        let mut body = Frame::default();
        let kind = match top.kind {
            FrameKind::ClassHead if top.reg == C_INTERFACE => {
                // Interface body: a type literal that ends the statement.
                self.pop();
                self.push(FrameKind::TypeLit).state = L_INTERFACE_BODY;
                self.operand_done();
                self.decorator = 0;
                return;
            }
            // Object literal as heritage.
            FrameKind::ClassHead if self.prev_kw == tk!(KwExtends) => {
                body.is_value = true;
                FrameKind::Object
            }
            // A body takes what its head recorded.
            FrameKind::ClassHead | FrameKind::FnHead => {
                body = self.pop();
                if top.kind == FrameKind::FnHead { FrameKind::FnBody } else { FrameKind::ClassBody }
            }
            _ if self.prev_arrow => {
                body.is_async = self.arrow_async;
                FrameKind::ArrowBody
            }
            // Await is the operator in a static block, yield a name.
            FrameKind::ClassBody if top.mods & MOD_STATIC != 0 && top.state == M_KEY_POS => {
                body.is_async = true;
                FrameKind::FnBody
            }
            _ if matches!(self.stmt_reg(), S_IMPORT | S_EXPORT | S_ATTRS) => FrameKind::ModuleSpec,
            _ if !self.at_stmt_start() && self.operand_allowed() => {
                body.is_value = true;
                FrameKind::Object
            }
            _ => {
                // Statement frames reset their registers when a block opens.
                self.set_stmt_reg(S_NONE);
                FrameKind::Block
            }
        };
        let f = self.push(kind);
        f.is_value = body.is_value;
        f.is_generator = body.is_generator;
        f.is_async = body.is_async;
        self.expect = if kind.is_stmt_holder() { Expect::Statement } else { Expect::Operand };
        self.clear_prev();
        self.decorator = 0;
    }

    pub(super) fn close_brace(&mut self) {
        // Virtual frames above the brace end with it.
        let Some(f) = self.pop_to(|k| k.closer() == b'}') else {
            // Unbalanced: treat as a block end.
            self.unbalanced_close();
            return;
        };
        match f.kind {
            FrameKind::Object | FrameKind::FnBody | FrameKind::ClassBody if f.is_value => {
                self.value_done();
                self.member_done();
            }
            FrameKind::Container => {
                self.clear_prev();
            }
            // An attributes clause ends its declaration, like a body.
            FrameKind::ModuleSpec if self.stmt_reg() != S_ATTRS => {
                self.operand_done();
            }
            FrameKind::ArrowBody => {
                // The arrow function is complete: it cannot be continued.
                self.pop_concise();
                self.expect = Expect::Statement;
                self.clear_prev();
            }
            _ => {
                // A statement-level body: a new statement may start; inside a class body a new
                // member may start.
                if matches!(self.top_kind(), FrameKind::ClassBody | FrameKind::Object) {
                    self.member_done();
                    self.operand_done();
                } else {
                    self.after_statement();
                    self.clear_prev();
                }
            }
        }
    }

    pub(super) fn open_paren(&mut self) {
        let (top, kw) = (*self.top(), self.prev_kw);
        if matches!(top.kind, FrameKind::Object | FrameKind::ClassBody) && top.state == M_KEY_SEEN {
            // Method: give it a head so the body picks up its kind.
            let f = self.push(FrameKind::FnHead);
            f.is_generator = top.mods & MOD_GEN != 0;
            f.is_async = top.mods & MOD_ASYNC != 0;
        }
        let head = *self.top();
        if head.kind == FrameKind::FnHead {
            let f = self.push(FrameKind::Params);
            f.is_generator = head.is_generator;
            f.is_async = head.is_async;
        } else if matches_tk!(kw, KwIf | KwWhile | KwFor | KwWith | KwSwitch | KwCatch)
            && self.frames[self.stmt_frame()].kind.is_stmt_holder()
        {
            self.push(FrameKind::Head).state = if kw == tk!(KwFor) { F_OF } else { F_NO_OF };
        } else if self.operand_allowed() || kw == tk!(KwNew) {
            // The async of async (...) waits for a => rather than making an await context.
            self.push(FrameKind::Group).mods = if kw == tk!(KwAsync) { MOD_ASYNC } else { 0 };
        } else {
            self.push(FrameKind::Call);
        }
        self.operand_done();
    }

    pub(super) fn close_paren(&mut self) {
        let Some(f) = self.pop_to(|k| k.closer() == b')') else {
            self.unbalanced_close();
            return;
        };
        match f.kind {
            FrameKind::Head => {
                // A statement (or `{`) follows.
                self.expect = Expect::Statement;
                self.clear_prev();
            }
            FrameKind::Group => {
                self.value_done();
                self.closed_group = true;
                self.closed_group_async = f.mods & MOD_ASYNC != 0;
            }
            _ => self.value_done(),
        }
    }

    pub(super) fn open_bracket(&mut self) {
        let top = self.top_kind();
        let kind = if matches!(top, FrameKind::Object | FrameKind::ClassBody)
            && self.top().state == M_KEY_POS
        {
            FrameKind::ComputedKey
        } else {
            FrameKind::Array
        };
        self.push(kind);
        self.operand_done();
    }

    pub(super) fn close_bracket(&mut self) {
        let Some(f) = self.pop_to(|k| k.closer() == b']') else {
            self.unbalanced_close();
            return;
        };
        if f.kind == FrameKind::ComputedKey {
            self.top_mut().state = M_KEY_SEEN;
        }
        self.value_done();
    }
}
