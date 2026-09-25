//! Words: which spellings are keywords here (contextual keywords), the statement keywords,
//! plain names, and member keys with their modifiers.

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use super::*;

/// Does the word start a statement no expression can continue? Reserved statement keywords always
/// do, contextual ones only after a line break.
#[rustfmt::skip::macros(tk)]
fn is_stmt_keyword(kw: u8, newline: bool, ts: bool) -> bool {
    match kw {
        tk!(
            KwIf | KwFor | KwWhile | KwReturn | KwVar | KwConst | KwSwitch | KwTry | KwThrow | KwDo
            | KwWith | KwBreak | KwContinue | KwDebugger | KwFunction | KwClass | KwImport
            | KwExport | KwEnum
        ) => true,
        tk!(
            KwLet | KwAsync | KwType | KwInterface | KwDeclare | KwNamespace | KwModule | KwAbstract
            | KwUsing
        ) => newline && (ts || matches_tk!(kw, KwLet | KwAsync | KwUsing)),
        _ => false,
    }
}

#[rustfmt::skip::macros(matches_tk, tk)]
impl Walk {
    /// Step the word at `pos`; returns its end.
    pub(super) fn step_word(&mut self, tokens: &Tokens, pos: usize, newline: bool) -> usize {
        let end = tokens.next_start(pos + 1);
        let kw = if self.after_dot { 0 } else { tokens.word_kw(pos, end - pos) };

        if self.in_type() {
            self.type_word(kw);
            return end;
        }
        // `for await (`: the `await` belongs to the head.
        if kw == tk!(KwAwait) && self.prev_kw == tk!(KwFor) {
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

    /// A word inside a type: a prefix or infix type keyword, else a type atom.
    fn type_word(&mut self, kw: u8) {
        match kw {
            tk!(
                KwKeyof | KwTypeof | KwReadonly | KwUnique | KwInfer | KwAbstract | KwNew
                | KwAsserts | KwImport | KwExtends | KwIs | KwIn | KwAs | KwSatisfies
            ) => {
                self.type_operator();
                self.prev_kw = kw;
                if kw == tk!(KwExtends) {
                    // A conditional type: the `?` and `:` to come belong to the type, not to an
                    // enclosing expression.
                    if let Some(i) = self.region_index() {
                        self.frames[i].open_questions += 1;
                    }
                }
            }
            _ => {
                // A statement keyword right after a completed type is an error on the same line
                // and was handled by the break rule on a new one; read it as an atom.
                self.type_atom(false);
                if keyword_type(kw) {
                    self.prev_kw = kw;
                }
            }
        }
    }

    /// Is the spelling with keyword code `kw` a keyword here? A contextual keyword (`yield`,
    /// `await`, `of`, `let`, `using`, `async`, the TypeScript declaration words, `as`,
    /// `satisfies`, `static`, `implements`, `from`) is a plain name (0) unless its position and
    /// the token after it say otherwise; every other code stands.
    fn resolve_keyword(&self, tokens: &Tokens, end: usize, kw: u8) -> u8 {
        match kw {
            tk!(KwYield) => {
                if !self.yield_is_keyword() {
                    return 0;
                }
            }
            tk!(KwAwait) => {
                if !self.await_is_keyword() {
                    return 0;
                }
            }
            tk!(KwOf) => {
                if !(self.top_kind() == FrameKind::Head
                    && self.top().head == H_FOR
                    && self.top().state == F_BOUND)
                {
                    return 0;
                }
            }
            tk!(KwLet) => {
                let nx = tokens.peek(end);
                let ok = nx.kind == tk!(Ident)
                    || (nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'[' | b'{'));
                let at_stmt = self.at_stmt_start()
                    || self.top_kind() == FrameKind::Head
                    || matches_tk!(self.prev_kw, KwDeclare | KwExport);
                if !ok || !at_stmt {
                    return 0;
                }
            }
            tk!(KwUsing) => {
                let nx = tokens.peek(end);
                let ok = nx.kind == tk!(Ident)
                    && !tokens.line_break_between(end, nx.pos)
                    && (self.at_stmt_start() || matches_tk!(self.prev_kw, KwDeclare | KwExport));
                if !ok {
                    return 0;
                }
            }
            tk!(KwAsync) => {
                // `async` is a modifier only when the next token is on the same line and continues
                // a function / arrow head.
                let nx = tokens.peek(end);
                let follows = (matches_tk!(nx.kind, Ident | String | Number | PrivateIdent)
                    || (nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'(' | b'*' | b'[')))
                    && !tokens.line_break_between(end, nx.pos);
                if !follows {
                    return 0;
                }
            }
            tk!(
                KwType | KwInterface | KwNamespace | KwModule | KwDeclare | KwAbstract | KwGlobal
            ) => {
                // Statement-level TS declarations only.
                let nx = tokens.peek(end);
                let starts_decl = (matches_tk!(nx.kind, Ident | String)
                    || (kw == tk!(KwGlobal) && nx.kind >= OP_KIND_BASE && nx.byte == b'{'))
                    && !tokens.line_break_between(end, nx.pos);
                let at_stmt = self.at_stmt_start()
                    || matches_tk!(self.prev_kw, KwExport | KwDeclare | KwDefault | KwAbstract)
                    || (kw == tk!(KwNamespace) && self.stmt_reg() == S_EXPORT_AS);
                if !(tokens.ts && starts_decl && at_stmt) {
                    return 0;
                }
            }
            tk!(KwAs | KwSatisfies) => {
                // Only after a value in an expression, in TS; `export as` opens `export as
                // namespace X`.
                let export_as =
                    kw == tk!(KwAs) && self.stmt_reg() == S_EXPORT && self.prev_kw == tk!(KwExport);
                let in_module_clause = self.top_kind() == FrameKind::ModuleSpec
                    || matches!(self.stmt_reg(), S_IMPORT | S_EXPORT);
                if !export_as && (!tokens.ts || self.operand_allowed() || in_module_clause) {
                    return 0;
                }
            }
            tk!(KwStatic) => {
                if !(self.top_kind() == FrameKind::ClassBody && self.top().state == M_KEY_POS) {
                    return 0;
                }
            }
            tk!(KwImplements) => {
                if self.top_kind() != FrameKind::ClassHead {
                    return 0;
                }
            }
            tk!(KwFrom) if !matches!(self.stmt_reg(), S_IMPORT | S_EXPORT | S_IMPORT_NAME) => {
                return 0;
            }
            _ => {}
        }
        kw
    }

