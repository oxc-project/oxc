use crate::token::{OP_KIND_BASE, tk};

use super::*;

/// Reserved statement keywords always start a statement; contextual ones only after a line break.
pub(super) fn is_stmt_keyword(kw: u8, newline: bool, ts: bool) -> bool {
    match kw {
        K_IF | K_FOR | K_WHILE | K_RETURN | K_VAR | K_CONST | K_SWITCH | K_TRY | K_THROW | K_DO
        | K_WITH | K_BREAK | K_CONTINUE | K_DEBUGGER | K_FUNCTION | K_CLASS | K_IMPORT
        | K_EXPORT | K_ENUM => true,
        K_LET | K_ASYNC | K_TYPE | K_INTERFACE | K_DECLARE | K_NAMESPACE | K_MODULE
        | K_ABSTRACT | K_USING => newline && (ts || matches!(kw, K_LET | K_ASYNC | K_USING)),
        _ => false,
    }
}

impl Walk {
    pub(super) fn step_word(&mut self, tokens: &Tokens, pos: usize, newline: bool) -> usize {
        let end = tokens.next_start(pos + 1);
        let kw = if self.after_dot { 0 } else { tokens.word_kw(pos, end - pos) };

        if self.in_type() {
            self.type_word(kw);
            return end;
        }
        // for await (: the await belongs to the head.
        if kw == K_AWAIT && self.prev_kw == K_FOR {
            self.for_await = true;
            return end;
        }
        let kw = self.resolve_keyword(tokens, end, kw);

        // Member keys in object literals / class bodies.
        if matches!(self.top_kind(), FrameKind::Object | FrameKind::ClassBody)
            && self.top().state == M_KEY_POS
        {
            return self.member_word(tokens, pos, end, kw);
        }
        self.statement_keyword_break(kw, newline, tokens.ts);
        if self.declared_name(tokens, end, kw) {
            return end;
        }
        self.keyword_word(tokens, end, kw);
        end
    }

    fn type_word(&mut self, kw: u8) {
        match kw {
            K_KEYOF | K_TYPEOF | K_READONLY | K_UNIQUE | K_INFER | K_ABSTRACT | K_NEW
            | K_ASSERTS | K_IMPORT | K_EXTENDS | K_IS | K_IN | K_AS | K_SATISFIES => {
                self.type_operator();
                self.prev_kw = kw;
                if kw == K_EXTENDS {
                    // A conditional type: its ? and : belong to the type.
                    if let Some(i) = self.region_index() {
                        self.frames[i].open_questions += 1;
                    }
                }
            }
            _ => {
                // A statement keyword right after a completed type is an error; read it as an atom.
                self.type_atom();
                if keyword_type(kw) {
                    self.prev_kw = kw;
                }
            }
        }
    }

