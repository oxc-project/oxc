use super::*;

impl Walk {
    pub(super) fn open_region(&mut self, rule: u8, decl: bool) {
        let f = self.push(FrameKind::TypeRegion);
        f.decl = decl;
        f.state = rule;
        f.atom = false;
        f.inner = false;
        self.operand_done();
    }

    pub(super) fn end_region_for(&mut self) -> bool {
        if self.top_kind() != FrameKind::TypeRegion {
            return false;
        }
        let r = self.pop();
        self.set_value();
        if r.state == R_EXPR {
            self.no_type_args = true;
        }
        true
    }

    pub(super) fn type_op(&mut self, tokens: &Tokens, pos: usize, c: u8, len: usize) -> usize {
        let top = self.top_kind();
        match c {
            b'(' => {
                if top == FrameKind::TypeRegion && self.top().atom && self.top().state == R_EXPR {
                    // x as T ( cannot continue the type.
                    self.end_region_for();
                    self.open_paren();
                    return pos + 1;
                }
                let f = self.push(FrameKind::TypeParen);
                f.decl = true;
                self.operand_done();
                pos + 1
            }
            b')' => {
                if top == FrameKind::TypeParen {
                    self.pop();
                    if let Some(i) = self.region_index() {
                        let r = &mut self.frames[i];
                        r.atom = true;
                        r.inner = true;
                    }
                    self.set_value();
                    self.clear_prev();
                    return pos + 1;
                }
                // Closes something outside the type.
                self.pop_virtual();
                self.close_paren();
                pos + 1
            }
            b'[' => {
                if top == FrameKind::TypeRegion
                    && self.top().atom
                    && self.top().state == R_EXPR
                    && tokens.line_break_between(self.prev_end, pos)
                {
                    self.end_region_for();
                    self.open_bracket();
                    return pos + 1;
                }
                let f = self.push(FrameKind::TypeBracket);
                f.decl = true;
                self.operand_done();
                pos + 1
            }
            b']' => {
                if top == FrameKind::TypeBracket {
                    self.pop();
                    if let Some(i) = self.region_index() {
                        let r = &mut self.frames[i];
                        r.atom = true;
                        r.inner = false;
                    }
                    self.set_value();
                    self.clear_prev();
                    return pos + 1;
                }
                self.pop_virtual();
                self.close_bracket();
                pos + 1
            }
            b'{' => {
                if top == FrameKind::TypeRegion && self.top().atom {
                    // A body follows a completed type (): T {).
                    let r = self.pop();
                    if r.state == R_INTERFACE {
                        // interface X extends Y {: the body.
                        let f = self.push(FrameKind::TypeLit);
                        f.decl = true;
                        f.is_value = false;
                        f.state = L_INTERFACE_BODY;
                        self.operand_done();
                        return pos + 1;
                    }
                    self.set_value();
                    self.open_brace();
                    return pos + 1;
                }
                let expr = self.region_index().is_some_and(|i| !self.frames[i].decl);
                let f = self.push(FrameKind::TypeLit);
                f.decl = !expr;
                f.is_value = expr;
                self.operand_done();
                pos + 1
            }
            b'}' => {
                if top == FrameKind::TypeLit {
                    let f = self.pop();
                    if f.state == L_INTERFACE_BODY {
                        // Interface body done: statement over.
                        if self.top_kind() == FrameKind::TypeRegion {
                            self.pop();
                        }
                        self.end_statement();
                        return pos + 1;
                    }
                    if let Some(i) = self.region_index() {
                        let r = &mut self.frames[i];
                        r.atom = true;
                        r.inner = false;
                    }
                    self.set_value();
                    self.clear_prev();
                    return pos + 1;
                }
                self.pop_virtual();
                self.close_brace();
                pos + 1
            }
            b'<' => {
                if keyword_type(self.prev_kw) {
                    // this / any / null... take no type arguments: this < compares.
                    self.end_region_for();
                    self.operand_done();
                    return pos + 1;
                }
                let decl = self.region_index().is_none_or(|i| self.frames[i].decl);
                let f = self.push(FrameKind::Angle);
                f.decl = decl;
                f.state = A_IN_TYPE;
                self.operand_done();
                pos + 1
            }
            b'>' => {
                if top == FrameKind::Angle {
                    self.close_angle();
                    return pos + 1;
                }
                // Relational > after x as T: the type is over.
                self.end_region_for();
                self.operand_done();
                pos + len
            }
            b',' => {
                match top {
                    FrameKind::Angle
                    | FrameKind::TypeParen
                    | FrameKind::TypeBracket
                    | FrameKind::TypeLit => {
                        self.type_operator();
                    }
                    _ => {
                        // Ends the region: next declarator / parameter / argument.
                        self.pop();
                        self.comma();
                    }
                }
                pos + 1
            }
            b';' => {
                if top == FrameKind::TypeLit {
                    self.type_operator();
                    return pos + 1;
                }
                self.pop_virtual();
                self.semicolon();
                pos + 1
            }
            b'=' => {
                if len == 2 {
                    // => continues a function type only right after its parameter list.
                    if top == FrameKind::TypeRegion && self.top().inner {
                        self.type_operator();
                        return pos + 2;
                    }
                    if top == FrameKind::TypeRegion && self.top().state == R_ARROW_RET {
                        let r = self.pop();
                        self.closed_group = true;
                        self.closed_group_async = r.is_async;
                        self.arrow(tokens, pos);
                        return pos + 2;
                    }
                    if matches!(
                        top,
                        FrameKind::Angle
                            | FrameKind::TypeParen
                            | FrameKind::TypeBracket
                            | FrameKind::TypeLit
                    ) {
                        self.type_operator();
                        return pos + 2;
                    }
                    self.end_region_for();
                    self.arrow(tokens, pos);
                    return pos + 2;
                }
                if top == FrameKind::Angle {
                    // Type-parameter default.
                    self.type_operator();
                    return pos + 1;
                }
                if matches!(top, FrameKind::TypeLit | FrameKind::TypeParen | FrameKind::TypeBracket)
                {
                    self.type_operator();
                    return pos + 1;
                }
                // Initializer / default value: the region ends.
                self.pop();
                self.assign();
                pos + 1
            }
            b'?' | b':' | b'|' | b'&' | b'.' | b'-' | b'+' | b'*' => {
                if len >= 2 && matches!(c, b'|' | b'&' | b'?') && tokens.src[pos + 1] == c {
                    // || / && / ??: expression operators.
                    self.end_region_for();
                    self.operand_done();
                    return pos + len;
                }
                if c == b'?' && tokens.src[pos + 1] == b'.' {
                    self.end_region_for();
                    self.operand_done();
                    self.after_dot = true;
                    return pos + 2;
                }
                if c == b'.' && len == 3 {
                    self.type_operator();
                    return pos + 3;
                }
                if c == b'?'
                    && top == FrameKind::TypeRegion
                    && self.top().atom
                    && self.top().state == R_EXPR
                    && self.top().open_questions == 0
                {
                    // x as T ? a : b: a conditional expression.
                    self.end_region_for();
                    self.question(tokens, pos);
                    return pos + 1;
                }
                if c == b':' && top == FrameKind::TypeRegion && self.top().open_questions > 0 {
                    // The : of a conditional type pays its ?.
                    self.top_mut().open_questions -= 1;
                }
                self.type_operator();
                pos + len
            }
            b'!' => {
                // x as T!: not a type token.
                self.end_region_for();
                self.set_value();
                self.clear_prev();
                pos + 1
            }
            _ => {
                // Any other operator ends the type.
                self.end_region_for();
                self.operand_done();
                pos + len
            }
        }
    }

