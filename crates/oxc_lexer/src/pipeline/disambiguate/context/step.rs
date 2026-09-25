//! Stepping one token: the line-break rules (ASI, restricted productions, the end of a
//! type), literals, and the dispatch to words, punctuation, types and JSX.

use crate::token::{OP_KIND_BASE, is_trivia_byte, matches_tk, tk};

use super::*;

/// Can `tok` (the token starting at `pos`) continue an expression that a value token ended on the
/// previous line? Used for ASI.
fn continues_expression(tokens: &Tokens, pos: usize) -> bool {
    let k = tokens.base_kind(pos);
    if k >= OP_KIND_BASE {
        let c = tokens.src[pos];
        let c1 = tokens.src[pos + 1];
        if (c == b'+' || c == b'-') && c1 == c {
            return false;
        }
        return matches!(
            c,
            b'+' | b'-'
                | b'*'
                | b'/'
                | b'%'
                | b'&'
                | b'|'
                | b'^'
                | b'<'
                | b'>'
                | b'='
                | b'?'
                | b'.'
                | b','
                | b'('
                | b'['
                | b':'
                | b')'
                | b']'
                | b'}'
        );
    }
    if k == tk!(Ident) {
        let e = tokens.next_start(pos + 1);
        let kw = tokens.word_kw(pos, e - pos);
        return matches_tk!(kw, KwIn | KwInstanceof)
            || (tokens.ts && matches_tk!(kw, KwAs | KwSatisfies));
    }
    matches_tk!(k, TemplateHead | TemplateNoSub)
}

/// Can `pos` continue a type after a completed type atom on the previous line? `.`, `|`, `&` may
/// follow a line break; `[`, `<`, `extends` may not.
fn continues_type_after_break(tokens: &Tokens, pos: usize) -> bool {
    let k = tokens.base_kind(pos);
    if k >= OP_KIND_BASE {
        let c = tokens.src[pos];
        let c1 = tokens.src[pos + 1];
        return (c == b'.' && c1 != b'.')
            || (c == b'|' && c1 != b'|')
            || (c == b'&' && c1 != b'&')
            || c == b'?'
            || c == b':'
            || c == b','
            || c == b')'
            || c == b']'
            || c == b'}'
            || c == b'>'
            || (c == b'=' && c1 == b'>');
    }
    false
}

impl Walk {
    pub(super) fn step(&mut self, tokens: &Tokens, pos: usize) -> usize {
        self.last_start = pos;
        let k = tokens.base_kind(pos);
        if is_trivia_byte(k) {
            return pos + 1;
        }
        let newline = tokens.line_break_between(self.prev_end, pos);
        self.stmt_done = false;

        // Skipping the inside of a closing JSX tag.
        if self.jsx_closing {
            if k == tk!(JsxTagEnd) {
                self.jsx_closing = false;
                self.jsx_element_done();
            }
            return pos + 1;
        }
        // Inside an opening tag / children: only structure matters.
        if matches!(self.top_kind(), FrameKind::JsxTag | FrameKind::JsxElem) {
            return self.step_jsx(tokens, pos, k);
        }

        // ASI: a value ended the previous line and this token cannot continue it.
        if newline && !self.operand_allowed() && !continues_expression(tokens, pos) {
            self.asi(tokens, pos);
        }
        // `let x` then a line break: only `=`, `,`, `;`, `:` and `!` can continue the declarator,
        // anything else starts a new statement.
        if newline && !self.operand_allowed() && self.top_declarator() == D_BOUND {
            let c = tokens.src[pos];
            if !(k >= OP_KIND_BASE && matches!(c, b'=' | b',' | b';' | b':' | b'!')) {
                self.end_statement();
            }
        }
        // A type ended on the previous line and this token cannot continue it: the annotation, and
        // any statement it belongs to, is over.
        if newline
            && self.top_kind() == FrameKind::TypeRegion
            && self.top().atom
            && !continues_type_after_break(tokens, pos)
        {
            self.end_region_by_break(tokens, pos);
        }
        // Restricted productions: `return` / `throw` / `yield` / `break` / `continue` followed by a
        // line break end their statement.
        if newline
            && matches_tk!(
                self.prev_kw,
                KwReturn | KwThrow | KwBreak | KwContinue | KwDebugger | KwYield
            )
            && self.top_kind() != FrameKind::TypeRegion
        {
            self.end_statement();
        }
        // The directive prologue ends at the first statement that is not a string literal.
        if self.at_stmt_start() {
            let si = self.stmt_frame();
            if self.frames[si].prologue && k != tk!(String) {
                self.frames[si].prologue = false;
            }
        }

        let end = match k {
            tk!(Ident) => self.step_word(tokens, pos, newline),
            tk!(Number | BigInt | String | RegExp | TemplateNoSub | PrivateIdent) => {
                let e = tokens.next_start(pos + 1);
                self.literal(tokens, pos, k, e);
                e
            }
            tk!(TemplateHead) => {
                let e = tokens.next_start(pos + 1);
                let in_type = self.in_type();
                let f = self.push(FrameKind::Sub);
                f.decl = in_type;
                self.set_operand();
                e
            }
            tk!(TemplateMiddle) => {
                let e = tokens.next_start(pos + 1);
                if self.pop_to(|k| k == FrameKind::Sub).is_none() {
                    self.unbalanced();
                }
                let in_type = self.in_type();
                let f = self.push(FrameKind::Sub);
                f.decl = in_type;
                self.set_operand();
                e
            }
            tk!(TemplateTail) => {
                let e = tokens.next_start(pos + 1);
                if self.pop_to(|k| k == FrameKind::Sub).is_none() {
                    self.unbalanced();
                }
                if self.in_type() {
                    self.type_atom(false);
                } else {
                    self.value_done();
                }
                e
            }
            tk!(JsxLt) => self.jsx_lt(tokens, pos),
            tk!(JsxTagEnd | JsxText) => pos + 1,
            _ => self.step_op(tokens, pos, newline),
        };
        self.prev_end = end;
        end
    }