    /// A contextual keyword is a plain name (0) unless its position and next token say otherwise.
    fn resolve_keyword(&self, tokens: &Tokens, end: usize, kw: u8) -> u8 {
        match kw {
            K_YIELD => {
                if !self.yield_is_keyword() {
                    return 0;
                }
            }
            K_AWAIT => {
                if !self.await_is_keyword() {
                    return 0;
                }
            }
            K_OF => {
                if !(self.top_kind() == FrameKind::Head
                    && self.top().head == H_FOR
                    && self.top().state == F_BOUND)
                {
                    return 0;
                }
            }
            K_LET => {
                let nx = tokens.next_sig(end);
                let ok = nx < tokens.n && {
                    let nk = tokens.base_kind(nx);
                    let c = tokens.src[nx];
                    nk == tk!(Ident) || (nk >= OP_KIND_BASE && (c == b'[' || c == b'{'))
                };
                let at_stmt = self.at_stmt_start()
                    || self.top_kind() == FrameKind::Head
                    || matches!(self.prev_kw, K_DECLARE | K_EXPORT);
                if !ok || !at_stmt {
                    return 0;
                }
            }
            K_USING => {
                let nx = tokens.next_sig(end);
                let ok = nx < tokens.n
                    && tokens.base_kind(nx) == tk!(Ident)
                    && !tokens.line_break_between(end, nx)
                    && (self.at_stmt_start() || matches!(self.prev_kw, K_DECLARE | K_EXPORT));
                if !ok {
                    return 0;
                }
            }
            K_ASYNC => {
                // async is a modifier only when a function / arrow head follows on the same line.
                let nx = tokens.next_sig(end);
                let same_line = nx < tokens.n && !tokens.line_break_between(end, nx);
                let nk = if nx < tokens.n { tokens.base_kind(nx) } else { 0 };
                let nc = if nx < tokens.n { tokens.src[nx] } else { 0 };
                let follows = same_line
                    && (nk == tk!(Ident)
                        || (nk >= OP_KIND_BASE && (nc == b'(' || nc == b'*' || nc == b'['))
                        || nk == tk!(String)
                        || nk == tk!(Number)
                        || nk == tk!(PrivateIdent));
                if !follows {
                    return 0;
                }
            }
            K_TYPE | K_INTERFACE | K_NAMESPACE | K_MODULE | K_DECLARE | K_ABSTRACT | K_GLOBAL => {
                // Statement-level TS declarations only.
                let nx = tokens.next_sig(end);
                let nk = if nx < tokens.n { tokens.base_kind(nx) } else { 0 };
                let nc = if nx < tokens.n { tokens.src[nx] } else { 0 };
                let same_line = nx < tokens.n && !tokens.line_break_between(end, nx);
                let starts_decl = same_line
                    && (nk == tk!(Ident)
                        || nk == tk!(String)
                        || (kw == K_GLOBAL && nk >= OP_KIND_BASE && nc == b'{'));
                let at_stmt = self.at_stmt_start()
                    || matches!(self.prev_kw, K_EXPORT | K_DECLARE | K_DEFAULT | K_ABSTRACT)
                    || (kw == K_NAMESPACE && self.stmt_reg() == S_EXPORT_AS);
                if !(tokens.ts && starts_decl && at_stmt) {
                    return 0;
                }
            }
            K_AS | K_SATISFIES => {
                // Only after a value in TS; export as opens export as namespace X.
                let export_as =
                    kw == K_AS && self.stmt_reg() == S_EXPORT && self.prev_kw == K_EXPORT;
                let in_module_clause = self.top_kind() == FrameKind::ModuleSpec
                    || matches!(self.stmt_reg(), S_IMPORT | S_EXPORT);
                if !export_as && (!tokens.ts || self.operand_allowed() || in_module_clause) {
                    return 0;
                }
            }
            K_STATIC => {
                if !(self.top_kind() == FrameKind::ClassBody && self.top().state == M_KEY_POS) {
                    return 0;
                }
            }
            K_IMPLEMENTS => {
                if self.top_kind() != FrameKind::ClassHead {
                    return 0;
                }
            }
            K_FROM if !matches!(self.stmt_reg(), S_IMPORT | S_EXPORT | S_IMPORT_NAME) => {
                return 0;
            }
            _ => {}
        }
        kw
    }

    fn statement_keyword_break(&mut self, kw: u8, newline: bool, ts: bool) {
        let import_attrs =
            kw == K_WITH && matches!(self.stmt_reg(), S_IMPORT | S_IMPORT_NAME | S_EXPORT);
        if !self.operand_allowed()
            && kw != 0
            && self.decorator == 0
            && !import_attrs
            && is_stmt_keyword(kw, newline, ts)
            && matches!(
                self.top_kind(),
                FrameKind::Root
                    | FrameKind::Block
                    | FrameKind::FnBody
                    | FrameKind::ArrowBody
                    | FrameKind::StaticBlock
                    | FrameKind::FnHead
            )
        {
            self.end_statement();
        }
    }