    /// A statement keyword that cannot continue an expression starts a new statement even
    /// without a separator.
    fn statement_keyword_break(&mut self, kw: u8, newline: bool, ts: bool) {
        let import_attrs =
            kw == tk!(KwWith) && matches!(self.stmt_reg(), S_IMPORT | S_IMPORT_NAME | S_EXPORT);
        if !self.operand_allowed()
            && kw != 0
            && self.decorator == 0
            && !import_attrs
            && is_stmt_keyword(kw, newline, ts)
            && (self.top_kind().is_stmt_holder() || self.top_kind() == FrameKind::FnHead)
        {
            self.end_statement();
        }
    }

    /// A statement register that takes this word as a declared name (`break label`, `type X`,
    /// `namespace N`, `enum E`, `import x`, `export as namespace N`). True when it did.
    fn declared_name(&mut self, tokens: &Tokens, end: usize, kw: u8) -> bool {
        match self.stmt_reg() {
            S_BREAK => {
                // `break label`: the statement is complete.
                self.set_stmt_reg(S_NONE);
                self.value_done();
                self.stmt_done = true;
                self.after_statement();
            }
            S_TYPE => {
                self.set_stmt_reg(S_TYPE_NAME);
                self.value_done();
            }
            S_NAMESPACE | S_ENUM if kw == 0 || kw == tk!(KwGlobal) => {
                // The declared name (dotted for namespaces).
                self.value_done();
            }
            S_IMPORT if kw == 0 || kw == tk!(KwType) => {
                // `import x` / `import type x` / `import x = ...`
                if kw == tk!(KwType) && self.prev_kw == tk!(KwImport) {
                    let nx = tokens.peek(end);
                    if nx.kind == tk!(Ident)
                        || (nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'{' | b'*'))
                    {
                        self.prev_kw = tk!(KwType);
                        return true;
                    }
                }
                self.set_stmt_reg(S_IMPORT_NAME);
                self.value_done();
            }
            S_EXPORT_AS if kw == tk!(KwNamespace) => {
                self.set_stmt_reg(S_EXPORT_AS_NS);
                self.set_operand();
                self.prev_kw = tk!(KwNamespace);
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

    /// The transition of the keyword `kw` (0: a plain name) in expression or statement position.
    fn keyword_word(&mut self, tokens: &Tokens, end: usize, kw: u8) {
        let stmt_reg = self.stmt_reg();
        let at_start = self.at_stmt_start();
        match kw {
            0 => self.plain_word(tokens, end, at_start),
            tk!(KwThis | KwSuper | KwNull | KwTrue | KwFalse) => {
                self.plain_word(tokens, end, false);
            }
            tk!(KwFunction) => {
                let value = !self.at_stmt_start()
                    && !self.export_default
                    && self.decorator == 0
                    && self.operand_allowed()
                    && !matches_tk!(self.prev_kw, KwExport | KwDeclare);
                let is_async = self.prev_async;
                let f = self.push(FrameKind::FnHead);
                f.is_value = value;
                f.is_async = is_async;
                f.is_generator = false;
                self.set_value();
                self.clear_prev();
                self.prev_kw = tk!(KwFunction);
                self.export_default = false;
            }
            tk!(KwClass) => {
                let value = if self.decorator != 0 {
                    self.decorator == 2
                } else {
                    !self.at_stmt_start()
                        && !self.export_default
                        && self.operand_allowed()
                        && !matches_tk!(self.prev_kw, KwExport | KwDeclare | KwAbstract)
                };
                let f = self.push(FrameKind::ClassHead);
                f.is_value = value;
                self.set_value();
                self.clear_prev();
                self.prev_kw = tk!(KwClass);
                self.export_default = false;
                self.decorator = 0;
            }
            tk!(KwExtends) => {
                // Class heritage expression.
                if self.top_kind() == FrameKind::ClassHead {
                    self.top_mut().state = C_EXTENDS;
                }
                self.keyword(tk!(KwExtends));
            }
            tk!(KwImplements) => {
                // Type references follow.
                self.top_mut().state = C_IMPLEMENTS;
                self.keyword(tk!(KwImplements));
            }
            tk!(KwWith) if matches!(stmt_reg, S_IMPORT | S_IMPORT_NAME | S_EXPORT) => {
                // Import attributes: `from "x" with { type: "json" }`.
                self.keyword(tk!(KwWith));
            }
            tk!(KwIf | KwWhile | KwFor | KwWith | KwSwitch | KwCatch) => {
                self.operand_done();
                if kw == tk!(KwCatch) {
                    // `catch {` without a binding.
                    self.expect = Expect::Statement;
                }
                self.prev_kw = kw;
                let hi = self.stmt_frame();
                self.frames[hi].head = match kw {
                    tk!(KwIf) => H_IF,
                    tk!(KwWhile) => H_WHILE,
                    tk!(KwFor) => H_FOR,
                    tk!(KwWith) => H_WITH,
                    tk!(KwSwitch) => H_SWITCH,
                    _ => H_CATCH,
                };
            }
            tk!(KwElse | KwDo | KwTry | KwFinally) => {
                self.expect = Expect::Statement;
                self.clear_prev();
                self.prev_kw = kw;
            }
            tk!(
                KwReturn | KwThrow | KwYield | KwAwait | KwTypeof | KwVoid | KwDelete | KwNew | KwIn
                | KwInstanceof | KwOf | KwDebugger
            ) => {
                if self.top_kind() == FrameKind::Head && matches_tk!(kw, KwOf | KwIn) {
                    self.top_mut().state = F_ITER;
                }
                self.keyword(kw);
            }
            tk!(KwCase) => {
                self.set_stmt_reg(S_CASE);
                self.keyword(tk!(KwCase));
            }
            tk!(KwDefault) => {
                if self.prev_kw == tk!(KwExport) {
                    self.export_default = true;
                    self.set_stmt_reg(S_NONE);
                    self.set_operand();
                } else {
                    self.set_stmt_reg(S_CASE);
                    self.set_operand();
                }
                self.clear_prev();
                self.prev_kw = tk!(KwDefault);
            }
            tk!(KwBreak | KwContinue) => {
                self.set_stmt_reg(S_BREAK);
                self.keyword(kw);
            }
            tk!(KwVar | KwConst | KwLet | KwUsing) => {
                if kw == tk!(KwConst) {
                    // `const enum`
                    let nx = tokens.peek(end);
                    if nx.kind == tk!(Ident) && tokens.ident_is(nx.pos, b"enum") {
                        self.keyword(tk!(KwConst));
                        return;
                    }
                }
                if let Some(di) = self.decl_frame() {
                    self.frames[di].state = D_BINDING;
                }
                self.keyword(kw);
            }
            tk!(KwImport) => {
                let nx = tokens.peek(end);
                if nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'(' | b'.') {
                    // `import(...)` / `import.meta`: an expression.
                    self.set_value();
                    self.clear_prev();
                    self.prev_kw = tk!(KwImport);
                } else {
                    self.set_stmt_reg(S_IMPORT);
                    self.keyword(tk!(KwImport));
                }
            }
            tk!(KwExport) => {
                self.set_stmt_reg(S_EXPORT);
                self.keyword(tk!(KwExport));
            }
            tk!(KwAs) => {
                if stmt_reg == S_EXPORT && self.prev_kw == tk!(KwExport) {
                    self.set_stmt_reg(S_EXPORT_AS);
                    self.keyword(tk!(KwAs));
                } else {
                    self.open_region(R_EXPR, false);
                    self.prev_kw = tk!(KwAs);
                }
            }
            tk!(KwSatisfies) => {
                self.open_region(R_EXPR, false);
                self.prev_kw = tk!(KwSatisfies);
            }
            tk!(KwAsync) => {
                // A modifier: the function or arrow it modifies decides expression-ness, so it is
                // transparent.
                self.clear_prev();
                self.prev_async = true;
                self.prev_kw = tk!(KwAsync);
            }
            tk!(KwType) => {
                self.set_stmt_reg(S_TYPE);
                self.keyword(tk!(KwType));
            }
            tk!(KwInterface) => {
                // Same head frame as a class: `extends`, `<` and the body may follow on later
                // lines.
                let f = self.push(FrameKind::ClassHead);
                f.is_value = false;
                f.reg = C_INTERFACE;
                self.set_value();
                self.clear_prev();
                self.prev_kw = tk!(KwInterface);
            }
            tk!(KwEnum) => {
                self.set_stmt_reg(S_ENUM);
                self.keyword(tk!(KwEnum));
            }
            tk!(KwNamespace | KwModule) => {
                self.set_stmt_reg(S_NAMESPACE);
                let declare = self.prev_kw == tk!(KwDeclare);
                self.keyword(kw);
                if declare && kw == tk!(KwModule) {
                    // `declare module "x"` may have no body.
                    self.set_stmt_reg(S_DECLARE_MODULE);
                }
            }
            tk!(KwDeclare | KwAbstract | KwGlobal) => {
                self.keyword(kw);
                if kw == tk!(KwGlobal) {
                    self.expect = Expect::Statement;
                }
            }
            tk!(KwStatic) => {
                self.top_mut().mods |= MOD_STATIC;
                self.keyword(tk!(KwStatic));
            }
            tk!(KwFrom) => {
                self.keyword(tk!(KwFrom));
            }
            _ => {
                // Any other keyword spelling used as a plain word.
                self.plain_word(tokens, end, at_start);
            }
        }
    }

    /// A plain identifier (or keyword used as a name) in expression / statement position.
    fn plain_word(&mut self, tokens: &Tokens, end: usize, at_start: bool) {
        // Declarator binding.
        if self.top_declarator() == D_BINDING {
            self.top_mut().state = D_BOUND;
            self.value_done();
            return;
        }
        // Label candidate: a lone identifier at statement start.
        if at_start && self.stmt_reg() == S_NONE && self.top_kind() != FrameKind::Head {
            let nx = tokens.peek(end);
            if nx.kind >= OP_KIND_BASE && nx.byte == b':' && tokens.src[nx.pos + 1] != b':' {
                self.set_stmt_reg(S_LABEL);
            }
        }
        // `async x => ...`: remember the modifier for the arrow.
        let is_async = self.prev_async;
        self.value_done();
        self.arrow_async = is_async;
    }

    /// A word at member-key position of an object literal or class body.
    fn member_word(&mut self, tokens: &Tokens, pos: usize, end: usize, kw: u8) -> usize {
        let is_class = self.top_kind() == FrameKind::ClassBody;
        // Modifiers apply when a key can follow on the same line.
        let nx = tokens.peek(end);
        let key_follows_any_line =
            matches_tk!(nx.kind, Ident | String | Number | BigInt | PrivateIdent)
                || (nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'[' | b'*' | b'#'));
        let key_follows = key_follows_any_line && !tokens.line_break_between(end, nx.pos);
        if kw == tk!(KwAsync) && key_follows {
            self.top_mut().mods |= MOD_ASYNC;
            self.operand_done();
            return end;
        }
        if is_class && kw == tk!(KwStatic) {
            if nx.kind >= OP_KIND_BASE && nx.byte == b'{' {
                self.top_mut().mods |= MOD_STATIC;
                self.keyword(tk!(KwStatic));
                return end;
            }
            if key_follows_any_line {
                self.top_mut().mods |= MOD_STATIC;
                self.operand_done();
                return end;
            }
        }
        if is_class
            && matches_tk!(
                kw,
                KwPublic | KwPrivate | KwProtected | KwReadonly | KwAbstract | KwOverride
                | KwDeclare | KwAccessor
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
