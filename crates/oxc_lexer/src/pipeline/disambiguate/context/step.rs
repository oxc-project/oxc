use crate::token::{OP_KIND_BASE, tk};

use super::*;

pub(super) fn continues_expression(tokens: &Tokens, pos: usize) -> bool {
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
        return kw == K_IN
            || kw == K_INSTANCEOF
            || (tokens.ts && (kw == K_AS || kw == K_SATISFIES));
    }
    matches!(k, tk!(TemplateHead) | tk!(TemplateNoSub))
}

/// ., |, & may follow a line break inside a type; [, <, extends may not.
pub(super) fn continues_type_after_break(tokens: &Tokens, pos: usize) -> bool {
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
        if k == tk!(Whitespace)
            || k == tk!(LineComment)
            || k == tk!(BlockComment)
            || k == tk!(Hashbang)
        {
            return pos + 1;
        }
        let newline = tokens.line_break_between(self.prev_end, pos);
        self.stmt_done = false;

        if self.jsx_closing {
            if k == tk!(JsxTagEnd) {
                self.jsx_closing = false;
                self.jsx_element_done();
            }
            return pos + 1;
        }
        if matches!(self.top_kind(), FrameKind::JsxTag | FrameKind::JsxElem) {
            return self.step_jsx(tokens, pos, k);
        }

        // ASI: a value ended the previous line and this token cannot continue it.
        if newline && !self.operand_allowed() && !continues_expression(tokens, pos) {
            self.asi(tokens, pos);
        }
        // let x then a line break: only =, ,, ;, :, ! continue the declarator.
        if newline
            && !self.operand_allowed()
            && let Some(di) = self.decl_frame()
            && self.frames[di].state == D_BOUND
            && di == self.frames.len() - 1
        {
            let c = tokens.src[pos];
            let k = tokens.base_kind(pos);
            if !(k >= OP_KIND_BASE && matches!(c, b'=' | b',' | b';' | b':' | b'!')) {
                self.end_statement();
            }
        }
        // A type ended on the previous line and this token cannot continue it.
        if newline
            && self.top_kind() == FrameKind::TypeRegion
            && self.top().atom
            && !continues_type_after_break(tokens, pos)
        {
            self.end_region_by_break(tokens, pos);
        }
        // Restricted productions: a line break ends the statement.
        if newline
            && matches!(
                self.prev_kw,
                K_RETURN | K_THROW | K_BREAK | K_CONTINUE | K_DEBUGGER | K_YIELD
            )
            && self.top_kind() != FrameKind::TypeRegion
        {
            self.end_statement();
        }
        // The directive prologue ends at the first statement that is not a string literal.
        if self.at_stmt_start() {
            let si = self.stmt_frame();
            if self.frames[si].prologue == 1 && k != tk!(String) {
                self.frames[si].prologue = 0;
            }
        }

        let end = match k {
            tk!(Ident) => self.step_word(tokens, pos, newline),
            tk!(Number)
            | tk!(BigInt)
            | tk!(String)
            | tk!(RegExp)
            | tk!(TemplateNoSub)
            | tk!(PrivateIdent) => {
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
                if self.pop_to(&[FrameKind::Sub]).is_none() {
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
                if self.pop_to(&[FrameKind::Sub]).is_none() {
                    self.unbalanced();
                }
                if self.in_type() {
                    self.type_atom();
                } else {
                    self.value_done();
                }
                e
            }
            tk!(JsxLt) => {
                let tpos = tokens.next_sig(pos + 1);
                if tokens.src[tpos] == b'/' {
                    // Closing tag: the element it closes is the nearest JsxElem frame.
                    if self.pop_to(&[FrameKind::JsxElem]).is_none() {
                        self.unbalanced();
                    }
                    self.jsx_closing = true;
                } else {
                    self.push(FrameKind::JsxTag);
                }
                pos + 1
            }
            tk!(JsxTagEnd) | tk!(JsxText) => pos + 1,
            _ => self.step_op(tokens, pos, newline),
        };
        self.prev_end = end;
        end
    }

    pub(super) fn asi(&mut self, tokens: &Tokens, pos: usize) {
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
        // More head or the body continues the head; otherwise the break ends a bodiless signature.
        if matches!(self.top_kind(), FrameKind::FnHead | FrameKind::ClassHead) {
            let c = tokens.src[pos];
            let k = tokens.base_kind(pos);
            if k >= OP_KIND_BASE && (c == b'{' || c == b'<' || c == b'(') {
                return;
            }
            // Right after function / class the name may still follow a line break.
            let unnamed = matches!(self.prev_kw, K_FUNCTION | K_CLASS);
            if unnamed && (k == tk!(Ident) || (k >= OP_KIND_BASE && c == b'*')) {
                return;
            }
            if k == tk!(Ident) {
                let e = tokens.next_start(pos + 1);
                if matches!(tokens.word_kw(pos, e - pos), K_EXTENDS | K_IMPLEMENTS) {
                    return;
                }
            }
            self.end_statement();
            return;
        }
        match self.top_kind() {
            FrameKind::ClassBody => {
                // Member initializer ended; a new member starts.
                let f = self.top_mut();
                f.state = M_KEY_POS;
                f.mods = 0;
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
                // Inside a <...> list a line break is trivia: <T\nextends U> is one list.
            }
            _ => self.end_statement(),
        }
    }

    pub(super) fn end_region_by_break(&mut self, tokens: &Tokens, pos: usize) {
        let r = self.pop();
        match r.state {
            R_STMT | R_INTERFACE => self.end_statement(),
            R_INLINE => {
                // A declarator / member annotation ended by a line break.
                match self.top_kind() {
                    FrameKind::ClassBody => {
                        let f = self.top_mut();
                        f.state = M_KEY_POS;
                        f.mods = 0;
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
                // x as T then a new line: the value is complete.
                self.set_value();
                self.no_type_args = true;
                if !continues_expression(tokens, pos) {
                    self.asi(tokens, pos);
                }
            }
            _ => self.set_value(),
        }
    }

    pub(super) fn literal(&mut self, tokens: &Tokens, pos: usize, k: u8, end: usize) {
        if self.top_kind() == FrameKind::TypeRegion || self.in_type() {
            self.type_atom();
            return;
        }
        // Directive prologue.
        if k == tk!(String) {
            let si = self.stmt_frame();
            if self.frames[si].prologue != 0 && self.at_stmt_start() {
                let j = tokens.next_sig(end);
                let confirmed = j >= tokens.n
                    || (tokens.base_kind(j) >= OP_KIND_BASE
                        && (tokens.src[j] == b';' || tokens.src[j] == b'}'))
                    || (tokens.line_break_between(end, j) && !continues_expression(tokens, j));
                if confirmed {
                    if end - pos == 12
                        && tokens.ident_is(pos + 1, b"use strict")
                        && tokens.src[end - 1] == tokens.src[pos]
                    {
                        self.frames[si].strict = true;
                    }
                } else {
                    self.frames[si].prologue = 0;
                }
            }
            // Module specifier: import "x", ... from "x".
            let reg = self.stmt_reg();
            if (matches!(reg, S_IMPORT | S_IMPORT_NAME)
                && matches!(self.prev_kw, K_IMPORT | K_FROM))
                || (reg == S_EXPORT && self.prev_kw == K_FROM)
            {
                self.value_done();
                let nx = tokens.next_sig(end);
                let attrs = nx < tokens.n
                    && tokens.base_kind(nx) == tk!(Ident)
                    && !tokens.line_break_between(end, nx)
                    && (tokens.ident_is(nx, b"with") || tokens.ident_is(nx, b"assert"));
                if attrs {
                    self.set_stmt_reg(S_IMPORT);
                    return;
                }
                self.stmt_done = true;
                self.after_statement();
                return;
            }
            if reg == S_DECLARE_MODULE {
                // declare module "x": bodiless unless { follows.
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
        self.prev_num = k == tk!(Number) || k == tk!(BigInt);
    }
}