    /// A register that takes this word as a declared name (type X, enum E, import x...).
    fn declared_name(&mut self, tokens: &Tokens, end: usize, kw: u8) -> bool {
        match self.stmt_reg() {
            S_BREAK => {
                // break label: the statement is complete.
                self.set_stmt_reg(S_NONE);
                self.value_done();
                self.stmt_done = true;
                self.after_statement();
            }
            S_TYPE => {
                self.set_stmt_reg(S_TYPE_NAME);
                self.value_done();
            }
            S_NAMESPACE | S_ENUM if kw == 0 || kw == K_GLOBAL => {
                // The declared name (dotted for namespaces).
                self.value_done();
            }
            S_IMPORT if kw == 0 || kw == K_TYPE => {
                // import x / import type x / import x = ...
                if kw == K_TYPE && self.prev_kw == K_IMPORT {
                    let nx = tokens.next_sig(end);
                    let nk = if nx < tokens.n { tokens.base_kind(nx) } else { 0 };
                    if nx < tokens.n
                        && (nk == tk!(Ident)
                            || (nk >= OP_KIND_BASE && matches!(tokens.src[nx], b'{' | b'*')))
                    {
                        self.prev_kw = K_TYPE;
                        return true;
                    }
                }
                self.set_stmt_reg(S_IMPORT_NAME);
                self.value_done();
            }
            S_EXPORT_AS if kw == K_NAMESPACE => {
                self.set_stmt_reg(S_EXPORT_AS_NS);
                self.set_operand();
                self.prev_kw = K_NAMESPACE;
            }
            S_EXPORT_AS_NS => {
                self.set_stmt_reg(S_NONE);
                self.value_done();
                self.stmt_done = true;
                self.after_statement();
            }
            _ => return false,
        }
        true
    }