    fn asi(&mut self, tokens: &Tokens, pos: usize) {
        // Concise arrow bodies end with the statement.
        self.pop_concise();
        // An expression-embedded type region ends too.
        while self.top_kind() == FrameKind::TypeRegion && self.top().state == R_EXPR {
            self.pop();
        }
        if self.top_kind() == FrameKind::TypeRegion {
            // A declaration type: handled by the caller's type check.
            return;
        }
        // A head continues onto the next line when its body (or more head) follows; otherwise the
        // break ends a bodiless signature.
        if matches!(self.top_kind(), FrameKind::FnHead | FrameKind::ClassHead) {
            let c = tokens.src[pos];
            let k = tokens.base_kind(pos);
            if k >= OP_KIND_BASE && (c == b'{' || c == b'<' || c == b'(') {
                return;
            }
            // Right after `function` / `class`, the name (or a generator's `*`) may follow a
            // line break: nothing has been declared yet, so there is no signature to end.
            let unnamed = matches_tk!(self.prev_kw, KwFunction | KwClass);
            if unnamed && (k == tk!(Ident) || (k >= OP_KIND_BASE && c == b'*')) {
                return;
            }
            if k == tk!(Ident) {
                let e = tokens.next_start(pos + 1);
                if matches_tk!(tokens.word_kw(pos, e - pos), KwExtends | KwImplements) {
                    return;
                }
            }
            self.end_statement();
            return;
        }
        match self.top_kind() {
            FrameKind::ClassBody => {
                // Member initializer ended; a new member starts.
                self.top_mut().next_member();
                self.set_operand();
            }
            FrameKind::Object
            | FrameKind::Call
            | FrameKind::Group
            | FrameKind::Array
            | FrameKind::Index
            | FrameKind::Params
            | FrameKind::Sub
            | FrameKind::Container
            | FrameKind::Head
            | FrameKind::ComputedKey => {
                // No statements here; nothing to end.
            }
            FrameKind::Angle => {
                // Inside a `<...>` list a line break is trivia: `<T\nextends U>` is one list.
            }
            _ => self.end_statement(),
        }
    }

    fn end_region_by_break(&mut self, tokens: &Tokens, pos: usize) {
        let r = self.pop();
        match r.state {
            R_STMT | R_INTERFACE => self.end_statement(),
            R_INLINE => {
                // A declarator / member annotation ended by a line break.
                match self.top_kind() {
                    FrameKind::ClassBody => {
                        self.top_mut().next_member();
                        self.set_operand();
                    }
                    FrameKind::FnHead => {
                        // Return type of a bodiless signature.
                        self.pop();
                        self.end_statement();
                    }
                    _ => self.end_statement(),
                }
            }
            R_EXPR => {
                // `x as T` then a new line: the value is complete.
                self.set_value();
                self.no_type_args = true;
                if !continues_expression(tokens, pos) {
                    self.asi(tokens, pos);
                }
            }
            _ => self.set_value(),
        }
    }

    fn literal(&mut self, tokens: &Tokens, pos: usize, k: u8, end: usize) {
        if self.in_type() {
            self.type_atom(false);
            return;
        }
        // Directive prologue.
        if k == tk!(String) {
            let si = self.stmt_frame();
            if self.frames[si].prologue && self.at_stmt_start() {
                let j = tokens.peek(end);
                let confirmed = j.kind == tk!(Eof)
                    || (j.kind >= OP_KIND_BASE && matches!(j.byte, b';' | b'}'))
                    || (tokens.line_break_between(end, j.pos)
                        && !continues_expression(tokens, j.pos));
                if confirmed {
                    if end - pos == 12
                        && tokens.ident_is(pos + 1, b"use strict")
                        && tokens.src[end - 1] == tokens.src[pos]
                    {
                        self.frames[si].strict = true;
                    }
                } else {
                    self.frames[si].prologue = false;
                }
            }
            // Module specifier: `import "x"`, `... from "x"`.
            let reg = self.stmt_reg();
            if (matches!(reg, S_IMPORT | S_IMPORT_NAME)
                && matches_tk!(self.prev_kw, KwImport | KwFrom))
                || (reg == S_EXPORT && self.prev_kw == tk!(KwFrom))
            {
                self.value_done();
                let nx = tokens.peek(end);
                let attrs = nx.kind == tk!(Ident)
                    && !tokens.line_break_between(end, nx.pos)
                    && (tokens.ident_is(nx.pos, b"with") || tokens.ident_is(nx.pos, b"assert"));
                if attrs {
                    self.set_stmt_reg(S_IMPORT);
                    return;
                }
                self.stmt_done = true;
                self.after_statement();
                return;
            }
            if reg == S_DECLARE_MODULE {
                // `declare module "x"`: bodiless unless `{` follows.
                self.value_done();
                self.stmt_done = true;
                return;
            }
        }
        // Member keys.
        match self.top_kind() {
            FrameKind::Object | FrameKind::ClassBody if self.top().state == M_KEY_POS => {
                self.top_mut().state = M_KEY_SEEN;
                self.value_done();
                return;
            }
            FrameKind::Head if self.top().state == F_START => {
                self.top_mut().state = F_EXPR;
            }
            _ => {}
        }
        self.value_done();
        self.prev_num = matches_tk!(k, Number | BigInt);
    }
}
