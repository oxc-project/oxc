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

        // In a `for (` head an operator other than member access makes the binding an expression
        // (`for (x = of / 2;;)`), so a later `of` is a plain identifier.
        if self.top_kind() == FrameKind::Head
            && self.top().state == F_BOUND
            && !matches!(c, b'.' | b'[' | b'(' | b')' | b']' | b'}' | b'{' | b',' | b';')
            && !(c == b'?' && c1 == b'.')
        {
            self.top_mut().state = F_EXPR;
        }

        // Type context: brackets and separators belong to the type.
        if self.in_type() {
            // `>` closes one Angle per byte; `<<` opens two.
            if c == b'>' && !(c1 == b'=' && self.top_kind() != FrameKind::Angle) {
                len = 1;
            }
            if c == b'<' && c1 == b'<' {
                len = 1;
            }
            if c == b'=' && c1 == b'>' {
                len = 2;
            }
            return self.type_op(tokens, pos, c, len);
        }

        match c {
            b'{' => {
                self.open_brace();
                pos + 1
            }
            b'}' => {
                self.close_brace();
                pos + 1
            }
            b'(' => {
                self.open_paren();
                pos + 1
            }
            b')' => {
                self.close_paren();
                pos + 1
            }
            b'[' => {
                self.open_bracket();
                pos + 1
            }
            b']' => {
                self.close_bracket();
                pos + 1
            }
            b';' => {
                self.semicolon();
                pos + 1
            }
            b',' => {
                self.comma();
                pos + 1
            }
            b':' => {
                self.colon(tokens);
                pos + 1
            }
            b'?' => {
                if len >= 2 {
                    // `?.` / `??` / `??=`
                    if c1 == b'.' {
                        self.operand_done();
                        self.after_dot = true;
                    } else {
                        self.operand_done();
                    }
                    return pos + len;
                }
                self.question(tokens, pos);
                pos + 1
            }
            b'.' => {
                if len == 3 {
                    // spread: what follows is a value, not a member key
                    if self.top_kind() == FrameKind::Object {
                        self.top_mut().state = M_VALUE;
                    }
                    self.operand_done();
                    return pos + 3;
                }
                if self.prev_num && self.prev_end == pos {
                    // `1.` continues the numeric literal.
                    self.set_value();
                    self.prev_num = false;
                    return pos + 1;
                }
                self.operand_done();
                self.after_dot = true;
                pos + 1
            }
            b'=' => {
                if len == 2 && c1 == b'>' {
                    self.arrow(tokens, pos);
                    return pos + 2;
                }
                if len == 1 {
                    self.assign();
                    return pos + 1;
                }
                // `==` / `===`
                self.operand_done();
                pos + len
            }
            b'!' => {
                if len == 1 && tokens.ts && !self.operand_allowed() && !newline {
                    // Postfix non-null / definite assignment.
                    self.set_value();
                    self.clear_prev();
                    return pos + 1;
                }
                self.operand_done();
                pos + len
            }
            b'+' | b'-' => {
                if len == 2 && c1 == c {
                    // `++` / `--`: postfix keeps the value.
                    if !self.operand_allowed() && !newline {
                        self.set_value();
                        self.clear_prev();
                    } else {
                        self.operand_done();
                    }
                    return pos + 2;
                }
                self.operand_done();
                pos + len
            }
            b'*' => {
                if len == 1 {
                    if self.top_kind() == FrameKind::FnHead {
                        self.top_mut().is_generator = true;
                        self.set_value();
                        self.clear_prev();
                        self.prev_kw = tk!(KwFunction);
                        return pos + 1;
                    }
                    if matches!(self.top_kind(), FrameKind::Object | FrameKind::ClassBody)
                        && self.top().state == M_KEY_POS
                    {
                        self.top_mut().mods |= MOD_GEN;
                        self.operand_done();
                        return pos + 1;
                    }
                    if self.prev_kw == tk!(KwYield) {
                        // `yield*`
                        self.set_operand();
                        return pos + 1;
                    }
                }
                self.operand_done();
                pos + len
            }
            b'<' => {
                if len >= 2 {
                    if c1 == b'<'
                        && len == 2
                        && tokens.ts
                        && !self.operand_allowed()
                        && !self.no_type_args
                        && lt_run_split(tokens, pos)
                    {
                        // Two openers, not a shift.
                        self.less_than(tokens, pos);
                        return pos + 1;
                    }
                    self.operand_done();
                    return pos + len;
                }
                self.less_than(tokens, pos);
                pos + 1
            }
            b'>' => {
                if self.top_kind() == FrameKind::Angle {
                    self.close_angle();
                    return pos + 1;
                }
                self.operand_done();
                pos + len
            }
            b'@' => {
                if self.decorator == 0 {
                    // `export @dec class` and `export default @dec class` decorate declarations.
                    let decl = self.at_stmt_start()
                        || !self.operand_allowed()
                        || self.prev_kw == tk!(KwExport)
                        || self.export_default;
                    self.decorator = if decl { 1 } else { 2 };
                }
                self.operand_done();
                pos + 1
            }
            b'#' => {
                // Stray `#` (private names are tk!(PrivateIdent) tokens).
                self.operand_done();
                pos + 1
            }
            _ => {
                // Every other operator expects an operand.
                self.operand_done();
                if self.top_kind() == FrameKind::Head && self.top().state != F_ITER {
                    self.top_mut().state = F_EXPR;
                }
                pos + len
            }
        }
    }

    pub(super) fn arrow(&mut self, tokens: &Tokens, pos: usize) {
        let is_async = if self.closed_group { self.closed_group_async } else { self.arrow_async };
        self.prev_arrow = true;
        // Concise body unless `{` follows.
        let nx = tokens.peek(pos + 2);
        let block = nx.kind >= OP_KIND_BASE && nx.byte == b'{';
        if !block {
            let f = self.push(FrameKind::Concise);
            f.is_generator = false;
            f.is_async = is_async;
            f.reserved = false;
        }
        self.operand_done();
        self.prev_arrow = true;
        self.arrow_async = is_async;
    }

    pub(super) fn assign(&mut self) {
        let reg = self.top_reg();
        if reg == S_TYPE_NAME {
            // `type X =`: the alias type.
            self.set_stmt_reg(S_NONE);
            self.open_region(R_STMT, true);
            return;
        }
        if reg == S_IMPORT_NAME {
            // `import X = ...`: a module reference.
            self.set_stmt_reg(S_NONE);
            self.open_region(R_STMT, true);
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
            FrameKind::ClassBody => {
                if tokens.ts {
                    self.open_region(R_INLINE, true);
                } else {
                    self.operand_done();
                }
                return;
            }
            FrameKind::Params => {
                if tokens.ts {
                    self.open_region(R_INLINE, true);
                } else {
                    self.operand_done();
                }
                return;
            }
            FrameKind::Group | FrameKind::Call => {
                if tokens.ts && !self.operand_allowed() {
                    // Arrow parameter annotation.
                    self.open_region(R_INLINE, true);
                } else {
                    self.operand_done();
                }
                return;
            }
            FrameKind::FnHead => {
                // Return type.
                self.open_region(R_INLINE, true);
                return;
            }
            FrameKind::ComputedKey => {
                // Index signature `[k: string]`.
                if tokens.ts {
                    self.open_region(R_INLINE, true);
                } else {
                    self.operand_done();
                }
                return;
            }
            _ => {}
        }
        match self.stmt_reg() {
            S_CASE | S_LABEL => {
                self.set_stmt_reg(S_NONE);
                self.after_statement();
                self.clear_prev();
                return;
            }
            _ => {}
        }
        if tokens.ts && self.top_declarator() == D_BOUND {
            // Declarator type annotation.
            self.open_region(R_INLINE, true);
            return;
        }
        if tokens.ts && self.closed_group && top != FrameKind::Head {
            // `(a): T =>` arrow return type; remember the group's `async`.
            let is_async = self.closed_group_async;
            self.open_region(R_ARROW_RET, true);
            self.top_mut().is_async = is_async;
            return;
        }
        self.operand_done();
    }

    pub(super) fn semicolon(&mut self) {
        self.pop_concise();
        match self.top_kind() {
            FrameKind::Head => {
                let f = self.top_mut();
                f.state = F_ITER;
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
        while self.top_kind() == FrameKind::TypeRegion {
            self.pop();
        }
        let top = self.top_kind();
        match top {
            FrameKind::Object | FrameKind::ClassBody => {
                let f = self.top_mut();
                f.next_member();
                f.open_questions = 0;
            }
            FrameKind::Head => {
                let f = self.top_mut();
                f.state = F_START;
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
        let top = self.top_kind();
        let kind;
        let mut value = false;
        let mut generator = false;
        let mut is_async = false;
        let mut strict = self.top().strict;
        let mut reserved = false;
        if matches!(top, FrameKind::JsxTag | FrameKind::JsxElem) {
            kind = FrameKind::Container;
        } else if top == FrameKind::ClassHead {
            if self.top().reg == C_INTERFACE {
                // Interface body: a type literal that ends the statement.
                self.pop();
                let f = self.push(FrameKind::TypeLit);
                f.decl = true;
                f.is_value = false;
                f.state = L_INTERFACE_BODY;
                self.operand_done();
                self.decorator = 0;
                return;
            }
            if self.top().state == C_EXTENDS && self.prev_kw == tk!(KwExtends) {
                // Object literal as heritage.
                kind = FrameKind::Object;
                value = true;
            } else {
                let h = self.pop();
                kind = FrameKind::ClassBody;
                value = h.is_value;
                strict = true;
            }
        } else if top == FrameKind::FnHead {
            let h = self.pop();
            kind = FrameKind::FnBody;
            value = h.is_value;
            generator = h.is_generator;
            is_async = h.is_async;
        } else if self.prev_arrow {
            kind = FrameKind::ArrowBody;
            is_async = self.arrow_async;
        } else if top == FrameKind::ClassBody
            && self.top().mods & MOD_STATIC != 0
            && self.top().state == M_KEY_POS
        {
            kind = FrameKind::StaticBlock;
            reserved = true;
            strict = true;
        } else {
            let reg = self.stmt_reg();
            if self.top_declarator() == D_BINDING {
                kind = FrameKind::Pattern;
            } else if matches!(reg, S_IMPORT | S_EXPORT | S_IMPORT_NAME) {
                kind = FrameKind::ModuleSpec;
            } else if reg == S_ENUM {
                kind = FrameKind::EnumBody;
            } else if reg == S_NAMESPACE || reg == S_DECLARE_MODULE || self.prev_kw == tk!(KwGlobal)
            {
                kind = FrameKind::Block;
            } else if matches!(top, FrameKind::Object) && self.top().state != M_VALUE {
                // `{` at key position of an object literal: malformed; treat as a nested object.
                kind = FrameKind::Object;
                value = true;
            } else if self.at_stmt_start() {
                kind = FrameKind::Block;
            } else if self.operand_allowed() {
                kind = FrameKind::Object;
                value = true;
            } else {
                kind = FrameKind::Block;
            }
        }
        // Statement frames reset their registers when a block opens.
        if kind == FrameKind::EnumBody || kind == FrameKind::Block || kind == FrameKind::TypeLit {
            self.set_stmt_reg(S_NONE);
        }
        let f = self.push(kind);
        f.is_value = value;
        f.strict = strict;
        f.reserved |= reserved;
        if matches!(kind, FrameKind::FnBody | FrameKind::ArrowBody) {
            f.is_generator = generator;
            f.is_async = is_async;
            f.reserved = false;
            f.prologue = true;
        }
        if kind == FrameKind::StaticBlock {
            f.is_generator = false;
            f.is_async = false;
        }
        if kind == FrameKind::TypeLit {
            f.decl = true;
            f.state = L_INTERFACE_BODY;
        }
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
            FrameKind::Object | FrameKind::TypeLit if f.is_value => {
                self.value_done();
                self.member_done();
            }
            FrameKind::FnBody | FrameKind::ClassBody if f.is_value => {
                self.value_done();
                self.member_done();
            }
            FrameKind::Container => {
                self.clear_prev();
            }
            FrameKind::Pattern => {
                if let Some(di) = self.decl_frame() {
                    self.frames[di].state = D_BOUND;
                }
                self.value_done();
            }
            FrameKind::ModuleSpec => {
                self.operand_done();
            }
            FrameKind::TypeLit => {
                if f.state == L_INTERFACE_BODY {
                    if self.top_kind() == FrameKind::TypeRegion {
                        self.pop();
                    }
                    self.end_statement();
                } else if self.region_index().is_some() {
                    self.type_atom(false);
                } else {
                    self.after_statement();
                    self.clear_prev();
                }
            }
            FrameKind::FnBody
            | FrameKind::ClassBody
            | FrameKind::StaticBlock
            | FrameKind::Block
            | FrameKind::EnumBody => {
                // A statement-level body: a new statement may start; inside a class body a new
                // member may start.
                if matches!(self.top_kind(), FrameKind::ClassBody | FrameKind::Object) {
                    self.member_done();
                    self.operand_done();
                } else if f.kind == FrameKind::FnBody
                    && matches!(self.top_kind(), FrameKind::TypeLit)
                {
                    self.operand_done();
                } else {
                    self.after_statement();
                    self.clear_prev();
                }
            }
            FrameKind::ArrowBody => {
                // The arrow function is complete: it cannot be continued.
                self.pop_concise();
                self.expect = Expect::Statement;
                self.clear_prev();
            }
            _ => {
                self.after_statement();
                self.clear_prev();
            }
        }
    }

    pub(super) fn open_paren(&mut self) {
        let top = self.top_kind();
        let si = self.stmt_frame();
        let head = if self.frames[si].kind.is_stmt_holder() { self.frames[si].head } else { 0 };
        let kind;
        let mut generator = false;
        let mut is_async = false;
        let mut head_kind = 0u8;
        if top == FrameKind::FnHead {
            kind = FrameKind::Params;
            generator = self.top().is_generator;
            is_async = self.top().is_async;
        } else if matches!(top, FrameKind::Object | FrameKind::ClassBody)
            && self.top().state == M_KEY_SEEN
        {
            // Method: give it a head so the body picks up its kind.
            let m = self.top().mods;
            let f = self.push(FrameKind::FnHead);
            f.is_value = false;
            f.is_generator = m & MOD_GEN != 0;
            f.is_async = m & MOD_ASYNC != 0;
            kind = FrameKind::Params;
            generator = m & MOD_GEN != 0;
            is_async = m & MOD_ASYNC != 0;
        } else if head != 0
            && matches_tk!(self.prev_kw, KwIf | KwWhile | KwFor | KwWith | KwSwitch | KwCatch)
        {
            kind = FrameKind::Head;
            head_kind = head;
            self.frames[si].head = 0;
        } else if self.operand_allowed() || self.prev_kw == tk!(KwNew) {
            kind = FrameKind::Group;
            is_async = self.prev_async;
        } else {
            kind = FrameKind::Call;
        }
        let f = self.push(kind);
        f.head = head_kind;
        f.open_questions = 0;
        if kind == FrameKind::Params {
            f.is_generator = generator;
            f.is_async = is_async;
            f.reserved = false;
        }
        if kind == FrameKind::Group {
            f.is_async = is_async;
        }
        if kind == FrameKind::Head {
            f.state = F_START;
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
                let si = self.stmt_frame();
                self.frames[si].head = 0;
            }
            FrameKind::Params => {
                self.set_value();
                self.clear_prev();
                self.closed_params = true;
            }
            FrameKind::Group => {
                self.value_done();
                self.closed_group = true;
                self.closed_group_async = f.is_async;
            }
            _ => {
                self.value_done();
            }
        }
    }

    pub(super) fn open_bracket(&mut self) {
        let top = self.top_kind();
        let kind = if matches!(top, FrameKind::Object | FrameKind::ClassBody)
            && self.top().state == M_KEY_POS
        {
            FrameKind::ComputedKey
        } else if self.top_declarator() == D_BINDING {
            FrameKind::ArrayPattern
        } else if self.operand_allowed() {
            FrameKind::Array
        } else {
            FrameKind::Index
        };
        let f = self.push(kind);
        f.open_questions = 0;
        self.operand_done();
    }

    pub(super) fn close_bracket(&mut self) {
        let Some(f) = self.pop_to(|k| k.closer() == b']') else {
            self.unbalanced_close();
            return;
        };
        match f.kind {
            FrameKind::ComputedKey => {
                self.top_mut().state = M_KEY_SEEN;
                self.value_done();
            }
            FrameKind::ArrayPattern => {
                if let Some(di) = self.decl_frame() {
                    self.frames[di].state = D_BOUND;
                }
                self.value_done();
            }
            _ => {
                self.value_done();
            }
        }
    }
}