    fn keyword_word(&mut self, tokens: &Tokens, end: usize, kw: u8) {
        let stmt_reg = self.stmt_reg();
        let at_start = self.at_stmt_start();
        match kw {
            0 => self.plain_word(tokens, end, at_start),
            K_THIS | K_SUPER | K_NULL | K_TRUE | K_FALSE => {
                self.plain_word(tokens, end, false);
            }
            K_FUNCTION => {
                let value = !self.at_stmt_start()
                    && !self.export_default
                    && self.decorator == 0
                    && self.operand_allowed()
                    && !matches!(self.prev_kw, K_EXPORT | K_DECLARE);
                let is_async = self.prev_async;
                let f = self.push(FrameKind::FnHead);
                f.is_value = value;
                f.is_async = is_async;
                f.is_generator = false;
                self.set_value();
                self.clear_prev();
                self.prev_kw = K_FUNCTION;
                self.export_default = false;
            }
            K_CLASS => {
                let value = if self.decorator != 0 {
                    self.decorator == 2
                } else {
                    !self.at_stmt_start()
                        && !self.export_default
                        && self.operand_allowed()
                        && !matches!(self.prev_kw, K_EXPORT | K_DECLARE | K_ABSTRACT)
                };
                let f = self.push(FrameKind::ClassHead);
                f.is_value = value;
                self.set_value();
                self.clear_prev();
                self.prev_kw = K_CLASS;
                self.export_default = false;
                self.decorator = 0;
            }
            K_EXTENDS => {
                // Class heritage expression.
                if self.top_kind() == FrameKind::ClassHead {
                    self.top_mut().state = C_EXTENDS;
                }
                self.keyword(K_EXTENDS);
            }
            K_IMPLEMENTS => {
                // Type references follow.
                self.top_mut().state = C_IMPLEMENTS;
                self.keyword(K_IMPLEMENTS);
            }
            K_WITH if matches!(stmt_reg, S_IMPORT | S_IMPORT_NAME | S_EXPORT) => {
                // Import attributes: from "x" with { type: "json" }.
                self.keyword(K_WITH);
            }
            K_IF | K_WHILE | K_FOR | K_WITH | K_SWITCH | K_CATCH => {
                self.operand_done();
                if kw == K_CATCH {
                    // catch { without a binding.
                    self.expect = Expect::Statement;
                }
                self.prev_kw = kw;
                let hi = self.stmt_frame();
                self.frames[hi].head = match kw {
                    K_IF => H_IF,
                    K_WHILE => H_WHILE,
                    K_FOR => H_FOR,
                    K_WITH => H_WITH,
                    K_SWITCH => H_SWITCH,
                    _ => H_CATCH,
                };
                self.for_await = false;
            }
            K_ELSE | K_DO | K_TRY | K_FINALLY => {
                self.expect = Expect::Statement;
                self.clear_prev();
                self.prev_kw = kw;
            }
            K_RETURN | K_THROW | K_YIELD | K_AWAIT | K_TYPEOF | K_VOID | K_DELETE | K_NEW
            | K_IN | K_INSTANCEOF | K_OF | K_DEBUGGER => {
                if self.top_kind() == FrameKind::Head && matches!(kw, K_OF | K_IN) {
                    self.top_mut().state = F_ITER;
                }
                self.keyword(kw);
            }
            K_CASE => {
                self.set_stmt_reg(S_CASE);
                self.keyword(K_CASE);
            }
            K_DEFAULT => {
                if self.prev_kw == K_EXPORT {
                    self.export_default = true;
                    self.set_stmt_reg(S_NONE);
                    self.set_operand();
                } else {
                    self.set_stmt_reg(S_CASE);
                    self.set_operand();
                }
                self.clear_prev();
                self.prev_kw = K_DEFAULT;
            }
            K_BREAK | K_CONTINUE => {
                self.set_stmt_reg(S_BREAK);
                self.keyword(kw);
            }
            K_VAR | K_CONST | K_LET | K_USING => {
                if kw == K_CONST {
                    // const enum
                    let nx = tokens.next_sig(end);
                    if nx < tokens.n
                        && tokens.base_kind(nx) == tk!(Ident)
                        && tokens.ident_is(nx, b"enum")
                    {
                        self.keyword(K_CONST);
                        return;
                    }
                }
                if let Some(di) = self.decl_frame() {
                    self.frames[di].state = D_BINDING;
                }
                self.keyword(kw);
            }
            K_IMPORT => {
                let nx = tokens.next_sig(end);
                let nc = if nx < tokens.n { tokens.src[nx] } else { 0 };
                if nx < tokens.n
                    && tokens.base_kind(nx) >= OP_KIND_BASE
                    && (nc == b'(' || nc == b'.')
                {
                    // import(...) / import.meta: an expression.
                    self.set_value();
                    self.clear_prev();
                    self.prev_kw = K_IMPORT;
                } else {
                    self.set_stmt_reg(S_IMPORT);
                    self.keyword(K_IMPORT);
                }
            }
            K_EXPORT => {
                self.set_stmt_reg(S_EXPORT);
                self.keyword(K_EXPORT);
            }
            K_AS => {
                if stmt_reg == S_EXPORT && self.prev_kw == K_EXPORT {
                    self.set_stmt_reg(S_EXPORT_AS);
                    self.keyword(K_AS);
                } else {
                    self.open_region(R_EXPR, false);
                    self.prev_kw = K_AS;
                }
            }
            K_SATISFIES => {
                self.open_region(R_EXPR, false);
                self.prev_kw = K_SATISFIES;
            }
            K_ASYNC => {
                // A modifier is transparent: the function or arrow it modifies decides.
                self.clear_prev();
                self.prev_async = true;
                self.prev_kw = K_ASYNC;
            }
            K_TYPE => {
                self.set_stmt_reg(S_TYPE);
                self.keyword(K_TYPE);
            }
            K_INTERFACE => {
                // Same head frame as a class: extends, < and the body may come on later lines.
                let f = self.push(FrameKind::ClassHead);
                f.is_value = false;
                f.reg = C_INTERFACE;
                self.set_value();
                self.clear_prev();
                self.prev_kw = K_INTERFACE;
            }
            K_ENUM => {
                self.set_stmt_reg(S_ENUM);
                self.keyword(K_ENUM);
            }
            K_NAMESPACE | K_MODULE => {
                self.set_stmt_reg(S_NAMESPACE);
                let declare = self.prev_kw == K_DECLARE;
                self.keyword(kw);
                if declare && kw == K_MODULE {
                    // declare module "x" may have no body.
                    self.set_stmt_reg(S_DECLARE_MODULE);
                }
            }
            K_DECLARE | K_ABSTRACT | K_GLOBAL => {
                self.keyword(kw);
                if kw == K_GLOBAL {
                    self.expect = Expect::Statement;
                }
            }
            K_STATIC => {
                self.top_mut().mods |= MOD_STATIC;
                self.keyword(K_STATIC);
            }
            K_FROM => {
                self.keyword(K_FROM);
            }
            _ => {
                // Any other keyword spelling used as a plain word.
                self.plain_word(tokens, end, at_start);
            }
        }
    }