    pub(super) fn close_angle(&mut self) {
        let a = self.pop();
        match a.state {
            A_ASSERT => {
                // Type assertion <T>: an operand follows.
                if self.top_kind() == FrameKind::TypeRegion && self.top().state == R_ASSERT {
                    self.pop();
                }
                self.operand_done();
            }
            A_EXPR_ARGS => {
                // An instantiation is a value, and no second list may follow.
                self.set_value();
                self.clear_prev();
                self.no_type_args = true;
            }
            A_DECL_PARAMS => {
                // Type parameters of a declaration head.
                self.set_value();
                self.clear_prev();
                match self.top_kind() {
                    FrameKind::FnHead => self.prev_kw = K_FUNCTION,
                    FrameKind::ClassHead => self.prev_kw = K_CLASS,
                    _ => {}
                }
            }
            _ => {
                if let Some(i) = self.region_index() {
                    let r = &mut self.frames[i];
                    r.atom = true;
                    r.inner = false;
                }
                self.set_value();
                self.clear_prev();
            }
        }
    }

    pub(super) fn less_than(&mut self, tokens: &Tokens, pos: usize) {
        // Type parameters of a declaration head or member.
        let head = match self.top_kind() {
            FrameKind::FnHead | FrameKind::ClassHead => true,
            FrameKind::Object | FrameKind::ClassBody => self.top().state == M_KEY_SEEN,
            _ => self.stmt_reg() == S_TYPE_NAME,
        };
        if tokens.ts && head {
            let f = self.push(FrameKind::Angle);
            f.decl = true;
            f.state = A_DECL_PARAMS;
            self.operand_done();
            return;
        }
        if tokens.ts && self.operand_allowed() {
            // <T>x assertion / <T,>() => generic arrow: a type list.
            self.open_region(R_ASSERT, false);
            let f = self.push(FrameKind::Angle);
            f.decl = false;
            f.state = A_ASSERT;
            self.operand_done();
            return;
        }
        if tokens.ts && !self.operand_allowed() && !self.no_type_args {
            // After a value: type arguments (f<T>(x)) or less-than.
            if self.expr_type_args(tokens, pos) {
                let f = self.push(FrameKind::Angle);
                f.decl = false;
                f.state = A_EXPR_ARGS;
                self.operand_done();
                return;
            }
        }
        self.operand_done();
        if self.top_kind() == FrameKind::Head && self.top().state != F_ITER {
            self.top_mut().state = F_EXPR;
        }
    }

    pub(super) fn expr_type_args(&mut self, tokens: &Tokens, lt: usize) -> bool {
        type_args_at(tokens, lt)
    }
}