    pub(super) fn plain_word(&mut self, tokens: &Tokens, end: usize, at_start: bool) {
        // Declarator binding.
        if let Some(si) = self.decl_frame()
            && self.frames[si].state == D_BINDING
            && si == self.frames.len() - 1
        {
            if self.frames[si].kind == FrameKind::Head {
                self.frames[si].decl_binding = true;
            }
            self.frames[si].state = D_BOUND;
            self.value_done();
            return;
        }
        // Label candidate: a lone identifier at statement start.
        if at_start && self.stmt_reg() == S_NONE && self.top_kind() != FrameKind::Head {
            let nx = tokens.next_sig(end);
            if nx < tokens.n
                && tokens.base_kind(nx) >= OP_KIND_BASE
                && tokens.src[nx] == b':'
                && tokens.src[nx + 1] != b':'
            {
                self.set_stmt_reg(S_LABEL);
            }
        }
        // async x => ...: remember the modifier for the arrow.
        let is_async = self.prev_async;
        self.value_done();
        self.arrow_async = is_async;
    }

    pub(super) fn member_word(&mut self, tokens: &Tokens, pos: usize, end: usize, kw: u8) -> usize {
        let is_class = self.top_kind() == FrameKind::ClassBody;
        // Modifiers apply when a key can follow on the same line.
        let nx = tokens.next_sig(end);
        let nk = if nx < tokens.n { tokens.base_kind(nx) } else { 0 };
        let nc = if nx < tokens.n { tokens.src[nx] } else { 0 };
        let key_follows = nx < tokens.n
            && !tokens.line_break_between(end, nx)
            && (matches!(
                nk,
                tk!(Ident) | tk!(String) | tk!(Number) | tk!(BigInt) | tk!(PrivateIdent)
            ) || (nk >= OP_KIND_BASE && (nc == b'[' || nc == b'*' || nc == b'#')));
        let key_follows_any_line = nx < tokens.n
            && (matches!(
                nk,
                tk!(Ident) | tk!(String) | tk!(Number) | tk!(BigInt) | tk!(PrivateIdent)
            ) || (nk >= OP_KIND_BASE && (nc == b'[' || nc == b'*' || nc == b'#')));
        if kw == K_ASYNC && key_follows {
            self.top_mut().mods |= MOD_ASYNC;
            self.operand_done();
            return end;
        }
        if is_class && kw == K_STATIC {
            if nx < tokens.n && nk >= OP_KIND_BASE && nc == b'{' {
                self.top_mut().mods |= MOD_STATIC;
                self.keyword(K_STATIC);
                return end;
            }
            if key_follows_any_line {
                self.top_mut().mods |= MOD_STATIC;
                self.operand_done();
                return end;
            }
        }
        if is_class
            && matches!(
                kw,
                K_PUBLIC
                    | K_PRIVATE
                    | K_PROTECTED
                    | K_READONLY
                    | K_ABSTRACT
                    | K_OVERRIDE
                    | K_DECLARE
                    | K_ACCESSOR
            )
            && key_follows_any_line
        {
            self.operand_done();
            return end;
        }
        if kw == 0
            && (tokens.ident_is(pos, b"get") || tokens.ident_is(pos, b"set"))
            && key_follows_any_line
        {
            self.operand_done();
            return end;
        }
        // The key itself.
        self.top_mut().state = M_KEY_SEEN;
        self.value_done();
        end
    }
}
